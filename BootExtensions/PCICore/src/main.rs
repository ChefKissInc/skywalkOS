// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![feature(alloc_error_handler)]

use alloc::boxed::Box;

use hashbrown::HashMap;
use kernel::{port::Port, SystemCall};

mod allocator;
mod logger;
mod panic;
mod regs;

#[macro_use]
extern crate log;
// #[macro_use]
extern crate alloc;

type Result<T> = core::result::Result<T, ()>;

trait PCIController: Sync {
    unsafe fn cfg_read8(&self, addr: pci_core::PCIAddress, off: u8) -> Result<u8>;
    unsafe fn cfg_read16(&self, addr: pci_core::PCIAddress, off: u8) -> Result<u16>;
    unsafe fn cfg_read32(&self, addr: pci_core::PCIAddress, off: u8) -> Result<u32>;
    unsafe fn cfg_write8(&self, addr: pci_core::PCIAddress, off: u8, value: u8) -> Result<()>;
    unsafe fn cfg_write16(&self, addr: pci_core::PCIAddress, off: u8, value: u16) -> Result<()>;
    unsafe fn cfg_write32(&self, addr: pci_core::PCIAddress, off: u8, value: u32) -> Result<()>;
}

struct PCIPIOController;

impl PCIPIOController {
    #[inline]
    unsafe fn send_addr(addr: pci_core::PCIAddress, off: u8) -> Result<()> {
        if addr.segment != 0 {
            return Err(());
        }

        Port::<u32, u32>::new(0xCF8).write(
            (u32::from(addr.bus) << 16)
                | (u32::from(addr.slot) << 11)
                | (u32::from(addr.func) << 8)
                | (u32::from(off) & !3u32)
                | 0x8000_0000,
        );
        Ok(())
    }
}

impl PCIController for PCIPIOController {
    unsafe fn cfg_read8(&self, addr: pci_core::PCIAddress, off: u8) -> Result<u8> {
        Self::send_addr(addr, off)?;
        Ok(Port::<u8, u8>::new(0xCFC + (u16::from(off) & 3)).read())
    }

    unsafe fn cfg_read16(&self, addr: pci_core::PCIAddress, off: u8) -> Result<u16> {
        Self::send_addr(addr, off)?;
        Ok(Port::<u16, u16>::new(0xCFC + (u16::from(off) & 3)).read())
    }

    unsafe fn cfg_read32(&self, addr: pci_core::PCIAddress, off: u8) -> Result<u32> {
        Self::send_addr(addr, off)?;
        Ok(Port::<u32, u32>::new(0xCFC + (u16::from(off) & 3)).read())
    }

    unsafe fn cfg_write8(&self, addr: pci_core::PCIAddress, off: u8, value: u8) -> Result<()> {
        Self::send_addr(addr, off)?;
        Port::<u8, u8>::new(0xCFC + (u16::from(off) & 3)).write(value);
        Ok(())
    }

    unsafe fn cfg_write16(&self, addr: pci_core::PCIAddress, off: u8, value: u16) -> Result<()> {
        Self::send_addr(addr, off)?;
        Port::<u16, u16>::new(0xCFC + (u16::from(off) & 3)).write(value);
        Ok(())
    }

    unsafe fn cfg_write32(&self, addr: pci_core::PCIAddress, off: u8, value: u32) -> Result<()> {
        Self::send_addr(addr, off)?;
        Port::<u32, u32>::new(0xCFC + (u16::from(off) & 3)).write(value);
        Ok(())
    }
}

#[used]
#[no_mangle]
static __stack_chk_guard: u64 = 0x595E9FBD94FDA766;

#[no_mangle]
extern "C" fn __stack_chk_fail() {
    panic!("stack check failure");
}

#[no_mangle]
extern "C" fn _start() -> ! {
    logger::init();

    let controller: Box<dyn PCIController> = Box::new(PCIPIOController);
    for bus in 0..=255 {
        for slot in 0..32 {
            for func in 0..8 {
                let addr = pci_core::PCIAddress {
                    segment: 0,
                    bus,
                    slot,
                    func,
                };
                let is_multifunction = unsafe {
                    controller
                        .cfg_read8(addr, pci_core::PCICfgOffset::HeaderType as u8)
                        .unwrap()
                        & 0x80
                        != 0
                };
                let vendor_id = unsafe {
                    controller
                        .cfg_read16(addr, pci_core::PCICfgOffset::VendorID as u8)
                        .unwrap()
                };
                if vendor_id == 0xFFFF || vendor_id == 0x0000 {
                    if is_multifunction {
                        continue;
                    } else {
                        break;
                    }
                }

                let device_id = unsafe {
                    controller
                        .cfg_read16(addr, pci_core::PCICfgOffset::DeviceID as u8)
                        .unwrap()
                };
                let class_code = unsafe {
                    controller
                        .cfg_read8(addr, pci_core::PCICfgOffset::ClassCode as u8)
                        .unwrap()
                };

                info!(
                    "PCI {:04x}:{:02x}:{:02x}.{} {:04x}:{:04x} {:02x}",
                    addr.segment, addr.bus, addr.slot, addr.func, vendor_id, device_id, class_code,
                );

                if !is_multifunction {
                    break;
                }
            }
        }
    }

    let mut device_owners = HashMap::new();

    loop {
        let Some(msg) = (unsafe { SystemCall::receive_message().unwrap() }) else {
            unsafe { SystemCall::skip() }
            continue;
        };
        if msg.proc_id == 0 {
        } else {
            device_owners.insert(msg.proc_id, pci_core::PCIAddress::default());
        }
        unsafe { SystemCall::ack(msg.id).unwrap() }
    }
}
