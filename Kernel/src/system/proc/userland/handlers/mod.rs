// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::{fmt::Write, ops::ControlFlow};

use tungstenkit::TerminationReason;

use crate::system::{gdt::PrivilegeLevel, proc::scheduler::Scheduler, RegisterState};

pub mod alloc;
pub mod dt;
pub mod message;
pub mod port;

pub fn kprint(state: &mut RegisterState) -> ControlFlow<Option<TerminationReason>> {
    // TODO: kill process on failure
    let s = unsafe { core::slice::from_raw_parts(state.rsi as *const u8, state.rdx as usize) };
    let Ok(s) = core::str::from_utf8(s) else {
        return ControlFlow::Break(Some(TerminationReason::MalformedBody));
    };

    #[cfg(debug_assertions)]
    write!(crate::system::serial::SERIAL.lock(), "{s}").unwrap();

    if let Some(v) = unsafe { (*crate::system::state::SYS_STATE.get()).terminal.as_mut() } {
        write!(v, "{s}").unwrap()
    }

    ControlFlow::Continue(())
}

pub fn register_irq_handler(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let irq = state.rsi as u8;
    if irq > 0xDF {
        return ControlFlow::Break(Some(TerminationReason::MalformedArgument));
    }
    let pid = scheduler.current_pid.unwrap();
    if scheduler.irq_handlers.try_insert(irq, pid).is_err() {
        return ControlFlow::Break(Some(TerminationReason::AlreadyExists));
    }

    crate::acpi::ioapic::wire_legacy_irq(irq, false);
    crate::intrs::idt::set_handler(
        irq + 0x20,
        1,
        PrivilegeLevel::Supervisor,
        super::irq_handler,
        true,
        true,
    );

    ControlFlow::Continue(())
}

pub fn thread_teardown(scheduler: &mut Scheduler) -> ControlFlow<Option<TerminationReason>> {
    let id = scheduler.current_tid.unwrap();
    scheduler.threads.remove(&id);
    scheduler.tid_gen.free(id);
    scheduler.current_tid = None;

    let pid = scheduler.current_pid.unwrap();
    if !scheduler.threads.iter().any(|(_, v)| v.pid == pid) {
        scheduler.processes.remove(&pid);
        scheduler.pid_gen.free(pid);
        scheduler.current_pid = None;
    }

    ControlFlow::Break(None)
}

pub fn process_teardown(scheduler: &mut Scheduler) {
    let pid = scheduler.current_pid.unwrap();
    scheduler.processes.remove(&pid);
    scheduler.pid_gen.free(pid);
    scheduler.current_pid = None;
    scheduler.threads.retain(|_, v| v.pid != pid);
}
