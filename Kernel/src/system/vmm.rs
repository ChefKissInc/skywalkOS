// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::boxed::Box;

use amd64::{
    msr::{
        pat::{PATEntry, PageAttributeTable},
        ModelSpecificReg,
    },
    paging::PageTableEntry,
};

#[repr(transparent)]
pub struct PageTableLvl4(amd64::paging::PageTable);

impl PageTableLvl4 {
    const VIRT_OFF: u64 = amd64::paging::PHYS_VIRT_OFFSET;

    #[inline]
    pub const fn new() -> Self {
        Self(amd64::paging::PageTable::new())
    }

    pub unsafe fn init(&mut self) {
        // Fix performance by utilising the PAT mechanism
        PageAttributeTable::new()
            .with_pat0(PATEntry::WriteBack)
            .with_pat1(PATEntry::WriteThrough)
            .with_pat2(PATEntry::WriteCombining)
            .with_pat3(PATEntry::WriteProtected)
            .write();

        self.map_higher_half();
        self.set();
    }

    pub unsafe fn map_mmio(&mut self, virt: u64, phys: u64, count: u64, flags: PageTableEntry) {
        debug_assert!(!flags.pwt());
        debug_assert!(!flags.pcd());
        self.map_pages(virt, phys, count, flags.with_huge_or_pat(true));
    }

    fn alloc_entry() -> u64 {
        Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as u64 - Self::VIRT_OFF
    }

    pub unsafe fn set(&mut self) {
        self.0.set(Self::VIRT_OFF);
    }

    pub unsafe fn map_pages(
        &mut self,
        virt: u64,
        phys: u64,
        count: u64,
        flags: amd64::paging::PageTableEntry,
    ) {
        self.0
            .map_pages(&Self::alloc_entry, virt, Self::VIRT_OFF, phys, count, flags);
    }

    pub unsafe fn map_huge_pages(
        &mut self,
        virt: u64,
        phys: u64,
        count: u64,
        flags: amd64::paging::PageTableEntry,
    ) {
        self.0
            .map_huge_pages(&Self::alloc_entry, virt, Self::VIRT_OFF, phys, count, flags);
    }

    pub unsafe fn map_higher_half(&mut self) {
        self.0.map_higher_half(&Self::alloc_entry, Self::VIRT_OFF);
    }
}
