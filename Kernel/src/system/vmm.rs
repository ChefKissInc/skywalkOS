// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

use alloc::boxed::Box;

use amd64::{
    msr::{
        pat::{PATEntry, PageAttributeTable},
        ModelSpecificReg,
    },
    paging::{pml4::PML4 as PML4Trait, PageTableEntry},
};

#[repr(transparent)]
pub struct PageTableLvl4(amd64::paging::PageTable);

impl PageTableLvl4 {
    #[inline]
    #[must_use]
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
}

impl PML4Trait for PageTableLvl4 {
    const VIRT_OFF: u64 = amd64::paging::PHYS_VIRT_OFFSET;

    fn get_entry(&mut self, offset: u64) -> &mut amd64::paging::PageTableEntry {
        &mut self.0.entries[offset as usize]
    }

    fn alloc_entry(&self) -> u64 {
        Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as u64
            - amd64::paging::PHYS_VIRT_OFFSET
    }
}
