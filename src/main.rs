#![no_std]
#![no_main]
#![feature(asm)]
#![warn(unused_extern_crates)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

mod system;

use core::fmt::Write;
use spin::Mutex;

unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

struct SerialWriter(u16);

impl SerialWriter {
    pub const fn new(port: u16) -> Self {
        Self { 0: port }
    }
}

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            unsafe { outb(self.0, c as u8) };
        }
        Ok(())
    }
}

static SERIAL: Mutex<SerialWriter> = Mutex::new(SerialWriter::new(0x3F8));

static _KERNEL_MAIN_CHECK: kaboom::EntryPoint = kernel_main;

#[no_mangle]
pub extern "sysv64" fn kernel_main(explosion: &'static kaboom::ExplosionResult) -> ! {
    let mut serial = SERIAL.lock();

    writeln!(serial, "Fuse ignition begun.").unwrap();
    writeln!(serial, "Bootloader data: {:X?}", explosion).unwrap();

    assert_eq!(explosion.revision, kaboom::CURRENT_REVISION);

    writeln!(serial, "Fuse initialization complete.").unwrap();

    loop {
        unsafe { asm!("hlt") }
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { SERIAL.force_unlock() }
    let mut serial = SERIAL.lock();

    if let Some(loc) = info.location() {
        write!(
            serial,
            "Panic in {} at ({}, {}): ",
            loc.file(),
            loc.line(),
            loc.column()
        )
        .unwrap();
        if let Some(args) = info.message() {
            if let Some(s) = args.as_str() {
                write!(serial, "{}.", s).unwrap();
            } else {
                write!(serial, "{:#X?}", args).unwrap();
            }
        } else {
            write!(serial, "No message provided.").unwrap();
        }
    } else {
        write!(serial, "Panic: {:#X?}", info.payload()).unwrap();
    }

    loop {
        unsafe { asm!("hlt") };
    }
}
