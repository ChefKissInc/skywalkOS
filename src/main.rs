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
use core::arch::asm;

use log::{debug, info};

use crate::{
    driver::{
        acpi::{apic::LocalAPIC, ACPIPlatform},
        keyboard::ps2::PS2Ctl,
    },
    sys::gdt::{PrivilegeLevel, SegmentSelector},
};

mod driver;
mod sys;
mod terminal_loop;
mod utils;

#[no_mangle]
extern "sysv64" fn kernel_main(boot_info: &'static kaboom::BootInfo) -> ! {
    unwinding::panic::catch_unwind(move || {
        sys::io::serial::SERIAL.lock().init();

        log::set_logger(&utils::logger::LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();

        assert_eq!(boot_info.revision, kaboom::CURRENT_REVISION);
        info!("Copyright VisualDevelopment 2021.");

        unsafe {
            debug!("Initialising the GDT.");
            sys::gdt::GDTR.load(
                SegmentSelector::new(1, PrivilegeLevel::Hypervisor),
                SegmentSelector::new(2, PrivilegeLevel::Hypervisor),
            );
            debug!("Initialising the IDT.");
            driver::intrs::idt::IDTR.load();
            debug!("Initialising the exception handlers.");
            driver::intrs::exc::init();
        }

        utils::tags::parse(boot_info.tags);

        debug!("Initializing paging");

        let state = unsafe { &mut *sys::state::SYS_STATE.get() };

        let pml4 = Box::leak(Box::new(sys::vmm::PageTableLvl4::new()));
        unsafe { pml4.init() }
        let pml4 = state.pml4.write(pml4);

        if let Some(terminal) = &mut state.terminal {
            terminal.map_fb();
        }
        info!("Fuse has been ignited!");

        let acpi = unsafe { state.acpi.assume_init_mut() };

        debug!("ACPI version {}", acpi.version);

        let hpet = acpi
            .find("HPET")
            .map(|v| unsafe {
                let addr = v as *const _ as usize;
                pml4.map_mmio(
                    addr,
                    addr - amd64::paging::PHYS_VIRT_OFFSET,
                    1,
                    amd64::paging::PageTableEntry::new()
                        .with_present(true)
                        .with_writable(true),
                );
                driver::timer::hpet::HighPrecisionEventTimer::new(v)
            })
            .unwrap();

        state.madt.write(driver::acpi::madt::MADTData::new(
            acpi.find("APIC").unwrap(),
        ));
        let addr = driver::acpi::apic::get_set_lapic_addr();
        let virt_addr = addr as usize + amd64::paging::PHYS_VIRT_OFFSET;
        unsafe {
            pml4.map_mmio(
                virt_addr,
                addr as usize,
                1,
                amd64::paging::PageTableEntry::new()
                    .with_present(true)
                    .with_writable(true),
            );
        }
        debug!("LAPIC address: {:#X?}", addr);
        state.lapic.write(LocalAPIC::new(virt_addr)).enable();
        let pmm = unsafe { state.pmm.assume_init_ref() };
        info!(
            "Used memory: {}MB out of {}MB",
            (pmm.total_pages - pmm.free_pages) * 4096 / 1000 / 1000,
            pmm.total_pages * 4096 / 1000 / 1000
        );

        unsafe { asm!("sti") }

        state
            .scheduler
            .write(spin::Mutex::new(sys::proc::sched::Scheduler::new(&hpet)));
        sys::proc::sched::Scheduler::start();

        loop {
            unsafe { asm!("hlt") }
        }
    })
    .unwrap()
}
