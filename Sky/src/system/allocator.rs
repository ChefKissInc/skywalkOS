// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use amd64::paging::PAGE_SIZE;

#[global_allocator]
static GLOBAL_ALLOCATOR: KernAllocator = KernAllocator;

struct KernAllocator;

unsafe impl core::alloc::GlobalAlloc for KernAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let count = layout.pad_to_align().size().div_ceil(PAGE_SIZE as _) as u64;
        let pmm = (*super::state::SYS_STATE.get()).pmm.as_ref().unwrap();

        let ptr = pmm.lock().alloc(count);
        if ptr.is_null() {
            ptr
        } else {
            let ptr = ptr.map_addr(|v| v.saturating_add(amd64::paging::PHYS_VIRT_OFFSET as _));
            ptr.write_bytes(0, (count * PAGE_SIZE) as _);
            ptr
        }
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.alloc(layout) // Memory is already zeroed by default
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let pmm = (*super::state::SYS_STATE.get()).pmm.as_ref().unwrap();
        pmm.lock().free(
            ptr.map_addr(|v| v - amd64::paging::PHYS_VIRT_OFFSET as usize),
            layout.pad_to_align().size().div_ceil(PAGE_SIZE as _) as _,
        );
    }
}

#[alloc_error_handler]
pub fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {layout:#X?}");
}
