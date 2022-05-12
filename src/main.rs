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

use alloc::boxed::Box;
use core::{arch::asm, fmt::Write};

use amd64::sys::{
    apic::LocalApic,
    cpu::{PrivilegeLevel, SegmentSelector},
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

        terminal_loop::terminal_loop(acpi, &pci, terminal, &mut ps2ctrl, &mut ac97);
    }

    loop {
        unsafe { asm!("hlt") }
    }
}

#[no_mangle]
extern "sysv64" fn kernel_main(explosion: &'static kaboom::Explosion) -> ! {
    unwinding::panic::catch_unwind(|| real_main(explosion)).unwrap()
}
