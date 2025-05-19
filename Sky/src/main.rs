// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![no_std]
#![no_main]
#![deny(warnings, clippy::nursery, unused_extern_crates)]
#![feature(alloc_error_handler, sync_unsafe_cell)]

use acpi::{tables::rsdp::RootSystemDescPtr, ACPIState};
use alloc::boxed::Box;
use hashbrown::HashMap;
use incr_id::IncrementalIDGen;
use skykit::{osdtentry::OSDTENTRY_NAME_KEY, SKExtensions};
use system::{pmm::BitmapAllocator, state::OSDTEntry};

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate bitfield_struct;

mod acpi;
mod bitmap;
mod incr_id;
mod interrupts;
mod logger;
mod system;
mod timer;

#[macro_export]
macro_rules! hlt_loop {
    () => {
        loop {
            unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) }
        }
    };
}

#[macro_export]
macro_rules! sti {
    () => {
        unsafe { core::arch::asm!("sti", options(nomem, nostack)) }
    };
}

#[macro_export]
macro_rules! cli {
    () => {
        unsafe { core::arch::asm!("cli", options(nomem, nostack)) }
    };
}

pub fn init_core(boot_info: &skyliftkit::BootInfo) {
    let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    state.verbose = boot_info.verbose;
    state.serial_enabled = boot_info.serial_enabled || boot_info.frame_buffer.is_none();

    unsafe {
        crate::system::gdt::GDTR.load();
        crate::interrupts::idt::IDTR.load();
        crate::interrupts::init_quirks();
        crate::system::exceptions::init();
    }

    state.pmm = Some(BitmapAllocator::new(boot_info.memory_map).into());

    let root = OSDTEntry {
        properties: HashMap::from([
            (OSDTENTRY_NAME_KEY.into(), "Root".into()),
            ("Version".into(), "0.0.1".into()),
        ]),
        ..Default::default()
    };
    state.dt_index = Some(HashMap::from([(root.id, root.into())]).into());
    state.dt_id_gen = Some(IncrementalIDGen::default().into());

    unsafe {
        state.acpi = Some(ACPIState::new(
            &*boot_info.acpi_rsdp.cast::<RootSystemDescPtr>(),
        ));
    }
}

pub fn init_paging(state: &mut crate::system::state::SystemState) {
    debug!("Initialising paging");

    let mut pml4 = Box::new(crate::system::vmm::PageTableLvl4::new());
    unsafe { pml4.init() }
    state.pml4 = Some(pml4.into());

    if let Some(v) = state.terminal.as_mut() {
        v.map_fb();
    }
}

#[no_mangle]
extern "C" fn kernel_main(boot_info: &'static skyliftkit::BootInfo) -> ! {
    logger::init();
    assert_eq!(boot_info.revision, skyliftkit::CURRENT_REVISION);
    init_core(boot_info);
    debug!("Copyright ChefKiss 2021-2025.");

    let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    state.terminal = boot_info.frame_buffer.map(|fb_info| {
        trace!("Got boot display: {fb_info:X?}");
        let mut terminal = crate::system::terminal::Terminal::new(unsafe {
            skybuffer::fb::FrameBuffer::new(
                fb_info.base,
                fb_info.resolution.width,
                fb_info.resolution.height,
                fb_info.pitch,
                fb_info.pixel_format,
            )
        });
        terminal.clear();
        terminal
    });

    init_paging(state);

    acpi::madt::setup(state);
    acpi::apic::setup(state);

    system::tasking::userland::setup();

    let fkcache: SKExtensions = postcard::from_bytes(boot_info.fkcache).unwrap();
    state.fkcache = Some(fkcache.into());
    state.scheduler =
        Some(system::tasking::scheduler::Scheduler::new(&acpi::get_hpet(state)).into());

    system::fkext::spawn_initial_matches();

    debug!("I'm out of here!");
    system::tasking::scheduler::Scheduler::unmask();

    hlt_loop!();
}
