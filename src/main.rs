#![no_std]
#![no_main]
#![feature(asm)]
#![feature(allocator_api)]
#![warn(unused_extern_crates)]

use core::panic::PanicInfo;

pub const PHYS_VIRT_OFFSET: u64 = 0xFFFFFFFF80000000;
pub const KERNEL_VIRT_OFFSET: u64 = 0xFFFFFFFF80000000;

#[no_mangle]
pub fn kernel_main() -> ! {
    let vga_buffer = (0xB8000 as u64 + PHYS_VIRT_OFFSET) as *mut u8;

    for (i, &byte) in b"Hello World!".iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
