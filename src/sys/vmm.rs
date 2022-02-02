/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use alloc::boxed::Box;

use amd64::{
    paging::pml4::Pml4 as Pml4Trait,
    registers::msr::{Msr, Pat, PatEntry},
};
use log::info;

#[repr(transparent)]
#[derive(Debug)]
pub struct Pml4(amd64::paging::PageTable);

impl Pml4 {
    pub fn new() -> Self {
        Self(amd64::paging::PageTable {
            entries: [amd64::paging::PageTableEntry::default(); 512],
        })
    }

    pub unsafe fn init(&mut self) {
        self.map_higher_half();

        // Fix performance by utilising the PAT mechanism
        let new_pat = Pat::new()
            .with_pat0(PatEntry::WriteBack)
            .with_pat1(PatEntry::WriteThrough)
            .with_pat2(PatEntry::WriteCombining)
            .with_pat3(PatEntry::WriteProtected);
        info!("PAT before: {:#X?}\nAfter: {:#X?}", Pat::read(), new_pat);
        new_pat.write();

        self.set();
    }
}

impl Pml4Trait for Pml4 {
    const VIRT_OFF: usize = amd64::paging::PHYS_VIRT_OFFSET;

    fn get_entry(&mut self, offset: usize) -> &mut amd64::paging::PageTableEntry {
        &mut self.0.entries[offset]
    }

    fn alloc_entry() -> usize {
        Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as usize
            - amd64::paging::PHYS_VIRT_OFFSET
    }
}
