/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use core::cell::UnsafeCell;

// use log::info;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: KernAllocator = KernAllocator::new();

#[derive(Debug)]
pub struct KernAllocator(pub UnsafeCell<super::pmm::BitmapAllocator>);

impl KernAllocator {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(super::pmm::BitmapAllocator::new()))
    }
}

unsafe impl Sync for KernAllocator {}

unsafe impl core::alloc::GlobalAlloc for KernAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // info!("Allocating {:X?}", layout);
        if let Some(ptr) = self
            .0
            .get()
            .as_mut()
            .unwrap()
            .alloc((layout.size() + 0xFFF) / 0x1000)
        {
            // info!("ret: {:#X?}", ptr);
            ptr.add(amd64::paging::PHYS_VIRT_OFFSET)
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        // info!("Deallocating {:X?} at {:#X?}", layout, ptr);
        self.0.get().as_mut().unwrap().free(
            ptr.sub(amd64::paging::PHYS_VIRT_OFFSET),
            (layout.size() + 0xFFF) / 0x1000,
        );
    }
}

#[alloc_error_handler]
pub fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {:#X?}", layout);
}
