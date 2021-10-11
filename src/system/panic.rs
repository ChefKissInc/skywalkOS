use core::fmt::Write;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { super::io::serial::SERIAL.force_unlock() }
    let mut serial = super::io::serial::SERIAL.lock();

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
