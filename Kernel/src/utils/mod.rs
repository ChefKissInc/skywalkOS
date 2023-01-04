// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};

use crate::{driver::acpi::ACPIPlatform, sys::pmm::BitmapAllocator};

pub mod bitmap;
pub mod incr_id;
pub mod logger;

#[macro_export]
macro_rules! hlt_loop {
    () => {
        loop {
            unsafe { core::arch::asm!("hlt") }
        }
    };
}

#[macro_export]
macro_rules! sti {
    () => {
        unsafe { core::arch::asm!("sti") }
    };
}

#[macro_export]
macro_rules! cli {
    () => {
        unsafe { core::arch::asm!("cli") }
    };
}

pub fn init_core(boot_info: &sulphur_dioxide::BootInfo) {
    let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
    state.kern_symbols.call_once(|| boot_info.kern_symbols);
    state.boot_settings = boot_info.settings;

    unsafe {
        crate::sys::gdt::GDTR.load();
        crate::driver::intrs::idt::IDTR.load();
        crate::driver::intrs::init_intr_quirks();
        crate::driver::intrs::exc::init();
    }

    state
        .pmm
        .call_once(|| spin::Mutex::new(BitmapAllocator::new(boot_info.memory_map)));

    // Switch ownership of symbol data to kernel
    state.kern_symbols.call_once(|| {
        boot_info
            .kern_symbols
            .iter()
            .map(|v| sulphur_dioxide::kern_sym::KernSymbol {
                name: Box::leak(v.name.to_owned().into_boxed_str()),
                ..*v
            })
            .collect::<Vec<_>>()
            .leak()
    });

    debug!(
        "ACPI v{}",
        state
            .acpi
            .call_once(|| ACPIPlatform::new(boot_info.acpi_rsdp))
            .version
    );

    state.modules = Some(boot_info.modules.to_vec());
}

pub fn init_paging(state: &mut crate::sys::state::SystemState) {
    debug!("Initialising paging");

    let pml4 = Box::leak(Box::new(crate::sys::vmm::PageTableLvl4::new()));
    unsafe { pml4.init() }

    state.pml4.call_once(|| pml4);

    if let Some(terminal) = &mut state.terminal {
        terminal.map_fb();
    }
}
