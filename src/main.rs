#![no_std]
#![no_main]
#![feature(asm)]
#![warn(unused_extern_crates)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::fmt::Write;

static STACK: [u8; 4096] = [0; 4096];

#[link_section = ".kaboom"]
#[used]
static EXPLOSION_INFO: kaboom::ExplosionInfo = kaboom::ExplosionInfo::new(&STACK[4095] as *const _);

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
fn kernel_main(explosion: &'static kaboom::ExplosionResult) -> ! {
    unsafe {
        writeln!(&mut SERIAL, "Fuse ignition begun.").unwrap();
        assert_eq!(explosion.rev, EXPLOSION_INFO.rev);
        writeln!(&mut SERIAL, "Bootloader data: {:X?}", explosion).unwrap();
        for tag in explosion.tags {
            match tag.uuid() {
                kaboom::tags::MemoryMap::UUID => writeln!(&mut SERIAL, "Memory map: {:X?}", &*(*tag as *const _ as *const kaboom::tags::MemoryMap)).unwrap(),
                kaboom::tags::FrameBufferInfo::UUID => {
                    let frame_buffer = &*(*tag as *const _ as *const kaboom::tags::FrameBufferInfo);
                    writeln!(&mut SERIAL, "Frame buffer: {:X?}", frame_buffer).unwrap();
                    let size = frame_buffer.resolution.width * frame_buffer.resolution.height;
                    frame_buffer.base.write_bytes(0, size.try_into().unwrap());
                }
                _ => writeln!(&mut SERIAL, "Unknown tag: {:X?}", tag).unwrap(),
            }
        }

        writeln!(&mut SERIAL, "I love Rust").unwrap();
    }

    loop {
        unsafe { asm!("hlt") };
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        unsafe { write!(&mut SERIAL, "Panic in {} at ({}, {}): ", loc.file(), loc.line(), loc.column()).unwrap() };
        if let Some(msg) = info.message() {
            unsafe { write!(&mut SERIAL, "{}", msg).unwrap() };
        } else {
            unsafe { write!(&mut SERIAL, "No message provided").unwrap() };
        }
    } else {
        unsafe { write!(&mut SERIAL, "Panic: {:#X?}", info.payload()).unwrap() };
    }

    loop {
        unsafe { asm!("hlt") };
    }
}

#[alloc_error_handler]
fn out_of_memory(layout: core::alloc::Layout) -> ! {
    panic!("Ran out of memory while trying to allocate {:#?}", layout);
}
