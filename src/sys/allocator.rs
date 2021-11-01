use core::{cell::UnsafeCell, ptr::null_mut};

/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#[global_allocator]
pub static GLOBAL_ALLOCATOR: KernAllocator = KernAllocator::new();

pub struct KernAllocator {
    pmm: UnsafeCell<spin::Once<super::pmm::BitmapAllocator>>,
}

impl KernAllocator {
    pub const fn new() -> Self {
        Self {
            pmm: UnsafeCell::new(spin::Once::new()),
        }
    }

    pub fn init(&self, allocator: super::pmm::BitmapAllocator) {
        unsafe {
            (*self.pmm.get()).call_once(|| allocator);
        }
    }
}

unsafe impl Sync for KernAllocator {}

unsafe impl core::alloc::GlobalAlloc for KernAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if let Some(pmm) = (*self.pmm.get()).get_mut() {
            if let Ok(ptr) = pmm.alloc(layout.size()) {
                ptr.add(amd64::paging::PHYS_VIRT_OFFSET as usize)
            } else {
                null_mut()
            }
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if let Some(pmm) = (*self.pmm.get()).get_mut() {
            assert!(ptr as u64 > amd64::paging::PHYS_VIRT_OFFSET);
            pmm.free(ptr.sub(amd64::paging::PHYS_VIRT_OFFSET as usize), layout.size());
        } else {
            panic!(
                "Failed to deallocate memory at {:#X?}, layout = {:#X?}",
                ptr, layout
            );
        }
    }
}

#[alloc_error_handler]
pub fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {:#X?}", layout);
}
