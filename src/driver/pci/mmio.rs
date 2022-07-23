//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use acpi::tables::mcfg::MCFGEntry;
use amd64::paging::{pml4::PML4, PageTableEntry};

#[derive(Clone)]
pub struct PCIMemoryIO {
    entry: MCFGEntry,
}

impl PCIMemoryIO {
    pub fn new(entry: MCFGEntry) -> Self {
        Self { entry }
    }

    #[inline]
    unsafe fn get_addr(&self, addr: super::PCIAddress, off: u8) -> usize {
        let segment = self.entry.segment;
        assert_eq!(addr.segment, segment, "PCI Express segment mismatch");

        let phys = (self.entry.base as usize
            + (((addr.bus as usize - self.entry.bus_start as usize) << 20)
                | ((addr.slot as usize) << 15)
                | ((addr.func as usize) << 12)))
            | (off as usize);
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
    unsafe fn cfg_read(
        &self,
        addr: super::PCIAddress,
        off: u8,
        access_size: super::PCIIOAccessSize,
    ) -> u32 {
        let addr = self.get_addr(addr, off);

        match access_size {
            super::PCIIOAccessSize::Byte => (addr as *const u8).read_volatile().into(),
            super::PCIIOAccessSize::Word => (addr as *const u16).read_volatile().into(),
            super::PCIIOAccessSize::DWord => (addr as *const u32).read_volatile(),
        }
    }

    unsafe fn cfg_write(
        &self,
        addr: super::PCIAddress,
        off: u8,
        value: u32,
        access_size: super::PCIIOAccessSize,
    ) {
        let addr = self.get_addr(addr, off);

        match access_size {
            super::PCIIOAccessSize::Byte => (addr as *mut u8).write_volatile(value as _),
            super::PCIIOAccessSize::Word => (addr as *mut u16).write_volatile(value as _),
            super::PCIIOAccessSize::DWord => (addr as *mut u32).write_volatile(value),
        }
    }
}
