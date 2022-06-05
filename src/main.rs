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
    sync_unsafe_cell
)]

extern crate alloc;

use alloc::boxed::Box;
use core::{arch::asm, fmt::Write};

use amd64::{
    cpu::{PrivilegeLevel, SegmentSelector},
    intrs::apic::LocalApic,
};
use log::{debug, info};

use crate::driver::{
    acpi::{apic::ApicHelper, Acpi},
    pci::PciIoAccessSize,
    ps2::PS2Ctl,
};

mod driver;
mod sys;
mod terminal_loop;
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

    let state = unsafe { &mut *sys::state::SYS_STATE.get() };

    let pml4 = Box::leak(Box::new(sys::vmm::Pml4::new()));
    unsafe { pml4.init() }
    state.pml4.write(pml4);

    if let Some(terminal) = &mut state.terminal {
        terminal.map_fb();
    }
    info!("Fuse has been ignited!");

    let acpi = unsafe { state.acpi.assume_init_mut() };

    debug!("ACPI version {}", acpi.version);

    state
        .madt
        .write(driver::acpi::madt::Madt::new(acpi.find("APIC").unwrap()));
    let addr = driver::acpi::apic::get_set_lapic_addr();
    debug!("LAPIC address: {:?}", addr);
    state
        .lapic
        .write(LocalApic::new(
            addr as usize + amd64::paging::PHYS_VIRT_OFFSET,
        ))
        .enable();

    unsafe { asm!("sti") }

    let pci = driver::pci::Pci::new();
    let ac97 = pci
        .find(move |dev| unsafe {
            dev.cfg_read(
                driver::pci::PciConfigOffset::ClassCode,
                PciIoAccessSize::Word,
            ) == 0x0401
        })
        .map(driver::ac97::Ac97::new)
        .map(|v| unsafe { (*driver::ac97::INSTANCE.get()).write(v) });

    if let Some(terminal) = &mut state.terminal {
        writeln!(terminal, "We welcome you to Firework").unwrap();
        writeln!(terminal, "I am the Fuse debug terminal").unwrap();
        writeln!(terminal, "Type 'help' to see the available commands.").unwrap();

        let mut ps2ctl = PS2Ctl::new();
        ps2ctl.init();
        unsafe {
            (*driver::ps2::INSTANCE.get()).write(ps2ctl);
        }

        terminal_loop::terminal_loop(acpi, &pci, terminal, ac97);
    }

    loop {
        unsafe { asm!("hlt") }
    }
}

#[no_mangle]
extern "sysv64" fn kernel_main(explosion: &'static kaboom::Explosion) -> ! {
    unwinding::panic::catch_unwind(|| real_main(explosion)).unwrap()
}
