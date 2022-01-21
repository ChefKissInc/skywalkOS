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
    naked_functions,
    const_mut_refs
)]

extern crate alloc;

use alloc::boxed::Box;
use core::arch::asm;

use font8x8::UnicodeFonts;
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

    log::set_logger(&utils::logger::SERIAL_LOGGER)
        .map(|()| {
            log::set_max_level(if cfg!(debug_assertions) {
                log::LevelFilter::Trace
            } else {
                log::LevelFilter::Info
            })
        })
        .unwrap();

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

    let fb = utils::parse_tags(explosion.tags);

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

    info!("{:#X?}", fb);

    // Display test
    if let Some(fb) = fb {
        unsafe {
            pml4.map_huge_pages(
                fb.base as usize,
                fb.base as usize - amd64::paging::PHYS_VIRT_OFFSET,
                (fb.width * fb.pitch + 0x20_0000 - 1) / 0x20_0000,
                amd64::paging::PageTableEntry::new()
                    .with_writable(true)
                    .with_present(true),
            );
        }
        fb.clear(vesa::pixel::Colour::new(0x00, 0x00, 0x00, 0x00).to_u32(fb.bitmask))
            .unwrap();

        let mut x = fb.width as usize / 2 - 4 * 8;
        for c in "Firework".chars() {
            let mut y = fb.height as usize / 2 - 4;
            for x_bit in &font8x8::BASIC_FONTS.get(c).unwrap() {
                for bit in 0..8 {
                    match *x_bit & (1 << bit) {
                        0 => {}
                        _ => {
                            fb.draw_pixel(
                                x + bit,
                                y,
                                vesa::pixel::Colour::new(0xFF, 0xFF, 0xFF, 0xFF).to_u32(fb.bitmask),
                            )
                            .unwrap();
                        }
                    }
                }
                y += 1;
            }
            x += 8;
        }
    }

    // Test interrupt handler
    info!("Testing the IDT; the below is intentional!");
    unsafe {
        asm!("div {:x}", in(reg) 0);
    }

    loop {
        unsafe { asm!("hlt") }
    }
}
