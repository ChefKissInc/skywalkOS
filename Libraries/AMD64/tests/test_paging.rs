// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![deny(warnings, clippy::nursery, unused_extern_crates)]

use amd64::paging::{
    PageTable, PageTableEntry, PageTableFlags, KERNEL_VIRT_OFFSET, PAGE_SIZE, PHYS_VIRT_OFFSET,
};

#[test]
fn test_flags() {
    assert_eq!(
        PageTableFlags::new_present()
            .with_writable(true)
            .as_entry(false),
        PageTableEntry::new().with_present(true).with_writable(true)
    );
    assert_eq!(
        PageTableFlags::new_present()
            .with_writable(true)
            .with_pat_entry(5)
            .as_entry(false),
        PageTableEntry::new()
            .with_present(true)
            .with_writable(true)
            .with_pwt(true)
            .with_pat(true)
    );
    assert_eq!(
        PageTableFlags::new_present()
            .with_writable(true)
            .with_pat_entry(7)
            .as_entry(false),
        PageTableEntry::new()
            .with_present(true)
            .with_writable(true)
            .with_pwt(true)
            .with_pcd(true)
            .with_pat(true)
    );
    assert_eq!(
        PageTableFlags::new_present()
            .with_writable(true)
            .with_pat_entry(5)
            .as_entry(true),
        PageTableEntry::new()
            .with_present(true)
            .with_writable(true)
            .with_pwt(true)
            .with_huge_or_pat(true)
    );
    assert_eq!(
        PageTableFlags::new_present()
            .with_writable(true)
            .with_pat_entry(7)
            .as_entry(true),
        PageTableEntry::new()
            .with_present(true)
            .with_writable(true)
            .with_pwt(true)
            .with_pcd(true)
            .with_huge_or_pat(true)
    );
}

fn alloc_entry() -> u64 {
    Box::leak(Box::new(PageTable::<0>::new())) as *mut _ as u64
}

#[test]
fn test_no_entry() {
    unsafe {
        let mut pml4 = Box::new(PageTable::<0>::new());
        assert_eq!(pml4.virt_to_phys(0), None);
    }
}

#[test]
fn test_map() {
    unsafe {
        let mut pml4 = Box::new(PageTable::<0>::new());
        pml4.map(
            &alloc_entry,
            0x20_0000,
            0x20_0000,
            1,
            PageTableFlags::new_present(),
        );
        assert_eq!(
            pml4.virt_to_phys(0x20_0000),
            Some((0x20_0000, PageTableFlags::new_present()))
        );
    }
}

#[test]
fn test_map_higher_half() {
    unsafe {
        let mut pml4 = Box::new(PageTable::<0>::new());
        pml4.map_higher_half(&alloc_entry);

        assert_eq!(pml4.virt_to_phys(PHYS_VIRT_OFFSET), None);
        assert_eq!(pml4.virt_to_phys(KERNEL_VIRT_OFFSET), None);

        for i in 1..0xFFFFF {
            assert_eq!(
                pml4.virt_to_phys(PHYS_VIRT_OFFSET + PAGE_SIZE * i),
                Some((
                    PAGE_SIZE * i,
                    PageTableFlags::new_present().with_writable(true)
                )),
            );
        }

        for i in 1..0x7FFFF {
            assert_eq!(
                pml4.virt_to_phys(KERNEL_VIRT_OFFSET + PAGE_SIZE * i),
                Some((
                    PAGE_SIZE * i,
                    PageTableFlags::new_present().with_writable(true)
                )),
            );
        }
    }
}
