// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[global_allocator]
static GLOBAL_ALLOCATOR: KernAllocator = KernAllocator;

struct KernAllocator;

unsafe impl core::alloc::GlobalAlloc for KernAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let state = super::state::SYS_STATE.get().as_mut().unwrap();
        state
            .pmm
            .get_mut()
            .unwrap()
            .lock()
            .alloc(((layout.size() + 0xFFF) / 0x1000) as _)
            .map_or(core::ptr::null_mut(), |ptr| {
                ptr.add(amd64::paging::PHYS_VIRT_OFFSET as _)
            })
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        // Memory is already zeroed by the allocator
        self.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let state = super::state::SYS_STATE.get().as_mut().unwrap();
        state.pmm.get_mut().unwrap().lock().free(
            ptr.sub(amd64::paging::PHYS_VIRT_OFFSET as _),
            ((layout.size() + 0xFFF) / 0x1000) as _,
        );
    }
}

#[alloc_error_handler]
pub fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {:#X?}", layout);
}
