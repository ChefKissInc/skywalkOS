/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use alloc::boxed::Box;

#[repr(transparent)]
#[derive(Debug)]
pub struct Pml4(amd64::paging::PageTable);

impl Pml4 {
    pub fn new() -> Self {
        Self(amd64::paging::PageTable {
            entries: [amd64::paging::PageTableEntry::default(); 512],
        })
    }
}

impl amd64::paging::pml4::Pml4 for Pml4 {
    const VIRT_OFF: usize = amd64::paging::PHYS_VIRT_OFFSET;

    fn get_entry(&mut self, offset: usize) -> &mut amd64::paging::PageTableEntry {
        &mut self.0.entries[offset]
    }

    fn alloc_entry() -> usize {
        Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as usize
            - amd64::paging::PHYS_VIRT_OFFSET
    }
}
