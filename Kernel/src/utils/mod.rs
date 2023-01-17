// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};

use hashbrown::HashMap;

use crate::{
    acpi::ACPIPlatform,
    system::{pmm::BitmapAllocator, state::BCRegistryEntry},
    utils::incr_id::IncrementalIDGen,
};

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
    let state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
    state.kern_symbols.call_once(|| boot_info.kern_symbols);
    state.boot_settings = boot_info.settings;

    unsafe {
        crate::system::gdt::GDTR.load();
        crate::intrs::idt::IDTR.load();
        crate::intrs::init_intr_quirks();
        crate::system::exc::init();
    }

    state
        .pmm
        .call_once(|| spin::Mutex::new(BitmapAllocator::new(boot_info.memory_map)));

    // Switch ownership of symbol data to kernel
    state.kern_symbols.call_once(|| {
        boot_info
            .kern_symbols
            .iter()
            .map(|v| sulphur_dioxide::KernSymbol {
                name: Box::leak(v.name.to_owned().into_boxed_str()),
                ..*v
            })
            .collect::<Vec<_>>()
            .leak()
    });

    let mut root = BCRegistryEntry {
        id: 0,
        properties: HashMap::from([
            ("Name".to_owned(), "Root".into()),
            ("Version".to_owned(), "0.0.1".into()),
        ]),
        ..Default::default()
    };
    let mut registry_tree_id_gen = IncrementalIDGen::new();
    let product = BCRegistryEntry {
        id: registry_tree_id_gen.next(),
        parent: Some(root.id),
        properties: HashMap::from([
            ("Name".to_owned(), "Product".into()),
            ("CPUType".to_owned(), "x86_64".into()),
            ("Vendor".to_owned(), "Generic".into()),
        ]),
        ..Default::default()
    };
    root.children.push(product.id);

    state
        .registry_tree_index
        .call_once(|| spin::Mutex::new(HashMap::from([(root.id, root), (product.id, product)])));

    state
        .registry_tree_id_gen
        .call_once(|| spin::Mutex::new(registry_tree_id_gen));

    state
        .acpi
        .call_once(|| ACPIPlatform::new(boot_info.acpi_rsdp));

    state.dc_cache = Some(boot_info.dc_cache.to_vec());
}

pub fn init_paging(state: &mut crate::system::state::SystemState) {
    debug!("Initialising paging");

    let pml4 = Box::leak(Box::new(crate::system::vmm::PageTableLvl4::new()));
    unsafe { pml4.init() }

    state.pml4.call_once(|| pml4);

    if let Some(terminal) = &mut state.terminal {
        terminal.map_fb();
    }
}
