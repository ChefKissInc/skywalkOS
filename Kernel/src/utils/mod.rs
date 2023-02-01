// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};

use hashbrown::HashMap;

use crate::{
    acpi::{tables::rsdp::RootSystemDescPtr, Acpi},
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

    unsafe {
        crate::system::gdt::GDTR.load();
        crate::intrs::idt::IDTR.load();
        crate::intrs::init_intr_quirks();
        crate::system::exc::init();
    }

    state.pmm = Some(spin::Mutex::new(BitmapAllocator::new(boot_info.memory_map)));

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
        id: 0,
        properties: HashMap::from([
            ("Name".to_owned(), "Root".into()),
            ("Version".to_owned(), "0.0.1".into()),
        ]),
        ..Default::default()
    };
    let mut dt_id_gen = IncrementalIDGen::new();
    let product = OSDTEntry {
        id: dt_id_gen.next(),
        parent: Some(root.id),
        properties: HashMap::from([
            ("Name".to_owned(), "Product".into()),
            ("CPUType".to_owned(), "x86_64".into()),
            ("Vendor".to_owned(), "Generic".into()),
        ]),
        ..Default::default()
    };
    root.children.push(product.id);

    state.dt_index = Some(spin::Mutex::new(HashMap::from([
        (root.id, root),
        (product.id, product),
    ])));

    state.dt_id_gen = Some(spin::Mutex::new(dt_id_gen));

    unsafe { state.acpi = Some(Acpi::new(&*boot_info.acpi_rsdp.cast::<RootSystemDescPtr>())) }

    state.dc_cache = Some(boot_info.dc_cache.to_vec());
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
