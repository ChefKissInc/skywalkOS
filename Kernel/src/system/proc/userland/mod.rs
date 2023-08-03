// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::ops::ControlFlow;

use fireworkkit::{
    msg::{KernelMessage, Message},
    syscall::SystemCall,
    TerminationReason,
};

use crate::system::{gdt::PrivilegeLevel, RegisterState};

mod handlers;
pub mod page_table;

unsafe extern "sysv64" fn irq_handler(state: &mut RegisterState) {
    let irq = (state.int_num - 0x20) as u8;
    crate::acpi::ioapic::set_irq_mask(irq, true);
    let mut scheduler = (*crate::system::state::SYS_STATE.get())
        .scheduler
        .as_ref()
        .unwrap()
        .lock();
    let pid = scheduler.irq_handlers.get(&irq).copied().unwrap();
    let s = postcard::to_allocvec(&KernelMessage::IRQFired(irq))
        .unwrap()
        .leak();

    let virt = scheduler
        .processes
        .get_mut(&pid)
        .unwrap()
        .track_kernelside_alloc(s.as_ptr() as _, s.len() as _);

    let msg = Message::new(
        scheduler.msg_id_gen.next(),
        0,
        core::slice::from_raw_parts(virt as *const _, s.len() as _),
    );
    scheduler.message_sources.insert(msg.id, 0);
    let process = scheduler.processes.get_mut(&pid).unwrap();
    process.track_msg(msg.id, virt);

    let tids = process.thread_ids.clone();
    if handlers::message::handle_new(&mut scheduler, pid, &tids, msg).is_break() {
        drop(scheduler);
        super::scheduler::schedule(state);
    }
}

unsafe extern "sysv64" fn syscall_handler(state: &mut RegisterState) {
    let sys_state = &mut *crate::system::state::SYS_STATE.get();
    let mut scheduler = sys_state.scheduler.as_ref().unwrap().lock();

    let flow = 'flow: {
        let Ok(v) = SystemCall::try_from(state.rdi) else {
            break 'flow ControlFlow::Break(Some(TerminationReason::MalformedArgument));
        };

        match v {
            SystemCall::KPrint => handlers::kprint(state),
            SystemCall::ReceiveMessage => handlers::message::receive(&mut scheduler, state),
            SystemCall::SendMessage => handlers::message::send(&mut scheduler, state),
            SystemCall::Quit => handlers::thread_teardown(&mut scheduler),
            SystemCall::Yield => ControlFlow::Break(None),
            SystemCall::PortIn => handlers::port::port_in(state),
            SystemCall::PortOut => handlers::port::port_out(state),
            SystemCall::RegisterIRQHandler => handlers::register_irq_handler(&mut scheduler, state),
            SystemCall::Allocate => handlers::alloc::alloc(&mut scheduler, state),
            SystemCall::Free => handlers::alloc::free(&mut scheduler, state),
            SystemCall::AckMessage => handlers::message::ack(&mut scheduler, state),
            SystemCall::NewOSDTEntry => handlers::osdtentry::new_entry(state),
            SystemCall::GetOSDTEntryInfo => handlers::osdtentry::get_info(&mut scheduler, state),
            SystemCall::SetOSDTEntryProp => handlers::osdtentry::set_prop(&mut scheduler, state),
        }
    };

    let ControlFlow::Break(reason) = flow else {
        return;
    };

    if let Some(reason) = reason {
        debug!(
            "PID {} performed illegal action (<{reason:?}>), good riddance.",
            scheduler.current_pid.unwrap()
        );
        handlers::process_teardown(&mut scheduler);
    }
    drop(scheduler);
    super::scheduler::schedule(state);
}

pub fn setup() {
    crate::intrs::idt::set_handler(249, 1, PrivilegeLevel::User, syscall_handler, false, true);
}
