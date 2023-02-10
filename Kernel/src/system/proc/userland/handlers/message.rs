// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use tungstenkit::syscall::{KernelMessage, Message};

use crate::system::{
    proc::{scheduler::Scheduler, ThreadState},
    RegisterState,
};

pub fn send(scheduler: &mut Scheduler, state: &mut RegisterState) {
    if !scheduler.processes.contains_key(&state.rsi) {
        todo!()
    }

    let src = scheduler.current_pid.unwrap();

    let msg = Message::new(scheduler.msg_id_gen.next(), src, unsafe {
        core::slice::from_raw_parts(state.rcx as *const _, state.r8 as _)
    });
    scheduler.message_sources.insert(msg.id, src);

    let process = scheduler.processes.get_mut(&state.rsi).unwrap();
    process.track_msg(msg.id, state.rcx);
    process.messages.push_front(msg);
}

pub fn receive(scheduler: &mut Scheduler, state: &mut RegisterState) {
    let pid = scheduler.current_pid.unwrap();
    let process = scheduler.processes.get_mut(&pid).unwrap();
    if let Some(msg) = process.messages.pop_back() {
        state.rdi = msg.id;
        state.rsi = msg.pid;
        state.rdx = msg.data.as_ptr() as u64;
        state.rcx = msg.data.len() as u64;
    } else {
        scheduler.current_thread_mut().unwrap().state = ThreadState::Suspended;
    }
}

pub fn ack(scheduler: &mut Scheduler, state: &mut RegisterState) {
    let id = state.rsi;

    let Some(&src) = scheduler.message_sources.get(&id) else {
        todo!()
    };
    let pid = scheduler.current_pid.unwrap();
    let process = scheduler.processes.get_mut(&pid).unwrap();
    if src == 0 {
        let addr = *process.message_allocations.get(&id).unwrap();
        let (size, _) = *process.allocations.get(&addr).unwrap();
        let data = addr as *const u8;
        let msg: KernelMessage =
            unsafe { postcard::from_bytes(core::slice::from_raw_parts(data, size as _)).unwrap() };
        let KernelMessage::IRQFired(irq) = msg;
        crate::acpi::ioapic::set_irq_mask(irq, false);
    }
    process.free_msg(id);
    scheduler.msg_id_gen.free(id);
}
