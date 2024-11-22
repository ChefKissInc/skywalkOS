// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::ops::ControlFlow;

use skykit::{syscall::SystemCall, TerminationReason};

use crate::system::{gdt::PrivilegeLevel, RegisterState};

pub mod handlers;
pub mod page_table;

unsafe extern "sysv64" fn syscall_handler(state: &mut RegisterState) {
    let sys_state = &mut *crate::system::state::SYS_STATE.get();
    let mut scheduler = sys_state.scheduler.as_ref().unwrap().lock();

    let flow = 'flow: {
        let Ok(v) = SystemCall::try_from(state.rdi) else {
            break 'flow ControlFlow::Break(Some(TerminationReason::MalformedArgument));
        };

        match v {
            SystemCall::KPrint => handlers::kprint(&scheduler, state),
            SystemCall::MsgRecv => handlers::msg::recv(&mut scheduler, state),
            SystemCall::MsgSend => handlers::msg::send(&mut scheduler, state),
            SystemCall::Quit => scheduler.thread_teardown(),
            SystemCall::Yield => ControlFlow::Break(None),
            SystemCall::PortIn => handlers::port::port_in(state),
            SystemCall::PortOut => handlers::port::port_out(state),
            SystemCall::RegisterIRQ => scheduler.register_irq(state),
            SystemCall::Allocate => handlers::alloc::alloc(&mut scheduler, state),
            SystemCall::Free => handlers::alloc::free(&mut scheduler, state),
            SystemCall::MsgAck => handlers::msg::ack(&mut scheduler, state),
            SystemCall::NewOSDTEntry => handlers::os_dt_entry::new_entry(state),
            SystemCall::GetOSDTEntryInfo => handlers::os_dt_entry::get_info(&mut scheduler, state),
            SystemCall::SetOSDTEntryProp => handlers::os_dt_entry::set_prop(&mut scheduler, state),
        }
    };

    let ControlFlow::Break(reason) = flow else {
        return;
    };

    if let Some(reason) = reason {
        debug!(
            "PID {} performed illegal action (<{reason:?}>). Killing it, good riddance.",
            scheduler.current_pid.unwrap()
        );
        scheduler.process_teardown();
    }
    scheduler.schedule(state);
}

pub fn setup() {
    crate::interrupts::idt::set_handler(249, 1, PrivilegeLevel::User, syscall_handler, false, true);
}
