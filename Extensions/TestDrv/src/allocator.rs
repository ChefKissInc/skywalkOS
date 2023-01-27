// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

use iridium_kit::syscall::SystemCall;

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

struct Allocator;

unsafe impl core::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        SystemCall::allocate(layout.size() as u64).unwrap()
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.alloc(layout) // Memory is already zeroed by the allocator
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        SystemCall::free(ptr).unwrap();
    }
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {:#X?}", layout);
}
