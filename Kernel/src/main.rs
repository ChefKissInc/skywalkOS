// Copyright (c) ChefKiss Inc 2021-2022.
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

mod driver;
mod sys;
mod utils;

#[used]
#[no_mangle]
static __stack_chk_guard: u64 = 0x595E9FBD94FDA766;

#[no_mangle]
extern "C" fn __stack_chk_fail() {
    panic!("stack check failure");
}

#[no_mangle]
extern "C" fn kernel_main(boot_info: &'static sulphur_dioxide::BootInfo) -> ! {
    unwinding::panic::catch_unwind(move || real_main(boot_info)).unwrap()
}

fn real_main(boot_info: &sulphur_dioxide::BootInfo) -> ! {
    utils::logger::init();
    assert_eq!(boot_info.revision, sulphur_dioxide::CURRENT_REVISION);
    utils::init_core(boot_info);
    debug!("Copyright ChefKiss Inc 2021-2022.");

    let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
    state.terminal = boot_info.frame_buffer.map(|fb_info| {
        debug!("Got boot display: {:X?}", fb_info);
        let mut terminal = crate::sys::terminal::Terminal::new(unsafe {
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
    let hpet = driver::acpi::get_hpet(state);

    driver::acpi::madt::setup(state);
    driver::acpi::apic::setup(state);

    sys::proc::userland::setup();
    let sched = state
        .scheduler
        .call_once(|| spin::Mutex::new(sys::proc::scheduler::Scheduler::new(&hpet)));
    for module in state.modules.as_ref().unwrap() {
        debug!("Spawning module {}", module.name);
        sched.lock().spawn_proc(module.data);
    }

    debug!("I'm out of here!");
    sys::proc::scheduler::Scheduler::unmask();

    hlt_loop!();
}
