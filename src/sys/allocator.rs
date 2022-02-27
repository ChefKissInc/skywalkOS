//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

#[global_allocator]
pub static GLOBAL_ALLOCATOR: KernAllocator = KernAllocator;

pub struct KernAllocator;

unsafe impl core::alloc::GlobalAlloc for KernAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if let Some(ptr) = super::state::SYS_STATE
            .pmm
            .get()
            .as_mut()
            .unwrap()
            .get_mut()
            .expect("PMM uninitialised")
            .alloc((layout.size() + 0xFFF) / 0x1000)
        {
            ptr.add(amd64::paging::PHYS_VIRT_OFFSET)
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        super::state::SYS_STATE
            .pmm
            .get()
            .as_mut()
            .unwrap()
            .get_mut()
            .expect("PMM uninitialised")
            .free(
                ptr.sub(amd64::paging::PHYS_VIRT_OFFSET),
                (layout.size() + 0xFFF) / 0x1000,
            );
    }
}

#[alloc_error_handler]
pub fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {:#X?}", layout);
}
