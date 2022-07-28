//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

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

use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};
use core::arch::asm;

use log::{debug, info};

use crate::{
    driver::acpi::{apic::LocalAPIC, ACPIPlatform},
    sys::{
        gdt::{PrivilegeLevel, SegmentSelector},
        pmm::BitmapAllocator,
        terminal::Terminal,
    },
};

mod driver;
mod sys;
mod terminal_loop;
mod utils;

#[no_mangle]
extern "sysv64" fn kernel_main(boot_info: &'static sulfur_dioxide::BootInfo) -> ! {
    unwinding::panic::catch_unwind(move || {
        sys::io::serial::SERIAL.lock().init();

        log::set_logger(&utils::logger::LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();

        let state = unsafe { &mut *sys::state::SYS_STATE.get() };

        state.kern_symbols.write(boot_info.kern_symbols);

        assert_eq!(boot_info.revision, sulfur_dioxide::CURRENT_REVISION);
        info!("Copyright ChefKiss Inc 2021.");

        state.boot_settings = boot_info.settings;
        debug!("Got boot settings: {:X?}", state.boot_settings);

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

        debug!("Got memory map: {:X?}", boot_info.memory_map);
        state
            .pmm
            .write(spin::Mutex::new(BitmapAllocator::new(boot_info.memory_map)));

        debug!("Got ACPI RSDP: {:X?}", boot_info.acpi_rsdp);
        state.acpi.write(ACPIPlatform::new(boot_info.acpi_rsdp));

        debug!("Got modules: {:#X?}", boot_info.modules);
        state.modules = Some(boot_info.modules.to_vec());

        if let Some(fb_info) = boot_info.frame_buffer {
            debug!("Got boot display: {:X?}", *fb_info);
            let mut terminal = Terminal::new(paper_fb::framebuffer::Framebuffer::new(
                fb_info.base,
                fb_info.resolution.width as usize,
                fb_info.resolution.height as usize,
                paper_fb::pixel::Bitmask {
                    r: fb_info.pixel_bitmask.red,
                    g: fb_info.pixel_bitmask.green,
                    b: fb_info.pixel_bitmask.blue,
                    a: fb_info.pixel_bitmask.alpha,
                },
                fb_info.pitch,
            ));
            terminal.clear();
            state.terminal = Some(terminal);
        }

        // Switch ownership of symbol data to kernel
        state.kern_symbols.write(
            boot_info
                .kern_symbols
                .iter()
                .map(|v| sulfur_dioxide::symbol::KernSymbol {
                    name: Box::leak(v.name.to_owned().into_boxed_str()),
                    ..*v
                })
                .collect::<Vec<_>>()
                .leak(),
        );

        debug!("Initialising paging");

        let pml4 = Box::leak(Box::new(sys::vmm::PageTableLvl4::new()));
        unsafe { pml4.init() }
        let pml4 = state.pml4.write(pml4);

        if let Some(terminal) = &mut state.terminal {
            terminal.map_fb();
        }
        info!("Cardboard has been synthesised!");

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
        let used = {
            let pmm = pmm.lock();
            (pmm.total_pages - pmm.free_pages) * 4096 / 1024 / 1024
        };
        let total = pmm.lock().total_pages * 4096 / 1024 / 1024;
        info!("Used memory: {}MiB out of {}MiB", used, total);

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
