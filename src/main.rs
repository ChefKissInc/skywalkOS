/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, unused_extern_crates, rust_2021_compatibility)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(const_raw_ptr_deref)]
#![feature(const_size_of_val)]
#![feature(naked_functions)]
#![feature(asm_sym)]
#![feature(asm_const)]

extern crate alloc;

use alloc::boxed::Box;

use amd64::paging::{KERNEL_VIRT_OFFSET, PML4};
use log::{debug, info};

mod sys;
mod utils;

#[no_mangle]
pub extern "sysv64" fn kernel_main(explosion: &'static kaboom::ExplosionResult) -> ! {
    sys::io::serial::SERIAL.lock().init();
    info!("Copyright VisualDevelopment 2021.");

    if cfg!(debug_assertions) {
        log::set_logger(&utils::logger::SERIAL_LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
    } else {
        log::set_logger(&utils::logger::SERIAL_LOGGER).unwrap();
    }

    unsafe {
        info!("Initialising thine GDT.");
        sys::gdt::GDTR.load(
            amd64::sys::cpu::SegmentSelector::new(1, amd64::sys::cpu::PrivilegeLevel::Hypervisor),
            amd64::sys::cpu::SegmentSelector::new(2, amd64::sys::cpu::PrivilegeLevel::Hypervisor),
        );
        info!("Initialising thine PIC.");
        // PIC initialization. temporary
        amd64::io::port::Port::<u8>::new(0x20).write(0x11);
        amd64::io::port::Port::<u8>::new(0xA0).write(0x11);
        let (master, slave) = (
            amd64::io::port::Port::<u8>::new(0x21),
            amd64::io::port::Port::<u8>::new(0xA1),
        );
        master.write(32);
        master.write(4);
        slave.write(2);
        master.write(1);
        slave.write(1);
        slave.write(0);
        master.write(0);
        info!("Initialising thine IDT.");
        sys::idt::init();

        let pml4 = <amd64::paging::PageTable as PML4>::get();
        info!("Testing PML4: KERNEL_VIRT_OFFSET + 0x20_0000 = {:#X?}", pml4.virt_to_phys(KERNEL_VIRT_OFFSET + 0x20_0000));
    }

    utils::parse_tags(explosion.tags);

    // At this point, memory allocations are now possible
    assert_eq!(explosion.revision, kaboom::CURRENT_REVISION);
    info!("Thoust fuseth hast been igniteth!");

    let test = Box::new(5);
    debug!("test = {:#X?}", test);
    core::mem::drop(test);

    info!("Wowse! We artst sending thoust ourst greatesth welcomes!");

    // Test interrupt handler
    unsafe {
        asm!("div {:x}", in(reg) 0);
    }

    loop {
        unsafe { asm!("hlt") }
    }
}
