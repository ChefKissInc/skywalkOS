// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};

use fireworkkit::osdtentry::OSDTENTRY_NAME_KEY;
use hashbrown::HashMap;

use crate::{
    acpi::{tables::rsdp::RootSystemDescPtr, ACPIState},
    system::{pmm::BitmapAllocator, state::OSDTEntry},
    utils::incr_id::IncrementalIDGen,
};

pub mod bitmap;
pub mod incr_id;
pub mod logger;

#[macro_export]
macro_rules! hlt_loop {
    () => {
        loop {
            unsafe { core::arch::asm!("hlt", options(nostack, preserves_flags)) }
        }
    };
}

#[macro_export]
macro_rules! sti {
    () => {
        unsafe { core::arch::asm!("sti", options(nostack, preserves_flags)) }
    };
}

#[macro_export]
macro_rules! cli {
    () => {
        unsafe { core::arch::asm!("cli", options(nostack, preserves_flags)) }
    };
}

pub fn init_core(boot_info: &sulphur_dioxide::BootInfo) {
    let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    state.kern_symbols = Some(boot_info.kern_symbols);
    state.verbose = boot_info.verbose;
    state.serial_enabled = boot_info.serial_enabled;

    unsafe {
        crate::system::gdt::GDTR.load();
        crate::intrs::idt::IDTR.load();
        crate::intrs::init_intr_quirks();
        crate::system::exc::init();
    }

    state.pmm = Some(BitmapAllocator::new(boot_info.memory_map).into());

    // Switch ownership of symbol data to kernel
    state.kern_symbols = Some(
        boot_info
            .kern_symbols
            .iter()
            .map(|v| sulphur_dioxide::KernSymbol {
                name: Box::leak(v.name.to_owned().into_boxed_str()),
                ..*v
            })
            .collect::<Vec<_>>()
            .leak(),
    );

    let mut root = OSDTEntry {
        properties: HashMap::from([
            (OSDTENTRY_NAME_KEY.into(), "Root".into()),
            ("Version".into(), "0.0.1".into()),
        ]),
        ..Default::default()
    };
    let mut dt_id_gen = IncrementalIDGen::new();
    let product = OSDTEntry {
        id: dt_id_gen.next(),
        parent: Some(root.id.into()),
        properties: HashMap::from([
            (OSDTENTRY_NAME_KEY.into(), "Product".into()),
            ("CPUType".into(), "x86_64".into()),
            ("Vendor".into(), "Generic".into()),
        ]),
        ..Default::default()
    };
    root.children.push(product.id.into());

    state.dt_index =
        Some(HashMap::from([(root.id, root.into()), (product.id, product.into())]).into());

    state.dt_id_gen = Some(dt_id_gen.into());

    unsafe {
        state.acpi = Some(ACPIState::new(
            &*boot_info.acpi_rsdp.cast::<RootSystemDescPtr>(),
        ));
    }
}

pub fn init_paging(state: &mut crate::system::state::SystemState) {
    debug!("Initialising paging");

    let pml4 = Box::leak(Box::new(crate::system::vmm::PageTableLvl4::new()));
    unsafe { pml4.init() }
    state.pml4 = Some(pml4);

    if let Some(terminal) = &mut state.terminal {
        terminal.map_fb();
    }
}
