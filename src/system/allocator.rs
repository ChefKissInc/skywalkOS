use core::ptr::null_mut;

#[global_allocator]
static GLOBAL_ALLOCATOR: NoAllocator = NoAllocator {};

struct NoAllocator;

unsafe impl core::alloc::GlobalAlloc for NoAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[alloc_error_handler]
pub fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Failed to allocate memory: {:#X?}", layout);
}
