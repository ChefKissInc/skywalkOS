// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

fn alloc_entry() -> u64 {
    Box::leak(Box::new(amd64::paging::PageTable::<0>::new())) as *mut _ as u64
}

#[test]
fn test_no_entry() {
    unsafe {
        let mut pml4 = Box::new(amd64::paging::PageTable::<0>::new());
        assert_eq!(pml4.virt_to_entry_addr(0), None);
        assert_eq!(pml4.virt_to_entry_addr_huge(0), None);
    }
}

#[test]
fn test_map() {
    unsafe {
        let mut pml4 = Box::new(amd64::paging::PageTable::<0>::new());
        pml4.map(
            &alloc_entry,
            0x20_0000,
            0x20_0000,
            1,
            amd64::paging::PageTableEntry::new().with_present(true),
        );
        assert_eq!(pml4.virt_to_entry_addr(0x20_0000), Some(0x20_0000));
    }
}

#[test]
fn test_map_higher_half_phys() {
    unsafe {
        let mut pml4 = Box::new(amd64::paging::PageTable::<0>::new());
        pml4.map_higher_half(&alloc_entry);

        for i in 0..2048 {
            assert_eq!(
                pml4.virt_to_entry_addr_huge(amd64::paging::PHYS_VIRT_OFFSET + 0x20_0000 * i),
                Some(0x20_0000 * i),
            );
        }
    }
}

#[test]
fn test_map_higher_half_kern() {
    unsafe {
        let mut pml4 = Box::new(amd64::paging::PageTable::<0>::new());
        pml4.map_higher_half(&alloc_entry);

        for i in 0..1024 {
            assert_eq!(
                pml4.virt_to_entry_addr_huge(amd64::paging::KERNEL_VIRT_OFFSET + 0x20_0000 * i),
                Some(0x20_0000 * i),
            );
        }
    }
}
