// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

use alloc::{collections::VecDeque, vec::Vec};

use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;
use tungstenkit::{port::Port, syscall::SystemCall};

mod allocator;
mod logger;
mod panic;
mod regs;

#[used]
#[no_mangle]
static __stack_chk_guard: u64 = 0x595E_9FBD_94FD_A766;

#[no_mangle]
extern "C" fn __stack_chk_fail() {
    panic!("stack check failure");
}

#[derive(Debug, Default, Clone, Copy)]
struct PCIAddress {
    pub segment: u16,
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
}

#[bitfield(bits = 16)]
#[derive(Debug)]
#[repr(u16)]
struct PCICommand {
    pub pio: bool,
    pub mmio: bool,
    pub bus_master: bool,
    pub special_cycle: bool,
    pub mem_write_and_invl: bool,
    pub vga_palette_snoop: bool,
    pub parity_error_resp: bool,
    pub wait_cycle_ctl: bool,
    pub serr: bool,
    pub fast_back_to_back: bool,
    pub disable_intrs: bool,
    #[skip]
    __: B5,
}

#[allow(dead_code)]
#[derive(IntoPrimitive)]
#[repr(u8)]
enum PCICfgOffset {
    VendorId = 0x0,
    DeviceId = 0x2,
    Command = 0x4,
    Status = 0x6,
    RevisionId = 0x8,
    ProgIf = 0x9,
    ClassCode = 0xA,
    Subclass = 0xB,
    CacheLineSize = 0xC,
    LatencyTimer = 0xD,
    HeaderType = 0xE,
    Bist = 0xF,
    BaseAddr0 = 0x10,
    BaseAddr1 = 0x14,
    BaseAddr2 = 0x18,
    BaseAddr3 = 0x1C,
    BaseAddr4 = 0x20,
    BaseAddr5 = 0x24,
    CardBusCisPtr = 0x28,
    SubSystemVendorId = 0x2C,
    SubSystemId = 0x2E,
    ExpansionRomBase = 0x30,
    CapabilitiesPtr = 0x34,
    InterruptLine = 0x3C,
    InterruptPin = 0x3D,
    MinimumGrant = 0x3E,
    MaximumLatency = 0x3F,
}

trait PCIControllerIO: Sync {
    unsafe fn cfg_read8(&self, addr: PCIAddress, off: u8) -> u8;
    unsafe fn cfg_read16(&self, addr: PCIAddress, off: u8) -> u16;
    unsafe fn cfg_read32(&self, addr: PCIAddress, off: u8) -> u32;
    unsafe fn cfg_write8(&self, addr: PCIAddress, off: u8, value: u8);
    unsafe fn cfg_write16(&self, addr: PCIAddress, off: u8, value: u16);
    unsafe fn cfg_write32(&self, addr: PCIAddress, off: u8, value: u32);
}

struct PCIDevice<'a> {
    addr: PCIAddress,
    controller: &'a PCIController,
}

#[allow(dead_code)]
impl<'a> PCIDevice<'a> {
    #[inline]
    #[must_use]
    pub const fn new(addr: PCIAddress, controller: &'a PCIController) -> Self {
        Self { addr, controller }
    }

    pub unsafe fn is_multifunction(&self) -> bool {
        self.cfg_read8::<_, u8>(PCICfgOffset::HeaderType) & 0x80 != 0
    }

    pub unsafe fn cfg_read8<A: Into<u8>, R: From<u8>>(&self, off: A) -> R {
        self.controller.cfg_read8(self.addr, off.into()).into()
    }

    pub unsafe fn cfg_read16<A: Into<u8>, R: From<u16>>(&self, off: A) -> R {
        self.controller.cfg_read16(self.addr, off.into()).into()
    }

    pub unsafe fn cfg_read32<A: Into<u8>, R: From<u32>>(&self, off: A) -> R {
        self.controller.cfg_read32(self.addr, off.into()).into()
    }

    pub unsafe fn cfg_write8<A: Into<u8>, R: Into<u8>>(&self, off: A, value: R) {
        self.controller
            .cfg_write8(self.addr, off.into(), value.into());
    }

    pub unsafe fn cfg_write16<A: Into<u8>, R: Into<u16>>(&self, off: A, value: R) {
        self.controller
            .cfg_write16(self.addr, off.into(), value.into());
    }

    pub unsafe fn cfg_write32<A: Into<u8>, R: Into<u32>>(&self, off: A, value: R) {
        self.controller
            .cfg_write32(self.addr, off.into(), value.into());
    }
}

struct PCIController;

impl PCIController {
    unsafe fn cfg_read8(&self, addr: PCIAddress, off: u8) -> u8 {
        PCIPortIO::new().cfg_read8(addr, off)
    }

    unsafe fn cfg_read16(&self, addr: PCIAddress, off: u8) -> u16 {
        PCIPortIO::new().cfg_read16(addr, off)
    }

