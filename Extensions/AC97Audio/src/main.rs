// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::multiple_crate_versions)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

use alloc::{collections::VecDeque, string::String, vec::Vec};

use fireworkkit::{
    msg::Message,
    osdtentry::{OSDTEntry, FKEXT_PROC_KEY},
    osvalue::OSValue,
    syscall::SystemCall,
    userspace::port::Port,
};
use hashbrown::HashMap;
use pcikit::{PCIAddress, PCICfgOffset, PCICommand, PCIDevice};

mod regs;

struct AC97 {
    pub _mixer: Port<u16, u16>,
    pub _audio_bus: Port<u32, u32>,
    pcm_out_bdl_last_ent: Port<u8, u8>,
    pcm_out_bdl_addr: Port<u32, u32>,
    pcm_out_transf_ctl: Port<u8, regs::RegBoxTransfer>,
    pub pcm_out_transf_status: Port<u16, u16>,
    buf: VecDeque<u8>,
    bdl: Vec<regs::BufferDescriptor>,
    playing: bool,
}

impl AC97 {
    #[inline]
    #[must_use]
    pub fn new(dev: &PCIDevice) -> Self {
        let irq: u8 = unsafe {
            dev.cfg_write16(
                PCICfgOffset::Command,
                dev.cfg_read16::<_, PCICommand>(PCICfgOffset::Command)
                    .with_pio(true)
                    .with_mmio(true)
                    .with_bus_master(true)
                    .with_disable_intrs(false),
            );
            dev.cfg_read8(PCICfgOffset::InterruptLine)
        };
        debug!("IRQ: {irq:#X?}");
        unsafe { SystemCall::register_irq_handler(irq) }
        let audio_bus = unsafe { dev.cfg_read16::<_, u16>(PCICfgOffset::BaseAddr1) & !1u16 };
        let pcm_out_bdl_last_ent = Port::new(audio_bus + regs::AudioBusReg::PCMOutLastEnt as u16);
        let pcm_out_bdl_addr = Port::new(audio_bus + regs::AudioBusReg::PCMOutBDLAddr as u16);
        let pcm_out_transf_ctl = Port::<_, regs::RegBoxTransfer>::new(
            audio_bus + regs::AudioBusReg::PCMOutTransferControl as u16,
        );
        let pcm_out_transf_status =
            Port::<_, u16>::new(audio_bus + regs::AudioBusReg::PCMOutStatus as u16);

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
            mixer.write_off(0u16, regs::MixerReg::Reset);

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
            mixer.write_off(0u16, regs::MixerReg::SampleRate);
            mixer.write_off(48000u16, regs::MixerReg::SampleRate);
        }

        let buf = VecDeque::with_capacity(4);
        let bdl = vec![regs::BufferDescriptor {
            samples: 0xFFFE,
            ctl: regs::BufferDescCtl::new()
                .with_last(true)
                .with_fire_interrupt(true),
            ..Default::default()
        }];

        Self {
            _mixer: mixer,
            _audio_bus: audio_bus,
            pcm_out_bdl_addr,
            pcm_out_bdl_last_ent,
            pcm_out_transf_ctl,
            pcm_out_transf_status,
            buf,
            bdl,
            playing: false,
        }
    }

    pub unsafe fn reset(&self) {
        self.pcm_out_transf_ctl
            .write(self.pcm_out_transf_ctl.read().with_reset(true));
        while self.pcm_out_transf_ctl.read().reset() {
            SystemCall::r#yield();
        }
        self.pcm_out_transf_ctl
            .write(self.pcm_out_transf_ctl.read().with_last_ent_fire_intr(true));
    }

    pub unsafe fn set_bdl(&mut self) {
        self.buf.make_contiguous();
        self.bdl[0].addr =
            (self.buf.as_slices().0.as_ptr() as u64 - fireworkkit::USER_VIRT_OFFSET) as _;
        self.pcm_out_bdl_addr
            .write((self.bdl.as_ptr() as u64 - fireworkkit::USER_VIRT_OFFSET) as _);
        self.pcm_out_bdl_last_ent.write(0);
    }

    pub unsafe fn begin_transfer(&self) {
        self.pcm_out_transf_ctl
            .write(self.pcm_out_transf_ctl.read().with_transfer_data(true));
    }

    pub fn start_playback(&mut self) {
        if !self.playing && !self.buf.is_empty() {
            self.playing = true;

            unsafe {
                self.reset();
                self.set_bdl();
                self.begin_transfer();
            }
        }
    }

    pub fn play_audio(&mut self, data: &[u8]) {
        self.buf.extend(data.iter().copied());
        self.start_playback();
    }
}

#[no_mangle]
extern "C" fn _start(instance: OSDTEntry) -> ! {
    fireworkkit::userspace::logger::init();

    let ent = instance.parent().unwrap();
    let mut addr = ent.get_property("Address");
    while addr.is_none() {
        addr = ent.get_property("Address");
        unsafe {
            SystemCall::r#yield();
        }
    }
    let addr: HashMap<String, OSValue> = addr.unwrap().try_into().unwrap();
    let addr: PCIAddress = {
        let segment: u16 = addr.get("Segment").cloned().unwrap().try_into().unwrap();
        let bus: u8 = addr.get("Bus").cloned().unwrap().try_into().unwrap();
        let slot: u8 = addr.get("Slot").cloned().unwrap().try_into().unwrap();
        let func: u8 = addr.get("Function").cloned().unwrap().try_into().unwrap();
        PCIAddress::new(segment, bus, slot, func)
    };
    let pid: u64 = ent
        .parent()
        .unwrap()
        .get_property(FKEXT_PROC_KEY)
        .unwrap()
        .try_into()
        .unwrap();

    let dev = PCIDevice::new(pid, addr);
    let mut this = AC97::new(&dev);
    this.play_audio(include_bytes!("test.dat"));

    loop {
        let msg = unsafe { Message::recv() };
        if msg.pid != 0 {
            continue;
        }

        if this.buf.is_empty() || !this.playing {
            this.playing = false;
            unsafe {
                this.reset();
            }
            continue;
        }

        if this.buf.len() < 0xFFFE * 2 {
            this.buf.resize(0xFFFE * 2, 0);
        }
        this.buf.drain(0..0xFFFE * 2);
        unsafe {
            this.pcm_out_transf_status.write(0x1C);
            this.reset();
            this.set_bdl();
            this.begin_transfer();
        }
    }
}
