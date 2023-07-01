// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::multiple_crate_versions)]

#[macro_use]
extern crate log;
// #[macro_use]
extern crate alloc;
#[macro_use]
extern crate itertools;

use alloc::boxed::Box;

use tkpci::{PCIAddress, PCICfgOffset};
use tungstenkit::{msg::Message, osdtentry::OSDTEntry, userspace::port::Port};

trait PCIControllerIO: Sync {
    unsafe fn read8(&self, addr: PCIAddress, off: u8) -> u8;
    unsafe fn read16(&self, addr: PCIAddress, off: u8) -> u16;
    unsafe fn read32(&self, addr: PCIAddress, off: u8) -> u32;
    unsafe fn write8(&self, addr: PCIAddress, off: u8, value: u8);
    unsafe fn write16(&self, addr: PCIAddress, off: u8, value: u16);
    unsafe fn write32(&self, addr: PCIAddress, off: u8, value: u32);
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
        self.read8::<_, u8>(PCICfgOffset::HeaderType) & 0x80 != 0
    }

    pub unsafe fn read8<A: Into<u8>, R: From<u8>>(&self, off: A) -> R {
        self.controller.read8(self.addr, off.into()).into()
    }

    pub unsafe fn read16<A: Into<u8>, R: From<u16>>(&self, off: A) -> R {
        self.controller.read16(self.addr, off.into()).into()
    }

    pub unsafe fn read32<A: Into<u8>, R: From<u32>>(&self, off: A) -> R {
        self.controller.read32(self.addr, off.into()).into()
    }

    pub unsafe fn write8<A: Into<u8>, R: Into<u8>>(&self, off: A, value: R) {
        self.controller.write8(self.addr, off.into(), value.into());
    }

    pub unsafe fn write16<A: Into<u8>, R: Into<u16>>(&self, off: A, value: R) {
        self.controller.write16(self.addr, off.into(), value.into());
    }

    pub unsafe fn write32<A: Into<u8>, R: Into<u32>>(&self, off: A, value: R) {
        self.controller.write32(self.addr, off.into(), value.into());
    }
}

struct PCIController;

impl PCIController {
    fn read8(&self, addr: PCIAddress, off: u8) -> u8 {
        unsafe { PCIPortIO::new().read8(addr, off) }
    }

    fn read16(&self, addr: PCIAddress, off: u8) -> u16 {
        unsafe { PCIPortIO::new().read16(addr, off) }
    }

    fn read32(&self, addr: PCIAddress, off: u8) -> u32 {
        unsafe { PCIPortIO::new().read32(addr, off) }
    }

    fn write8(&self, addr: PCIAddress, off: u8, value: u8) {
        unsafe {
            PCIPortIO::new().write8(addr, off, value);
        }
    }

    fn write16(&self, addr: PCIAddress, off: u8, value: u16) {
        unsafe {
            PCIPortIO::new().write16(addr, off, value);
        }
    }

    fn write32(&self, addr: PCIAddress, off: u8, value: u32) {
        unsafe {
            PCIPortIO::new().write32(addr, off, value);
        }
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
    unsafe fn read8(&self, addr: PCIAddress, off: u8) -> u8 {
        Self::send_addr(addr, off);
        Port::<u8, u8>::new(0xCFC + (u16::from(off) & 3)).read()
    }

    unsafe fn read16(&self, addr: PCIAddress, off: u8) -> u16 {
        Self::send_addr(addr, off);
        Port::<u16, u16>::new(0xCFC + (u16::from(off) & 3)).read()
    }

    unsafe fn read32(&self, addr: PCIAddress, off: u8) -> u32 {
        Self::send_addr(addr, off);
        Port::<u32, u32>::new(0xCFC + (u16::from(off) & 3)).read()
    }

    unsafe fn write8(&self, addr: PCIAddress, off: u8, value: u8) {
        Self::send_addr(addr, off);
        Port::<u8, u8>::new(0xCFC + (u16::from(off) & 3)).write(value);
    }

    unsafe fn write16(&self, addr: PCIAddress, off: u8, value: u16) {
        Self::send_addr(addr, off);
        Port::<u16, u16>::new(0xCFC + (u16::from(off) & 3)).write(value);
    }

    unsafe fn write32(&self, addr: PCIAddress, off: u8, value: u32) {
        Self::send_addr(addr, off);
        Port::<u32, u32>::new(0xCFC + (u16::from(off) & 3)).write(value);
    }
}

#[no_mangle]
extern "C" fn _start(instance: OSDTEntry) -> ! {
    tungstenkit::userspace::logger::init();

    let controller = Box::new(PCIController);
    for (bus, slot) in iproduct!(0..=255, 0..32) {
        for func in 0..8 {
            let addr = PCIAddress::new(0, bus, slot, func);
            let multifunction =
                (controller.read8(addr, PCICfgOffset::HeaderType as u8) & 0x80) != 0;
            let vendor_id = controller.read16(addr, PCICfgOffset::VendorID as u8);
            if vendor_id == 0xFFFF || vendor_id == 0x0000 {
                if multifunction {
                    continue;
                }
                break;
            }

            let device_id = controller.read16(addr, PCICfgOffset::DeviceID as u8);
            let class_code = controller.read8(addr, PCICfgOffset::ClassCode as u8);

            let ent = instance.new_child(None);
            ent.set_property("VendorID", vendor_id.into());
            ent.set_property("DeviceID", device_id.into());
            ent.set_property("ClassCode", class_code.into());

            if !multifunction {
                break;
            }
        }
    }

    loop {
        let msg = unsafe { Message::receive() };
        if msg.pid == 0 {
            continue;
        }

        if msg.data == (1u64).to_be_bytes() {
            info!("Received packet hello");
        }
    }
}
