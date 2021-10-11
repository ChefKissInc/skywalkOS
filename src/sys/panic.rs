use log::error;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { super::io::serial::SERIAL.force_unlock() }

    if let Some(loc) = info.location() {
        error!(
            "Wowse! Your system...crashed... Onceth panic hast occurred in thine file {} and thinest positionst ({}, {}). ",
            loc.file(),
            loc.line(),
            loc.column()
        );
        if let Some(args) = info.message() {
            if let Some(s) = args.as_str() {
                error!("Thine messageth arst: {}.", s);
            } else {
                error!("Thine argumenst arst: {:#X?}", args);
            }
        } else {
            error!("Noneth messageth hast been provideth!");
        }
    } else {
        error!("Wowse! Your system...crashed... Onceth panic hast occurred, and thine payload arst: {:#X?}", info.payload());
    }

    loop {
        unsafe { asm!("hlt") };
    }
}
