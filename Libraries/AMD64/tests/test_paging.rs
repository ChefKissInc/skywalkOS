// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

#![deny(
    warnings,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    unused_extern_crates,
    rust_2021_compatibility
)]

use amd64::paging::pml4::PML4 as PML4Trait;

#[repr(transparent)]
pub struct PML4(amd64::paging::PageTable);

impl PML4 {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(amd64::paging::PageTable {
            entries: [amd64::paging::PageTableEntry::default(); 512],
        })
    }
}

impl PML4Trait for PML4 {
    const VIRT_OFF: u64 = 0;

    fn get_entry(&mut self, offset: u64) -> &mut amd64::paging::PageTableEntry {
        &mut self.0.entries[offset as usize]
    }

    fn alloc_entry(&self) -> u64 {
        Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as u64
    }
}

#[test]
fn test_map_higher_half_phys() {
    unsafe {
        let mut pml4 = Box::new(PML4::new());
        pml4.map_higher_half();

        for i in 0..2048 {
            assert_eq!(
                pml4.virt_to_entry_addr(amd64::paging::PHYS_VIRT_OFFSET + 0x20_0000 * i),
                Some(0x20_0000 * i)
            );
        }
    }
}

#[test]
fn test_map_higher_half_kern() {
    unsafe {
        let mut pml4 = Box::new(PML4::new());
        pml4.map_higher_half();

        for i in 0..1024 {
            assert_eq!(
                pml4.virt_to_entry_addr(amd64::paging::KERNEL_VIRT_OFFSET + 0x20_0000 * i),
                Some(0x20_0000 * i)
            );
        }
    }
}
