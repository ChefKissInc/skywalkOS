//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::{collections::VecDeque, vec, vec::Vec};
use core::{cell::SyncUnsafeCell, mem::MaybeUninit};

use amd64::{cpu::RegisterState, io::port::Port};
use log::debug;
use modular_bitfield::prelude::*;

use super::pci::{PciCmd, PciConfigOffset, PciDevice, PciIo, PciIoAccessSize};

#[bitfield(bits = 16)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u16)]
pub struct MasterOutputVolume {
    pub right: B6,
    #[skip]
    __: B2,
    pub left: B6,
    #[skip]
    __: B1,
    pub mute: bool,
}

#[bitfield(bits = 16)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u16)]
pub struct PcmOutputVolume {
    pub right: B5,
    #[skip]
    __: B3,
    pub left: B5,
    #[skip]
    __: B2,
    pub mute: bool,
}

#[bitfield(bits = 8)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub struct RegBoxTransfer {
    pub transfer_data: bool,
    pub reset: bool,
    pub last_ent_fire_intr: bool,
    pub ioc_intr: bool,
    pub fifo_err_intr: bool,
    #[skip]
    __: B3,
}

#[bitfield(bits = 16)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u16)]
pub struct RegBoxStatus {
    pub transfer_data: bool,
    pub end_of_transfer: bool,
    pub last_ent_fire_intr: bool,
    pub ioc_intr: bool,
    pub fifo_err_intr: bool,
    #[skip]
    __: B11,
}

#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy)]
#[bits = 2]
pub enum PcmChannels {
    #[default]
    Two = 0,
    Four,
    Six,
}

#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy)]
#[bits = 2]
pub enum PcmOutMode {
    #[default]
    SixteenSamples = 0,
    TwentySamples,
}

#[bitfield(bits = 32)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u32)]
pub struct GlobalControl {
    pub interrupts: bool,
    pub cold_reset: bool,
    pub warm_reset: bool,
    pub shut_down: bool,
    #[skip]
    __: u16,
    pub channels: PcmChannels,
    pub pcm_out_mode: PcmOutMode,
    #[skip]
    __: u8,
}

#[bitfield(bits = 32)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u32)]
pub struct GlobalStatus {
    #[skip]
    __: B20,
    #[skip(setters)]
    pub channel_caps: PcmChannels,
    pub sample_caps: PcmOutMode,
    #[skip]
    __: u8,
}

