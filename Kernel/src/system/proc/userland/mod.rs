// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::ops::ControlFlow;

use tungstenkit::syscall::{KernelMessage, Message, SystemCall};

use crate::system::{gdt::PrivilegeLevel, RegisterState};

mod handlers;
pub mod page_table;

unsafe extern "C" fn irq_handler(state: &mut RegisterState) {
    let irq = (state.int_num - 0x20) as u8;
    crate::acpi::ioapic::set_irq_mask(irq, true);
    let mut scheduler = (*crate::system::state::SYS_STATE.get())
        .scheduler
        .as_ref()
        .unwrap()
        .lock();
    let pid = scheduler.irq_handlers.get(&irq).cloned().unwrap();
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

    let tids = process.tids.clone();
    let idle = scheduler.current_tid.is_none();
    for tid in tids.into_iter() {
        let thread = scheduler.threads.get_mut(&tid).unwrap();
        if thread.state.is_suspended() {
            thread.state = super::ThreadState::Inactive;
            if idle {
                drop(scheduler);
                super::scheduler::schedule(state);
                state.rax = msg.id;
                state.rdi = msg.pid;
                state.rsi = msg.data.as_ptr() as _;
                state.rdx = msg.data.len() as _;
                return;
            }
            break;
        }
    }

    let process = scheduler.processes.get_mut(&pid).unwrap();
    process.messages.push_front(msg);
}

unsafe extern "C" fn syscall_handler(state: &mut RegisterState) {
    let sys_state = &mut *crate::system::state::SYS_STATE.get();
    let mut scheduler = sys_state.scheduler.as_ref().unwrap().lock();

    let mut flow = 'flow: {
        let Ok(v) = SystemCall::try_from(state.rdi) else {
            break 'flow ControlFlow::Break(true);
        };

        match v {
            SystemCall::KPrint => handlers::kprint(state),
            SystemCall::ReceiveMessage => handlers::message::receive(&mut scheduler, state),
            SystemCall::SendMessage => handlers::message::send(&mut scheduler, state),
            SystemCall::Quit => {
                handlers::thread_teardown(&mut scheduler);
                ControlFlow::Break(false)
            }
            SystemCall::Yield => ControlFlow::Break(false),
            SystemCall::RegisterProvider => handlers::provider::register(&mut scheduler, state),
            SystemCall::GetProvidingProcess => handlers::provider::get(&mut scheduler, state),
            SystemCall::PortIn => handlers::port::port_in(state),
            SystemCall::PortOut => handlers::port::port_out(state),
            SystemCall::RegisterIRQHandler => handlers::register_irq_handler(&mut scheduler, state),
            SystemCall::Allocate => handlers::alloc::alloc(&mut scheduler, state),
            SystemCall::Free => handlers::alloc::free(&mut scheduler, state),
            SystemCall::AckMessage => handlers::message::ack(&mut scheduler, state),
            SystemCall::GetDTEntryInfo => {
                handlers::device_tree::get_entry_info(&mut scheduler, state)
            }
        }
    };

    if flow == ControlFlow::Continue(())
        && scheduler.current_thread_mut().unwrap().state.is_suspended()
    {
        flow = ControlFlow::Break(false);
    }

    if let ControlFlow::Break(kill) = flow {
        if kill {
            debug!(
                "Process {} caused error, killing",
                scheduler.current_pid.unwrap()
            );
            handlers::process_teardown(&mut scheduler);
        }
        drop(scheduler);
        super::scheduler::schedule(state);
    }
}

pub fn setup() {
    crate::intrs::idt::set_handler(249, 1, PrivilegeLevel::User, syscall_handler, false, true);
}
