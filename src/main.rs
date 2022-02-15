/*
 * Copyright (c) VisualDevelopment 2021-2022.
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

use alloc::{boxed::Box, string::String};
use core::{arch::asm, fmt::Write};

use log::info;

use crate::driver::ps2::{KeyEvent, PS2Ctl};

mod driver;
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

    utils::parse_tags(explosion.tags);

    // At this point, memory allocations are now possible
    info!("Initializing paging");
    unsafe {
        crate::sys::state::SYS_STATE
            .pml4
            .get()
            .as_mut()
            .unwrap()
            .call_once(|| Box::leak(Box::new(sys::vmm::Pml4::new())));
        crate::sys::state::SYS_STATE
            .pml4
            .get()
            .as_mut()
            .unwrap()
            .get_mut()
            .unwrap()
            .init();
    }
    info!("Thoust fuseth hast been igniteth!");

    info!("Wowse! We artst sending thoust ourst greatesth welcomes!");

    // Terminal
    if let Some(terminal) = unsafe { sys::state::SYS_STATE.terminal.get().as_mut() }
        .unwrap()
        .get_mut()
    {
        terminal.map_fb();
        terminal.clear();

        writeln!(terminal, "We welcome thoust to Firework").unwrap();
        writeln!(
            terminal,
            "I arst thine Firework kerneleth debugging terminalth (FKDBGT) (patent pending)"
        )
        .unwrap();
        writeln!(
            terminal,
            "Typeth thine 'help' commandst to seeth available commandst."
        )
        .unwrap();
        let mut ps2ctrl = PS2Ctl::new();
        ps2ctrl.init();
        'menu: loop {
            write!(terminal, "\nFirework# ").unwrap();
            let mut cmd = String::new();
            loop {
                if let Ok(KeyEvent::Pressed(c)) = ps2ctrl.wait_for_key() {
                    terminal.write_char(c).unwrap();
                    match c {
                        '\n' => {
                            match cmd.as_str() {
                                "help" => {
                                    writeln!(
                                        terminal,
                                        "Fireworkst Kerneleth debugging terminalth (FKDBGT)"
                                    )
                                    .unwrap();
                                    writeln!(terminal, "Available commandst:").unwrap();
                                    writeln!(
                                        terminal,
                                        "    greeting <= Very epicst exampleth commandeth"
                                    )
                                    .unwrap();
                                    writeln!(
                                        terminal,
                                        "    restart  <= Restarteth thoust machineth by resetting \
                                         thine CPU"
                                    )
                                    .unwrap();
                                    writeln!(
                                        terminal,
                                        "    help     <= Displayeth thine currenst messageth"
                                    )
                                    .unwrap();
                                }
                                "greeting" => {
                                    writeln!(
                                        terminal,
                                        "Greetings, thoust veryth estimeedeth consumerst"
                                    )
                                    .unwrap()
                                }
                                "restart" => ps2ctrl.reset_cpu(),
                                _ => writeln!(terminal, "Unknownst commandeth").unwrap(),
                            }

                            continue 'menu;
                        }
                        _ => cmd.push(c),
                    }
                }
            }
        }
    }

    loop {
        unsafe { asm!("hlt") }
    }
}
