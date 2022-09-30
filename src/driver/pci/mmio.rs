// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use acpi::tables::mcfg::MCFGEntry;
use amd64::paging::{pml4::PML4, PageTableEntry};

#[derive(Clone)]
pub struct PCIMemoryIO {
    entry: MCFGEntry,
}

impl PCIMemoryIO {
    #[must_use]
    pub const fn new(entry: MCFGEntry) -> Self {
        Self { entry }
    }

    #[inline]
    unsafe fn get_addr(&self, addr: super::PCIAddress, off: u8) -> u64 {
        let segment = self.entry.segment;
        assert_eq!(addr.segment, segment, "PCI Express segment mismatch");

        let phys = (self.entry.base
            + (((u64::from(addr.bus) - u64::from(self.entry.bus_start)) << 20)
                | (u64::from(addr.slot) << 15)
                | (u64::from(addr.func) << 12)))
            | u64::from(off);
        let virt = phys + amd64::paging::PHYS_VIRT_OFFSET;
        (*crate::sys::state::SYS_STATE.get())
            .pml4
            .assume_init_mut()
            .map_pages(
                virt,
                phys,
                1,
                PageTableEntry::new().with_present(true).with_writable(true),
            );
        virt
    }
}

impl super::PCIControllerIO for PCIMemoryIO {
    unsafe fn cfg_read8(&self, addr: super::PCIAddress, off: u8) -> u8 {
        (self.get_addr(addr, off) as *const u8).read_volatile()
    }

    unsafe fn cfg_read16(&self, addr: super::PCIAddress, off: u8) -> u16 {
        (self.get_addr(addr, off) as *const u16).read_volatile()
    }

    unsafe fn cfg_read32(&self, addr: super::PCIAddress, off: u8) -> u32 {
        (self.get_addr(addr, off) as *const u32).read_volatile()
    }

    unsafe fn cfg_write8(&self, addr: super::PCIAddress, off: u8, value: u8) {
        (self.get_addr(addr, off) as *mut u8).write_volatile(value);
    }

    unsafe fn cfg_write16(&self, addr: super::PCIAddress, off: u8, value: u16) {
        (self.get_addr(addr, off) as *mut u16).write_volatile(value);
    }

    unsafe fn cfg_write32(&self, addr: super::PCIAddress, off: u8, value: u32) {
        (self.get_addr(addr, off) as *mut u32).write_volatile(value);
    }
}
