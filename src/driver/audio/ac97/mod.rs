// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::{collections::VecDeque, vec, vec::Vec};
use core::{cell::SyncUnsafeCell, mem::MaybeUninit};

use amd64::io::port::Port;
use log::debug;

use crate::{
    driver::pci::{PCICfgOffset, PCICommand, PCIDevice},
    sys::RegisterState,
};

mod regs;

pub struct AC97 {
    pub mixer: Port<u16, u16>,
    pub audio_bus: Port<u32, u32>,
    pcm_out_bdl_last_ent: Port<u8, u8>,
    pcm_out_bdl_addr: Port<u32, u32>,
    pcm_out_transf_ctl: Port<u8, regs::RegBoxTransfer>,
    buf: VecDeque<u8>,
    bdl: Vec<regs::BufferDescriptor>,
    playing: bool,
}

pub static INSTANCE: SyncUnsafeCell<MaybeUninit<AC97>> = SyncUnsafeCell::new(MaybeUninit::uninit());

unsafe extern "sysv64" fn handler(_state: &mut RegisterState) {
    let this = (*INSTANCE.get()).assume_init_mut();

    if this.buf.is_empty() || !this.playing {
        this.playing = false;
        return;
    }

    for _ in 0..(0xFFFE * 2) {
        this.buf.pop_front();
    }
    this.buf.make_contiguous();

    this.reset();
    this.set_bdl();
    this.begin_transfer();
}

impl AC97 {
    pub fn new(dev: &PCIDevice) -> Self {
        unsafe {
            dev.cfg_write16(
                PCICfgOffset::Command,
                dev.cfg_read16::<_, PCICommand>(PCICfgOffset::Command)
                    .with_pio(true)
                    .with_bus_master(true)
                    .with_disable_intrs(false),
            );

            let irq: u8 = dev.cfg_read8(PCICfgOffset::InterruptLine);
            debug!("IRQ: {:#X?}", irq);
            crate::driver::acpi::ioapic::wire_legacy_irq(irq, false);
            crate::driver::intrs::idt::set_handler(0x20 + irq, handler, true, true);
        }
        let audio_bus = unsafe { dev.cfg_read16::<_, u16>(PCICfgOffset::BaseAddr1) & !1u16 };
        let pcm_out_bdl_last_ent = Port::new(audio_bus + regs::AudioBusReg::PCMOutLastEnt as u16);
        let pcm_out_bdl_addr = Port::new(audio_bus + regs::AudioBusReg::PCMOutBDLAddr as u16);
        let pcm_out_transf_ctl = Port::<_, regs::RegBoxTransfer>::new(
            audio_bus + regs::AudioBusReg::PCMOutTransferControl as u16,
        );

        let audio_bus = Port::new(audio_bus);
        let mixer = Port::new(unsafe { dev.cfg_read16::<_, u16>(PCICfgOffset::BaseAddr0) & !1u16 });

        unsafe {
            audio_bus.write_off(
                audio_bus
                    .read_off::<_, regs::GlobalControl>(regs::AudioBusReg::GlobalControl)
                    .with_cold_reset(true)
                    .with_interrupts(true),
                regs::AudioBusReg::GlobalControl,
            );
            mixer.write_off(!0u16, regs::MixerReg::Reset);

            mixer.write_off(
                regs::MasterOutputVolume::new()
                    .with_right(0x3F)
                    .with_left(0x3F)
                    .with_mute(false),
                regs::MixerReg::MasterVolume,
            );
            mixer.write_off(
                regs::PcmOutputVolume::new()
                    .with_right(0x1F)
                    .with_left(0x1F)
                    .with_mute(false),
                regs::MixerReg::PCMOutVolume,
            );
            debug!(
                "Sample rate: {:#?}",
                mixer.read_off::<_, u16>(regs::MixerReg::SampleRate)
            );
            // BUG: QEMU has a bug and 48KHz audio doesn't work
            mixer.write_off(44100u16, regs::MixerReg::SampleRate);
        }

        let buf = VecDeque::with_capacity(4);
        let bdl = vec![regs::BufferDescriptor {
            addr: 0,
            samples: 0xFFFE,
            ctl: regs::BufferDescCtl::new()
                .with_last(true)
                .with_fire_interrupt(true),
        }];

        Self {
            mixer,
            audio_bus,
            pcm_out_bdl_addr,
            pcm_out_bdl_last_ent,
            pcm_out_transf_ctl,
            buf,
            bdl,
            playing: false,
        }
    }

    pub unsafe fn reset(&self) {
        self.pcm_out_transf_ctl
            .write(self.pcm_out_transf_ctl.read().with_reset(true));
        while self.pcm_out_transf_ctl.read().reset() {
            core::arch::asm!("pause");
        }
        self.pcm_out_transf_ctl.write(
            self.pcm_out_transf_ctl
                .read()
                .with_last_ent_fire_intr(true)
                .with_ioc_intr(true),
        );
    }

    pub unsafe fn set_bdl(&mut self) {
        self.bdl[0].addr = ((self.buf.as_slices().0.as_ptr() as u64
            - amd64::paging::PHYS_VIRT_OFFSET)
            & 0xFFFF_FFFF) as u32;
        self.pcm_out_bdl_addr.write(
            ((self.bdl.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET) & 0xFFFF_FFFF) as _,
        );
        self.pcm_out_bdl_last_ent.write(0);
    }

    pub unsafe fn begin_transfer(&self) {
        self.pcm_out_transf_ctl
            .write(self.pcm_out_transf_ctl.read().with_transfer_data(true));
    }

    pub fn start_playback(&mut self) {
        if !self.playing && !self.buf.is_empty() {
            self.playing = true;
            self.buf.make_contiguous();

            unsafe {
                self.reset();
                self.set_bdl();
                self.begin_transfer();
            }
        }
    }

    pub fn stop_playback(&mut self) {
        self.playing = false;
        unsafe {
            self.reset();
        }
    }

    pub fn play_audio(&mut self, data: &[u8]) {
        for a in data {
            self.buf.push_back(*a);
        }
        self.start_playback();
    }
}
