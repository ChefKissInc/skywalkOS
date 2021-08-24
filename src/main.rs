#![no_std]
#![no_main]
#![feature(asm)]
#![warn(unused_extern_crates)]

use core::fmt::Write;
use core::panic::PanicInfo;
use kaboom;

static STACK: [u8; 4096] = [0; 4096];

#[link_section = ".kaboom"]
#[used]
static EXPLOSION_INFO: kaboom::ExplosionInfo = kaboom::ExplosionInfo::new(&STACK[4095] as *const u8);

pub const PHYS_VIRT_OFFSET: u64 = 0xFFFF800000000000;
pub const KERNEL_VIRT_OFFSET: u64 = 0xFFFFFFFF80000000;

unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

struct Serial;

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            unsafe { outb(0x3F8, c as u8) };
        }
        Ok(())
    }
}

static mut SERIAL: Serial = Serial {};

#[no_mangle]
pub fn kernel_main(explosion: &kaboom::ExplosionResult) -> ! {
    unsafe {
        writeln!(&mut SERIAL, "Hello!").unwrap();
        writeln!(&mut SERIAL, "{:#X?}", explosion).unwrap();

        explosion.framebuffer.base.write_bytes(0, explosion.framebuffer.resolution.0 * explosion.framebuffer.resolution.1);

        writeln!(&mut SERIAL, "I love Rust").unwrap();
    }

    loop {
        unsafe { asm!("hlt") };
    }
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    unsafe {
        write!(&mut SERIAL, "PANIC: {:?}", info).unwrap();
    }
    loop {
        unsafe { asm!("hlt") };
    }
}
