// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[global_allocator]
static GLOBAL_ALLOCATOR: KernAllocator = KernAllocator;

struct KernAllocator;

unsafe impl core::alloc::GlobalAlloc for KernAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let count = ((layout.pad_to_align().size() + 0xFFF) / 0x1000) as u64;
        let pmm = (*super::state::SYS_STATE.get()).pmm.as_ref().unwrap();
        let ptr = pmm
            .lock()
            .alloc(count)
            .map_or(core::ptr::null_mut(), |ptr| {
                ptr.add(amd64::paging::PHYS_VIRT_OFFSET as _)
            });
        if !ptr.is_null() {
            ptr.write_bytes(0, (count * 0x1000) as _);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.alloc(layout) // Memory is already zeroed by the allocator by default
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let pmm = (*super::state::SYS_STATE.get()).pmm.as_ref().unwrap();
        pmm.lock().free(
            ptr.sub(amd64::paging::PHYS_VIRT_OFFSET as _),
            ((layout.pad_to_align().size() + 0xFFF) / 0x1000) as _,
        );
    }
}

#[alloc_error_handler]
pub fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {layout:#X?}");
}
