// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use crate::syscall::SystemCall;

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

struct Allocator;

unsafe impl core::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut ptr: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::Allocate as u64,
            in("rsi") layout.pad_to_align().size() as u64,
            out("rax") ptr,
            options(nostack, preserves_flags),
        );
        ptr as *mut u8
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.alloc(layout) // Memory is already zeroed by the allocator
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::Free as u64,
            in("rsi") ptr as u64,
            options(nostack, preserves_flags),
        );
    }
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {:#X?}", layout);
}
