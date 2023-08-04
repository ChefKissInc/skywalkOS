// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

fn alloc_entry() -> u64 {
    Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as u64
}

#[test]
fn test_map_higher_half_phys() {
    unsafe {
        let mut pml4 = Box::new(amd64::paging::PageTable::new());
        pml4.map_higher_half(&alloc_entry, 0);

        for i in 0..2048 {
            assert_eq!(
                pml4.virt_to_entry_addr(amd64::paging::PHYS_VIRT_OFFSET + 0x20_0000 * i, 0),
                Some(0x20_0000 * i)
            );
        }
    }
}

#[test]
fn test_map_higher_half_kern() {
    unsafe {
        let mut pml4 = Box::new(amd64::paging::PageTable::new());
        pml4.map_higher_half(&alloc_entry, 0);

        for i in 0..1024 {
            assert_eq!(
                pml4.virt_to_entry_addr(amd64::paging::KERNEL_VIRT_OFFSET + 0x20_0000 * i, 0),
                Some(0x20_0000 * i)
            );
        }
    }
}
