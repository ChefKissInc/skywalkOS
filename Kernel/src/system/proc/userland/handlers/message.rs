// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use iridium_kit::syscall::{KernelMessage, Message, SystemCallStatus};

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn send(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    if !scheduler.processes.contains_key(&state.rsi) {
        return SystemCallStatus::MalformedData;
    }

    let src = scheduler.current_thread_mut().unwrap().proc_id;

    let msg = Message::new(scheduler.message_id_gen.next(), src, unsafe {
        core::slice::from_raw_parts(state.rcx as *const _, state.r8 as _)
    });
    scheduler.message_sources.insert(msg.id, src);

    let process = scheduler.processes.get_mut(&state.rsi).unwrap();
    let sys_state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
    sys_state
        .user_allocations
        .get_mut()
        .unwrap()
        .lock()
        .track_msg(msg.id, state.rcx);
    process.messages.push_front(msg);
    SystemCallStatus::Success
}

pub fn receive(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
    let process = scheduler.processes.get_mut(&proc_id).unwrap();
    if let Some(msg) = process.messages.pop_back() {
        state.rdi = msg.id;
        state.rsi = msg.proc_id;
        state.rdx = msg.data.as_ptr() as u64;
        state.rcx = msg.data.len() as u64;
    } else {
        // TODO: block thread until there's a new message
        state.rdi = 0;
    }
    SystemCallStatus::Success
}

pub fn ack(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    let id = state.rsi;

    let Some(&src) = scheduler.message_sources.get(&id) else {
        return SystemCallStatus::MalformedData;
    };
    let sys_state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
    let mut user_allocations = sys_state.user_allocations.get_mut().unwrap().lock();
    if src == 0 {
        let addr = *user_allocations.message_allocations.get(&id).unwrap();
        let size = user_allocations.allocations.get(&addr).unwrap().1;
        let data = addr as *const u8;
        let msg: KernelMessage =
            unsafe { postcard::from_bytes(core::slice::from_raw_parts(data, size as _)).unwrap() };
        let KernelMessage::IRQFired(irq) = msg;
        crate::acpi::ioapic::set_irq_mask(irq, false);
    }
    user_allocations.free_msg(id);
    scheduler.message_id_gen.free(id);
    SystemCallStatus::Success
}
