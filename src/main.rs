/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, unused_extern_crates, rust_2021_compatibility)]
#![feature(
    asm_sym,
    asm_const,
    alloc_error_handler,
    allocator_api,
    const_size_of_val,
    panic_info_message,
    naked_functions
)]

extern crate alloc;

use alloc::boxed::Box;
use core::arch::asm;

use log::info;

mod sys;
mod utils;

#[used]
static STACK: [u8; 0x5_0000] = [0; 0x5_0000];

#[link_section = ".kaboom"]
#[used]
static EXPLOSION_FUEL: kaboom::ExplosionFuel =
    kaboom::ExplosionFuel::new(&STACK[0x5_0000 - 1] as *const _);

#[no_mangle]
extern "sysv64" fn kernel_main(explosion: &'static kaboom::ExplosionResult) -> ! {
    sys::io::serial::SERIAL.lock().init();

    if cfg!(debug_assertions) {
        log::set_logger(&utils::logger::SERIAL_LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
    } else {
        log::set_logger(&utils::logger::SERIAL_LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Info))
            .unwrap();
    }

    assert_eq!(explosion.revision, kaboom::CURRENT_REVISION);
    info!("Copyright VisualDevelopment 2021.");

    unsafe {
        info!("Initialising thine GDT.");
        sys::gdt::GDTR.load(
            amd64::sys::cpu::SegmentSelector::new(1, amd64::sys::cpu::PrivilegeLevel::Hypervisor),
            amd64::sys::cpu::SegmentSelector::new(2, amd64::sys::cpu::PrivilegeLevel::Hypervisor),
        );
        info!("Initialising thine IDT.");
        sys::idt::init();
        info!("Initialising thine exceptionst handleth.");
        sys::exceptions::init();
    }

    utils::parse_tags(explosion.tags);

    // At this point, memory allocations are now possible
    info!("Initializing paging");
    let pml4 = Box::leak(Box::new(sys::paging::Pml4::new()));
    unsafe {
        pml4.map_higher_half();
        info!(
            "Testing PML4: KERNEL_VIRT_OFFSET + 0x20_0000 = {:#X?}",
            pml4.virt_to_phys(amd64::paging::KERNEL_VIRT_OFFSET + 0x20_0000)
        );

        pml4.set()
    }
    info!("Thoust fuseth hast been igniteth!");

    info!("Wowse! We artst sending thoust ourst greatesth welcomes!");

    // Test interrupt handler
    info!("Testing the IDT; the below is intentional!");
    unsafe {
        asm!("div {:x}", in(reg) 0);
    }

    loop {
        unsafe { asm!("hlt") }
    }
}