    unsafe fn cfg_read32(&self, addr: PCIAddress, off: u8) -> u32 {
        PCIPortIO::new().cfg_read32(addr, off)
    }

    unsafe fn cfg_write8(&self, addr: PCIAddress, off: u8, value: u8) {
        PCIPortIO::new().cfg_write8(addr, off, value);
    }

    unsafe fn cfg_write16(&self, addr: PCIAddress, off: u8, value: u16) {
        PCIPortIO::new().cfg_write16(addr, off, value);
    }

    unsafe fn cfg_write32(&self, addr: PCIAddress, off: u8, value: u32) {
        PCIPortIO::new().cfg_write32(addr, off, value);
    }

    pub fn find<P: FnMut(&PCIDevice) -> bool>(&self, mut pred: P) -> Option<PCIDevice> {
        let mut scan_seg = |segment, bus_start, bus_end| {
            for bus in bus_start..=bus_end {
                for slot in 0..32 {
                    for func in 0..8 {
                        let addr = PCIAddress {
                            segment,
                            bus,
                            slot,
                            func,
                        };
                        let dev = PCIDevice::new(addr, self);
                        if pred(&dev) {
                            return Some(dev);
                        }
                    }
                }
            }
            None
        };

        scan_seg(0, 0, 255)
    }
}

#[derive(Clone)]
struct PCIPortIO;

impl PCIPortIO {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    unsafe fn send_addr(addr: PCIAddress, off: u8) {
        assert_eq!(addr.segment, 0, "Using segments on PCI non-express");

        Port::<u32, u32>::new(0xCF8).write(
            (u32::from(addr.bus) << 16)
                | (u32::from(addr.slot) << 11)
                | (u32::from(addr.func) << 8)
                | (u32::from(off) & !3u32)
                | 0x8000_0000,
        );
    }
}

impl PCIControllerIO for PCIPortIO {
    unsafe fn cfg_read8(&self, addr: PCIAddress, off: u8) -> u8 {
        Self::send_addr(addr, off);
        Port::<u8, u8>::new(0xCFC + (u16::from(off) & 3)).read()
    }

    unsafe fn cfg_read16(&self, addr: PCIAddress, off: u8) -> u16 {
        Self::send_addr(addr, off);
        Port::<u16, u16>::new(0xCFC + (u16::from(off) & 3)).read()
    }

    unsafe fn cfg_read32(&self, addr: PCIAddress, off: u8) -> u32 {
        Self::send_addr(addr, off);
        Port::<u32, u32>::new(0xCFC + (u16::from(off) & 3)).read()
    }

    unsafe fn cfg_write8(&self, addr: PCIAddress, off: u8, value: u8) {
        Self::send_addr(addr, off);
        Port::<u8, u8>::new(0xCFC + (u16::from(off) & 3)).write(value);
    }

    unsafe fn cfg_write16(&self, addr: PCIAddress, off: u8, value: u16) {
        Self::send_addr(addr, off);
        Port::<u16, u16>::new(0xCFC + (u16::from(off) & 3)).write(value);
    }

    unsafe fn cfg_write32(&self, addr: PCIAddress, off: u8, value: u32) {
        Self::send_addr(addr, off);
        Port::<u32, u32>::new(0xCFC + (u16::from(off) & 3)).write(value);
    }
}

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
        debug!("IRQ: {:#X?}", irq);
        unsafe { SystemCall::register_irq_handler(irq).unwrap() }
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
            SystemCall::skip();
        }
        self.pcm_out_transf_ctl
            .write(self.pcm_out_transf_ctl.read().with_last_ent_fire_intr(true));
    }

    pub unsafe fn set_bdl(&mut self) {
        self.buf.make_contiguous();
        self.bdl[0].addr =
            (self.buf.as_slices().0.as_ptr() as u64 - tungstenkit::USER_PHYS_VIRT_OFFSET) as _;
        self.pcm_out_bdl_addr
            .write((self.bdl.as_ptr() as u64 - tungstenkit::USER_PHYS_VIRT_OFFSET) as _);
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
extern "C" fn _start() -> ! {
    logger::init();

    let dev = PCIController
        .find(|dev| unsafe { dev.cfg_read16::<_, u16>(PCICfgOffset::ClassCode) == 0x0401 })
        .unwrap();
    let mut this = AC97::new(&dev);
    this.play_audio(include_bytes!("song.raw"));

    loop {
        let Some(msg) = (unsafe { SystemCall::receive_message().unwrap() }) else {
            unsafe { SystemCall::skip() };
            continue;
        };
        if msg.proc_id == 0 {
            if this.buf.is_empty() || !this.playing {
                this.playing = false;
            } else {
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
        unsafe { SystemCall::ack_message(msg.id).unwrap() }
    }
}
