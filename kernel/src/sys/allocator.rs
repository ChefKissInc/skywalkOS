// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[global_allocator]
static GLOBAL_ALLOCATOR: KernAllocator = KernAllocator;

struct KernAllocator;

unsafe impl core::alloc::GlobalAlloc for KernAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        (*super::state::SYS_STATE.get())
            .pmm
            .assume_init_mut()
            .lock()
            .alloc(((layout.size() + 0xFFF) / 0x1000).try_into().unwrap())
            .map_or(core::ptr::null_mut(), |ptr| {
                ptr.add(amd64::paging::PHYS_VIRT_OFFSET.try_into().unwrap())
            })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        (*super::state::SYS_STATE.get())
            .pmm
            .assume_init_mut()
            .lock()
            .free(
                ptr.sub(amd64::paging::PHYS_VIRT_OFFSET.try_into().unwrap()),
                ((layout.size() + 0xFFF) / 0x1000).try_into().unwrap(),
            );
    }
}

#[alloc_error_handler]
pub fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {:#X?}", layout);
}
