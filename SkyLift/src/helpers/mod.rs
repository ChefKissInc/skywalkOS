// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub mod elf;
pub mod fb;
pub mod mem;
pub mod setup;

pub fn pa_to_kern_va<T>(v: *const T) -> *const T {
    (v as u64 + amd64::paging::PHYS_VIRT_OFFSET) as *const T
}

pub fn phys_to_kern_ref<T>(v: &'_ T) -> &'_ T {
    unsafe { &*pa_to_kern_va(v) }
}

pub fn phys_to_kern_slice_ref<T>(v: &'_ [T]) -> &'_ [T] {
    unsafe { core::slice::from_raw_parts(pa_to_kern_va(v.as_ptr()), v.len()) }
}
