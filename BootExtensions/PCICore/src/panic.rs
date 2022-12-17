use kernel::SystemCall;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("{info}");
    unsafe { SystemCall::exit().unwrap() };
    unreachable!();
}
