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
    const_mut_refs
)]

extern crate alloc;

use alloc::{boxed::Box, string::String};
use core::{arch::asm, fmt::Write};

use amd64::sys::{
    apic::LocalApic,
    cpu::{PrivilegeLevel, SegmentSelector},
};
use log::{debug, error, info};

use crate::driver::{
    acpi::{apic::ApicHelper, Acpi},
    pci::{PciAddress, PciConfigOffset, PciDevice, PciIoAccessSize},
    ps2::{KeyEvent, PS2Ctl},
};

mod driver;
mod sys;
mod utils;

fn real_main(explosion: &'static kaboom::Explosion) -> ! {
    sys::io::serial::SERIAL.lock().init();

    log::set_logger(&utils::logger::LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();

    assert_eq!(explosion.revision, kaboom::CURRENT_REVISION);
    info!("Copyright VisualDevelopment 2021.");

    unsafe {
        debug!("Initialising the GDT.");
        sys::gdt::GDTR.load(
            SegmentSelector::new(1, PrivilegeLevel::Hypervisor),
            SegmentSelector::new(2, PrivilegeLevel::Hypervisor),
        );
        debug!("Initialising the IDT.");
        sys::idt::init();
        debug!("Initialising the exception handlers.");
        sys::exc::init();
    }

    utils::tags::parse(explosion.tags);

    debug!("Initializing paging");

    let pml4 = unsafe { &mut *sys::state::SYS_STATE.pml4.get() };
    pml4.call_once(|| Box::leak(Box::new(sys::vmm::Pml4::new())));
    unsafe { pml4.get_mut().unwrap().init() }

    if let Some(terminal) = unsafe { (&mut *sys::state::SYS_STATE.terminal.get()).get_mut() } {
        terminal.map_fb();
    }
    info!("Fuse has been ignited!");

    let acpi = unsafe { (&mut *sys::state::SYS_STATE.acpi.get()).get_mut().unwrap() };

    debug!("ACPI version {}", acpi.version);

    unsafe {
        (&*sys::state::SYS_STATE.madt.get())
            .call_once(|| driver::acpi::madt::Madt::new(acpi.find("APIC").unwrap()));
        let addr = driver::acpi::apic::get_final_lapic_addr();
        debug!("LAPIC address: {:?}", addr);
        driver::acpi::apic::set_lapic_addr(addr);
        (&*sys::state::SYS_STATE.lapic.get())
            .call_once(|| LocalApic::new(addr as usize + amd64::paging::PHYS_VIRT_OFFSET))
            .enable();
        asm!("sti");
    }

    let pci = driver::pci::Pci::new();
    let mut ac97 = pci
        .find(move |dev| unsafe {
            dev.cfg_read(
                driver::pci::PciConfigOffset::ClassCode,
                PciIoAccessSize::Word,
            ) == 0x0401
        })
        .map(driver::ac97::Ac97::new);

    if let Some(terminal) = unsafe { (&mut *sys::state::SYS_STATE.terminal.get()).get_mut() } {
        writeln!(terminal, "We welcome you to Firework").unwrap();
        writeln!(terminal, "I am the Fuse debug terminal").unwrap();
        writeln!(terminal, "Type 'help' to see the available commands.").unwrap();

        let mut ps2ctrl = PS2Ctl::new();
        ps2ctrl.init();
        'menu: loop {
            write!(terminal, "\nFirework# ").unwrap();
            let mut cmd = String::new();
            loop {
                match ps2ctrl.wait_for_key() {
                    Ok(KeyEvent::Pressed(c)) => {
                        terminal.write_char(c).unwrap();
                        match c {
                            '\n' => {
                                match cmd.as_str() {
                                    "help" => {
                                        info!(
                                            r#"Fuse debug terminal
Available commands:
    clear      <= Clear terminal
    greeting   <= Very epic example command
    acpidump   <= Dump ACPI information
    pcidump    <= Dump PCI devices
    audiotest  <= Play test sound through AC97
    restart    <= Restart machine by resetting CPU
    help       <= Display this"#
                                        );
                                    }
                                    "clear" => terminal.clear(),
                                    "greeting" => info!("Greetings, User."),
                                    "acpidump" => {
                                        info!("ACPI version {}", acpi.version);
                                        for table in &acpi.tables {
                                            info!("Table '{}': {:#X?}", table.0, table.1);
                                        }
                                    }
                                    "pcidump" => {
                                        for bus in 0..=255 {
                                            for slot in 0..32 {
                                                for func in 0..8 {
                                                    let device = PciDevice::new(
                                                        PciAddress {
                                                            bus,
                                                            slot,
                                                            func,
                                                            ..Default::default()
                                                        },
                                                        pci.io.as_ref(),
                                                    );
                                                    unsafe {
                                                        let vendor_id = device.cfg_read(
                                                            PciConfigOffset::VendorId,
                                                            PciIoAccessSize::Word,
                                                        );
                                                        if vendor_id != 0xFFFF {
                                                            info!(
                                                                "PCI Device at {}:{}:{} has \
                                                                 vendor ID {:#06X} and device ID \
                                                                 {:#06X}, class code {:#06X}",
                                                                bus,
                                                                slot,
                                                                func,
                                                                vendor_id,
                                                                device.cfg_read(
                                                                    PciConfigOffset::DeviceId,
                                                                    PciIoAccessSize::Word,
                                                                ),
                                                                device.cfg_read(
                                                                    PciConfigOffset::ClassCode,
                                                                    PciIoAccessSize::Word,
                                                                ),
                                                            )
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    "audiotest" => {
                                        if let Some(ac97) = &mut ac97 {
                                            let modules =
                                                unsafe { &*sys::state::SYS_STATE.modules.get() }
                                                    .get()
                                                    .unwrap();
                                            if let Some(module) =
                                                modules.iter().find(|v| v.name == "testaudio")
                                            {
                                                info!("Starting playback of test audio");
                                                ac97.play_audio(module.data)
                                            } else {
                                                error!(
                                                    "Failure to find 'testaudio' boot loader \
                                                     module"
                                                );
                                            }
                                        } else {
                                            error!("No sound device available!");
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
                    Ok(KeyEvent::BackSpace) => {
                        if !cmd.is_empty() {
                            cmd.pop();
                            terminal.backspace()
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    loop {
        unsafe { asm!("hlt") }
    }
}

#[no_mangle]
extern "sysv64" fn kernel_main(explosion: &'static kaboom::ExplosionResult) -> ! {
    unwinding::panic::catch_unwind(|| real_main(explosion)).unwrap()
}
