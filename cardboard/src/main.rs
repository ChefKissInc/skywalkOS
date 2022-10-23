// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![no_main]
#![deny(
    warnings,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    unused_extern_crates,
    rust_2021_compatibility
)]
#![allow(clippy::module_name_repetitions, clippy::similar_names)]
#![feature(
    asm_const,
    alloc_error_handler,
    const_size_of_val,
    panic_info_message,
    naked_functions,
    const_mut_refs,
    sync_unsafe_cell
)]

extern crate alloc;
#[macro_use]
extern crate log;

use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};

use crate::{
    driver::acpi::{apic::LocalAPIC, ACPIPlatform},
    sys::{pmm::BitmapAllocator, terminal::Terminal},
};

mod driver;
mod sys;
mod utils;

#[no_mangle]
#[allow(clippy::too_many_lines)]
extern "sysv64" fn kernel_main(boot_info: &'static sulphur_dioxide::BootInfo) -> ! {
    unwinding::panic::catch_unwind(move || {
        sys::io::serial::SERIAL.lock().init();

        log::set_logger(&utils::logger::LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();

        assert_eq!(boot_info.revision, sulphur_dioxide::CURRENT_REVISION);

        let state = unsafe { &mut *sys::state::SYS_STATE.get() };
        state.kern_symbols.write(boot_info.kern_symbols);
        state.boot_settings = boot_info.settings;

        info!("Copyright ChefKiss Inc 2021-2022.");

        unsafe {
            sys::gdt::GDTR.load();
            driver::intrs::idt::IDTR.load();
            driver::intrs::exc::init();
        }

        state
            .pmm
            .write(spin::Mutex::new(BitmapAllocator::new(boot_info.memory_map)));
        state.acpi.write(ACPIPlatform::new(boot_info.acpi_rsdp));
        state.modules = Some(boot_info.modules.to_vec());

        if let Some(fb_info) = boot_info.frame_buffer {
            debug!("Got boot display: {:X?}", *fb_info);
            let mut terminal = Terminal::new(unsafe {
                paper_fb::framebuffer::Framebuffer::new(
                    fb_info.base,
                    fb_info.resolution.width,
                    fb_info.resolution.height,
                    fb_info.pitch,
                    paper_fb::pixel::Bitmask {
                        r: fb_info.pixel_bitmask.red,
                        g: fb_info.pixel_bitmask.green,
                        b: fb_info.pixel_bitmask.blue,
                        a: fb_info.pixel_bitmask.alpha,
                    },
                )
            });
            terminal.clear();
            state.terminal = Some(terminal);
        }

        // Switch ownership of symbol data to kernel
        state.kern_symbols.write(
            boot_info
                .kern_symbols
                .iter()
                .map(|v| sulphur_dioxide::kern_sym::KernSymbol {
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
        info!("I am Cardboard.");

        let acpi = unsafe { state.acpi.assume_init_mut() };

        debug!("ACPI v{}", acpi.version);

        let hpet = acpi
            .find("HPET")
            .map(|v| unsafe {
                let addr = v as *const _ as u64;
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
        let virt_addr = addr + amd64::paging::PHYS_VIRT_OFFSET;
        unsafe {
            pml4.map_mmio(
                virt_addr,
                addr,
                1,
                amd64::paging::PageTableEntry::new()
                    .with_present(true)
                    .with_writable(true),
            );
        }
        debug!("LAPIC address is {addr:#X?}");
        state.lapic.write(LocalAPIC::new(virt_addr)).enable();

        let pmm = unsafe { state.pmm.assume_init_ref() };
        let used = {
            let pmm = pmm.lock();
            (pmm.total_pages - pmm.free_pages) * 4096 / 1024 / 1024
        };
        let total = pmm.lock().total_pages * 4096 / 1024 / 1024;
        info!("{}MiB used out of {}MiB.", used, total);

        unsafe { core::arch::asm!("sti") }
        let sched = state
            .scheduler
            .write(spin::Mutex::new(sys::proc::sched::Scheduler::new(&hpet)));
        info!("Starting boot DriverCore extensions");
        for module in state.modules.as_ref().unwrap() {
            if module.name.starts_with("com.ChefKissInc.DriverCore.") {
                info!("    Spawning boot DriverCore extension {:#X?}", module.name);
                sched.get_mut().spawn_proc(module.data);
            }
        }
        info!("Done with boot DriverCore extensions.");
        info!("Kernel out.");
        sys::proc::sched::Scheduler::start();

        loop {
            unsafe { core::arch::asm!("hlt") }
        }
    })
    .unwrap()
}
