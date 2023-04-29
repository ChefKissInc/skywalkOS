// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::ops::ControlFlow;

use amd64::paging::{pml4::PML4, PageTableEntry};
use tungstenkit::{
    syscall::{KernelMessage, Message},
    TerminationReason,
};

use crate::system::{
    proc::{scheduler::Scheduler, ThreadState},
    RegisterState,
};

pub fn handle_new(
    scheduler: &mut Scheduler,
    pid: u64,
    tids: &[u64],
    msg: Message,
) -> ControlFlow<Option<TerminationReason>> {
    let idle = scheduler.current_tid.is_none();
    let mut was_suspended = false;
    for tid in tids {
        let thread = scheduler.threads.get_mut(tid).unwrap();
        if thread.state.is_suspended() {
            was_suspended = true;
            thread.state = ThreadState::Inactive;
            thread.regs.rax = msg.id;
            thread.regs.rdi = msg.pid;
            thread.regs.rsi = msg.data.as_ptr() as _;
            thread.regs.rdx = msg.data.len() as _;
            if idle {
                return ControlFlow::Break(None);
            }
            break;
        }
    }
    if !was_suspended {
        let process = scheduler.processes.get_mut(&pid).unwrap();
        process.messages.push_front(msg);
    }
    ControlFlow::Continue(())
}

pub fn send(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    if !scheduler.processes.contains_key(&state.rsi) {
        return ControlFlow::Break(Some(TerminationReason::NotFound));
    }

    let src = scheduler.current_pid.unwrap();
    let msg = Message::new(scheduler.msg_id_gen.next(), src, unsafe {
        core::slice::from_raw_parts(state.rdx as *const _, state.rcx as _)
    });
    scheduler.message_sources.insert(msg.id, src);

    let target = state.rsi;
    let cur = scheduler.current_process_mut().unwrap();
    cur.track_msg(msg.id, state.rdx);
    if target != src {
        let process = scheduler.processes.get_mut(&target).unwrap();
        unsafe {
            process.cr3.map_pages(
                state.rdx,
                state.rdx - tungstenkit::USER_PHYS_VIRT_OFFSET,
                (state.rcx + 0xFFF) / 0x1000,
                PageTableEntry::new().with_present(true).with_user(true),
            );
        }
        let tids = process.tids.clone();
        return handle_new(scheduler, target, &tids, msg);
    }
    cur.messages.push_front(msg);
    ControlFlow::Continue(())
}

pub fn receive(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let process = scheduler.current_process_mut().unwrap();
    if let Some(msg) = process.messages.pop_back() {
        state.rax = msg.id;
        state.rdi = msg.pid;
        state.rsi = msg.data.as_ptr() as u64;
        state.rdx = msg.data.len() as u64;
        ControlFlow::Continue(())
    } else {
        scheduler.current_thread_mut().unwrap().state = ThreadState::Suspended;
        ControlFlow::Break(None)
    }
}

pub fn ack(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let id = state.rsi;

    let Some(&src) = scheduler.message_sources.get(&id) else {
        return ControlFlow::Break(Some(TerminationReason::NotFound));
    };
    scheduler.message_sources.remove(&id);

    let src_process = if src == 0 {
        scheduler.current_process_mut().unwrap()
    } else {
        scheduler.processes.get_mut(&src).unwrap()
    };
    let addr = src_process.message_allocations.get(&id).copied().unwrap();
    let (size, _) = src_process.allocations.get(&addr).copied().unwrap();
    if src == 0 {
        let msg: KernelMessage = unsafe {
            postcard::from_bytes(core::slice::from_raw_parts(addr as *const _, size as _)).unwrap()
        };
        let KernelMessage::IRQFired(irq) = msg;
        crate::acpi::ioapic::set_irq_mask(irq, false);
    }
    src_process.free_msg(id);
    scheduler.msg_id_gen.free(id);
    if src != 0 && src != scheduler.current_pid.unwrap() {
        let process = scheduler.current_process_mut().unwrap();
        unsafe { process.cr3.unmap_pages(addr, (size + 0xFFF) / 0x1000) }
    }

    ControlFlow::Continue(())
}
