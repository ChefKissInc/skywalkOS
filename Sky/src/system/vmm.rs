// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::boxed::Box;

use amd64::{
    msr::{
        pat::{PATEntry, PageAttributeTable},
        ModelSpecificReg,
    },
    paging::{PageTable, PageTableFlags},
};

#[repr(transparent)]
pub struct PageTableLvl4(PageTable<{ amd64::paging::PHYS_VIRT_OFFSET }>);

impl Default for PageTableLvl4 {
    fn default() -> Self {
        Self::new()
    }
}

impl PageTableLvl4 {
    #[inline]
    pub const fn new() -> Self {
        Self(amd64::paging::PageTable::new())
    }

    fn alloc_entry() -> u64 {
        Box::leak(Box::new(PageTable::<0>::new())) as *mut _ as u64
            - amd64::paging::PHYS_VIRT_OFFSET
    }

    pub unsafe fn set_cr3(&mut self) {
        self.0.set_cr3();
    }

    pub unsafe fn map(&mut self, virt: u64, phys: u64, count: u64, flags: PageTableFlags) {
        self.0.map(&Self::alloc_entry, virt, phys, count, flags);
    }

    pub unsafe fn map_higher_half(&mut self) {
        self.0.map_higher_half(&Self::alloc_entry);
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
        self.set_cr3();
    }

    pub unsafe fn map_mmio(&mut self, virt: u64, phys: u64, count: u64, flags: PageTableFlags) {
        self.map(virt, phys, count, flags.with_pat_entry(1));
    }
}