#[bitfield(bits = 16)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u16)]
pub struct BufferDescCtl {
    #[skip]
    __: B14,
    pub last: bool,
    pub fire_interrupt: bool,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct BufferDescriptor {
    pub addr: u32,
    pub samples: u16,
    pub ctl: BufferDescCtl,
}

#[repr(u16)]
pub enum NamRegs {
    Reset = 0x0,
    MasterVolume = 0x2,
    PcmOutVolume = 0x18,
    SampleRate = 0x2C,
}

#[repr(u16)]
pub enum NabmRegs {
    PcmOutBdlAddr = 0x10,
    // PcmOutCurrentEnt = 0x14,
    PcmOutLastEnt = 0x15,
    PcmOutStatus = 0x16,
    // PcmOutTransferedSamples = 0x18,
    // PcmOutNextProcessedEnt = 0x1A,
    PcmOutTransferControl = 0x1B,
    GlobalControl = 0x2C,
    GlobalStatus = 0x30,
}

pub struct Ac97 {
    pub mixer_reset: Port<u16, u16>,
    pub mixer_master_vol: Port<u16, MasterOutputVolume>,
    pub mixer_pcm_vol: Port<u16, PcmOutputVolume>,
    pub mixer_sample_rate: Port<u16, u16>,
    pub global_ctl: Port<u32, GlobalControl>,
    pub global_sts: Port<u32, GlobalStatus>,
    pub pcm_out_bdl_last_ent: Port<u8, u8>,
    pub pcm_out_bdl_addr: Port<u32, u32>,
    pub pcm_out_transf_ctl: Port<u8, RegBoxTransfer>,
    pub pcm_out_transf_sts: Port<u16, RegBoxStatus>,
    pub buf: VecDeque<u8>,
    pub bdl: Vec<BufferDescriptor>,
}

pub static INSTANCE: SyncUnsafeCell<MaybeUninit<Ac97>> = SyncUnsafeCell::new(MaybeUninit::uninit());

unsafe extern "sysv64" fn handler(_state: &mut RegisterState) {
    let this = (&mut *INSTANCE.get()).assume_init_mut();

    for _ in 0..(0xFFFE * 2) {
        this.buf.pop_front();
    }
    this.buf.make_contiguous();

    this.reset();
    this.set_bdl();
    this.begin_transfer();
}

impl Ac97 {
    pub fn new<T: PciIo>(dev: PciDevice<T>) -> Self {
        unsafe {
            dev.cfg_write(
                PciConfigOffset::Command,
                u16::from(
                    PciCmd::from(
                        dev.cfg_read(PciConfigOffset::Command, PciIoAccessSize::Word) as u16,
                    )
                    .with_pio(true)
                    .with_bus_master(true)
                    .with_disable_intrs(false),
                ) as _,
                PciIoAccessSize::Word,
            );

            let irq = dev.cfg_read(PciConfigOffset::InterruptLine, PciIoAccessSize::Byte) as u8;
            debug!("IRQ: {:#X?}", irq);
            crate::driver::acpi::ioapic::wire_legacy_irq(irq, false);
            crate::sys::idt::set_handler(0x20 + irq, handler, true, true);
        }
        let audio_bus = unsafe {
            (dev.cfg_read(PciConfigOffset::BaseAddr1, PciIoAccessSize::DWord) as u16) & !1u16
        };
        let global_ctl = Port::<_, GlobalControl>::new(audio_bus + NabmRegs::GlobalControl as u16);
        let global_sts = Port::new(audio_bus + NabmRegs::GlobalStatus as u16);
        let pcm_out_bdl_last_ent = Port::new(audio_bus + NabmRegs::PcmOutLastEnt as u16);
        let pcm_out_bdl_addr = Port::new(audio_bus + NabmRegs::PcmOutBdlAddr as u16);
        let pcm_out_transf_ctl =
            Port::<_, RegBoxTransfer>::new(audio_bus + NabmRegs::PcmOutTransferControl as u16);
        let pcm_out_transf_sts = Port::new(audio_bus + NabmRegs::PcmOutStatus as u16);
        let mixer = unsafe {
            (dev.cfg_read(PciConfigOffset::BaseAddr0, PciIoAccessSize::DWord) as u16) & !1u16
        };
        let mixer_reset = Port::new(mixer + NamRegs::Reset as u16);
        let mixer_master_vol = Port::new(mixer + NamRegs::MasterVolume as u16);
        let mixer_pcm_vol = Port::new(mixer + NamRegs::PcmOutVolume as u16);
        let mixer_sample_rate = Port::new(mixer + NamRegs::SampleRate as u16);

        unsafe {
            // Resume from cold reset
            global_ctl.write(
                global_ctl
                    .read()
                    .with_cold_reset(true)
                    .with_interrupts(true),
            );
            mixer_reset.write(!0u16);

            // Set volume and sample rate
            mixer_master_vol.write(
                MasterOutputVolume::new()
                    .with_right(0x3F)
                    .with_left(0x3F)
                    .with_mute(false),
            );
            mixer_pcm_vol.write(
                PcmOutputVolume::new()
                    .with_right(0x1F)
                    .with_left(0x1F)
                    .with_mute(false),
            );
            debug!("Sample rate: {:#?}", mixer_sample_rate.read());
            // NOTE: QEMU has a bug and 48KHz audio doesn't work
            mixer_sample_rate.write(44100);
        }

        let buf = VecDeque::with_capacity(4);
        let bdl = vec![BufferDescriptor {
            addr: 0,
            samples: 0xFFFE,
            ctl: BufferDescCtl::new()
                .with_last(true)
                .with_fire_interrupt(true),
        }];

        Self {
            global_ctl,
            global_sts,
            mixer_reset,
            mixer_master_vol,
            mixer_pcm_vol,
            mixer_sample_rate,
            pcm_out_bdl_last_ent,
            pcm_out_bdl_addr,
            pcm_out_transf_ctl,
            pcm_out_transf_sts,
            buf,
            bdl,
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
        )
    }

    pub unsafe fn set_bdl(&mut self) {
        self.bdl[0].addr =
            (self.buf.as_slices().0.as_ptr() as usize - amd64::paging::PHYS_VIRT_OFFSET) as u32;
        self.pcm_out_bdl_addr
            .write((self.bdl.as_ptr() as usize - amd64::paging::PHYS_VIRT_OFFSET) as _);
        self.pcm_out_bdl_last_ent.write(0);
    }

    pub unsafe fn begin_transfer(&self) {
        self.pcm_out_transf_ctl
            .write(self.pcm_out_transf_ctl.read().with_transfer_data(true));
    }

    pub fn play_audio(&mut self, data: &[u8]) {
        for a in data {
            self.buf.push_back(*a);
        }
        self.buf.make_contiguous();
        unsafe {
            self.reset();
            self.set_bdl();
            self.begin_transfer();
        }
    }
}
