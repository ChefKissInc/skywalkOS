//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::boxed::Box;

use amd64::{
    paging::{pml4::PML4 as PML4Trait, PageTableEntry},
    registers::msr::{
        pat::{PATEntry, PageAttributeTable},
        ModelSpecificReg,
    },
};

#[repr(transparent)]
#[derive(Debug)]
pub struct PageTableLvl4(amd64::paging::PageTable);

impl PageTableLvl4 {
    pub fn new() -> Self {
        Self(amd64::paging::PageTable::new())
    }

    #[inline]
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

    #[inline]
    pub unsafe fn map_mmio(
        &mut self,
        virt: usize,
        phys: usize,
        count: usize,
        flags: PageTableEntry,
    ) {
        debug_assert!(!flags.pwt());
        debug_assert!(!flags.pcd());
        self.map_pages(virt, phys, count, flags.with_huge_or_pat(true))
    }
}

impl PML4Trait for PageTableLvl4 {
    const VIRT_OFF: usize = amd64::paging::PHYS_VIRT_OFFSET;

    #[inline]
    fn get_entry(&mut self, offset: usize) -> &mut amd64::paging::PageTableEntry {
        &mut self.0.entries[offset]
    }

    #[inline]
    fn alloc_entry() -> usize {
        Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as usize
            - amd64::paging::PHYS_VIRT_OFFSET
    }
}
