// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::boxed::Box;

pub mod fb;
pub mod file;
pub mod mem;
pub mod parse_elf;
pub mod setup;

#[repr(transparent)]
pub struct PML4(amd64::paging::PageTable);

impl amd64::paging::pml4::PML4 for PML4 {
    const VIRT_OFF: u64 = 0;

    fn get_entry(&mut self, offset: u64) -> &mut amd64::paging::PageTableEntry {
        &mut self.0.entries[offset as usize]
    }

    fn alloc_entry(&self) -> u64 {
        Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as u64
    }
}

pub fn pa_to_kern_va<T>(v: *const T) -> *const T {
    (v as u64 + amd64::paging::PHYS_VIRT_OFFSET) as *const T
}

pub fn phys_to_kern_ref<T>(v: &'_ T) -> &'_ T {
    unsafe { pa_to_kern_va(v).as_ref().unwrap() }
}

pub fn phys_to_kern_slice_ref<T>(v: &'_ [T]) -> &'_ [T] {
    unsafe { core::slice::from_raw_parts(pa_to_kern_va(v.as_ptr()), v.len()) }
}
