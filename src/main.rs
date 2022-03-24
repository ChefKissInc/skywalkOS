//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

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
    const_mut_refs,
    derive_default_enum
)]

extern crate alloc;

use alloc::{boxed::Box, string::String};
use core::{arch::asm, fmt::Write};

use log::info;

use crate::driver::{
    acpi::Acpi,
    pci::{PciAddress, PciIoAccessSize},
    ps2::{KeyEvent, PS2Ctl},
};

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
        info!("Initialising the GDT.");
        sys::gdt::GDTR.load(
            amd64::sys::cpu::SegmentSelector::new(1, amd64::sys::cpu::PrivilegeLevel::Hypervisor),
            amd64::sys::cpu::SegmentSelector::new(2, amd64::sys::cpu::PrivilegeLevel::Hypervisor),
        );
        info!("Initialising the IDT.");
        sys::idt::init();
        info!("Initialising the exception handlers.");
        sys::exceptions::init();
    }

    utils::parse_tags(explosion.tags);

    info!("Initializing paging");
    unsafe {
        let pml4 = crate::sys::state::SYS_STATE.pml4.get().as_mut().unwrap();
        pml4.call_once(|| Box::leak(Box::new(sys::vmm::Pml4::new())));
        pml4.get_mut().unwrap().init();
    }
    info!("Fuse has been ignited!");

    let acpi = unsafe { sys::state::SYS_STATE.acpi.get().as_mut() }
        .unwrap()
        .get_mut()
        .unwrap();

    info!("ACPI version {}", acpi.version);

    let _madt = driver::acpi::madt::Madt::new(acpi.find("APIC").unwrap());
    let pci = driver::pci::Pci::new();
    let _ac97 = driver::ac97::Ac97::new(
        pci.find(move |dev| {
            dev.cfg_read(
                driver::pci::PciConfigOffset::ClassCode as _,
                PciIoAccessSize::Word,
            ) == 0x0401
        })
        .unwrap(),
    );

    if let Some(terminal) = unsafe { sys::state::SYS_STATE.terminal.get().as_mut() }
        .unwrap()
        .get_mut()
    {
        terminal.map_fb();
        terminal.clear();

        writeln!(terminal, "We welcome you to Firework").unwrap();
        writeln!(terminal, "I am the Fuse debug terminal").unwrap();
        writeln!(terminal, "Type 'help' to see the available commands.").unwrap();
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
                                    writeln!(terminal, "Fuse debug terminal").unwrap();
                                    writeln!(terminal, "Available commands:").unwrap();
                                    writeln!(
                                        terminal,
                                        "    greeting   <= Very epic example command"
                                    )
                                    .unwrap();
                                    writeln!(terminal, "    acpidump   <= Dump ACPI information")
                                        .unwrap();
                                    writeln!(terminal, "    pcidump    <= Dump PCI devices")
                                        .unwrap();
                                    writeln!(
                                        terminal,
                                        "    restart    <= Restart machine by resetting CPU"
                                    )
                                    .unwrap();
                                    writeln!(terminal, "    help       <= Display this").unwrap();
                                }
                                "greeting" => writeln!(terminal, "Greetings, User.").unwrap(),
                                "acpidump" => {
                                    writeln!(terminal, "ACPI version {}", acpi.version).unwrap();
                                    for table in &acpi.tables {
                                        writeln!(terminal, "Table '{}': {:#X?}", table.0, table.1)
                                            .unwrap()
                                    }
                                }
                                "pcidump" => {
                                    for bus in 0..=255 {
                                        for slot in 0..32 {
                                            for func in 0..8 {
                                                let device = driver::pci::PciDevice::new(
                                                    PciAddress {
                                                        bus,
                                                        slot,
                                                        func,
                                                        ..Default::default()
                                                    },
                                                    pci.io.as_ref(),
                                                );
                                                let vendor_id = device.cfg_read(
                                                    driver::pci::PciConfigOffset::VendorId as _,
                                                    PciIoAccessSize::Word,
                                                );
                                                if vendor_id != 0xFFFF {
                                                    writeln!(
                                                        terminal,
                                                        "PCI Device at {}:{}:{} has vendor ID \
                                                         {:#06X} and device ID {:#06X}, class \
                                                         code {:#06X}",
                                                        bus,
                                                        slot,
                                                        func,
                                                        vendor_id,
                                                        device.cfg_read(
                                                            driver::pci::PciConfigOffset::DeviceId
                                                                as _,
                                                            PciIoAccessSize::Word
                                                        ),
                                                        device.cfg_read(
                                                            driver::pci::PciConfigOffset::ClassCode
                                                                as _,
                                                            PciIoAccessSize::Word
                                                        ),
                                                    )
                                                    .unwrap();
                                                }
                                            }
                                        }
                                    }
                                }
                                "restart" => ps2ctrl.reset_cpu(),
                                _ => writeln!(terminal, "Unknown command").unwrap(),
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
