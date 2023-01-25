// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![feature(
    asm_const,
    alloc_error_handler,
    const_size_of_val,
    naked_functions,
    const_mut_refs,
    sync_unsafe_cell
)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

mod acpi;
mod intrs;
mod system;
mod timer;
mod utils;

#[used]
#[no_mangle]
static __stack_chk_guard: u64 = 0x595E_9FBD_94FD_A766;

#[no_mangle]
extern "C" fn __stack_chk_fail() {
    panic!("stack check failure");
}

#[no_mangle]
extern "C" fn kernel_main(boot_info: &'static sulphur_dioxide::BootInfo) -> ! {
    utils::logger::init();
    assert_eq!(boot_info.revision, sulphur_dioxide::CURRENT_REVISION);
    utils::init_core(boot_info);
    debug!("Copyright ChefKiss Inc 2021-2023.");

    let state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
    state.terminal = boot_info.frame_buffer.map(|fb_info| {
        debug!("Got boot display: {:X?}", fb_info);
        let mut terminal = crate::system::terminal::Terminal::new(unsafe {
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
        terminal
    });

    utils::init_paging(state);

    let hpet = acpi::get_hpet(state);

    acpi::madt::setup(state);
    acpi::apic::setup(state);

    system::proc::userland::setup();
    let sched = state
        .scheduler
        .call_once(|| spin::Mutex::new(system::proc::scheduler::Scheduler::new(&hpet)));
    let cache: driver_core::DCCache =
        postcard::from_bytes(state.dc_cache.as_ref().unwrap()).unwrap();

    let len = cache.infos.len();
    info!("Got {len} DriverCore extensions");
    for info in cache.infos {
        info!(
            "Spawning DriverCore extension {} <{}>",
            info.name, info.identifier
        );
        sched.lock().spawn_proc(cache.payloads[info.identifier]);
    }

    debug!("I'm out of here!");
    system::proc::scheduler::Scheduler::unmask();

    hlt_loop!();
}
