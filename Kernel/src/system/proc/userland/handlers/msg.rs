// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::ops::ControlFlow;

use amd64::paging::PageTableEntry;
use fireworkkit::{
    msg::{KernelMessage, Message},
    TerminationReason,
};
use hashbrown::HashSet;

use crate::system::{
    proc::{scheduler::Scheduler, ThreadState},
    RegisterState,
};

pub fn handle_new(
    scheduler: &mut Scheduler,
    pid: u64,
    tids: HashSet<u64>,
    msg: Message,
) -> ControlFlow<Option<TerminationReason>> {
    let idle = scheduler.current_tid.is_none();
    for tid in tids {
        let thread = scheduler.threads.get_mut(&tid).unwrap();
        if thread.state.is_suspended() {
            thread.state = ThreadState::Inactive;
            thread.regs.rax = msg.id;
            thread.regs.rdi = msg.pid;
            thread.regs.rsi = msg.data.as_ptr() as _;
            thread.regs.rdx = msg.data.len() as _;
            if idle {
                return ControlFlow::Break(None);
            }
            return ControlFlow::Continue(());
        }
    }
    let process = scheduler.processes.get_mut(&pid).unwrap();
    process.messages.push_front(msg);
    ControlFlow::Continue(())
}

pub fn send(
    scheduler: &mut Scheduler,
    state: &RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let target = state.rsi;

    if !scheduler.processes.contains_key(&target) {
        return ControlFlow::Break(Some(TerminationReason::NotFound));
    }

    let (addr, size) = (state.rdx, state.rcx);
    let src = scheduler.current_pid.unwrap();
    let msg = Message::new(scheduler.msg_id_gen.next(), src, unsafe {
        core::slice::from_raw_parts(addr as *const _, size as _)
    });
    scheduler.message_sources.insert(msg.id, src);

    let cur = scheduler.current_process_mut().unwrap();
    cur.track_msg(msg.id, addr);
    if target != src {
        let process = scheduler.processes.get(&target).unwrap();
        unsafe {
            process.cr3.lock().map(
                addr,
                addr - fireworkkit::USER_PHYS_VIRT_OFFSET,
                (size + 0xFFF) / 0x1000,
                PageTableEntry::new().with_present(true).with_user(true),
            );
        }
        let tids = process.thread_ids.clone();
        return handle_new(scheduler, target, tids, msg);
    }
    cur.messages.push_front(msg);
    ControlFlow::Continue(())
}

pub fn recv(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let process = scheduler.current_process_mut().unwrap();
    let Some(msg) = process.messages.pop_back() else {
        scheduler.current_thread_mut().unwrap().state = ThreadState::Suspended;
        return ControlFlow::Break(None);
    };

    state.rax = msg.id;
    state.rdi = msg.pid;
    state.rsi = msg.data.as_ptr() as u64;
    state.rdx = msg.data.len() as u64;
    ControlFlow::Continue(())
}

pub fn ack(
    scheduler: &mut Scheduler,
    state: &RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let msg_id = state.rsi;

    let Some(src_pid) = scheduler.message_sources.remove(&msg_id) else {
        return ControlFlow::Break(Some(TerminationReason::NotFound));
    };

    let cur_pid = scheduler.current_pid.unwrap();
    let pid = if src_pid == 0 { cur_pid } else { src_pid };
    let process = scheduler.processes.get_mut(&pid).unwrap();
    let addr = *process.msg_id_to_addr.get(&msg_id).unwrap();
    let size = process.allocations.get(&addr).copied().unwrap().0;
    if src_pid == 0 {
        let msg: KernelMessage = unsafe {
            postcard::from_bytes(core::slice::from_raw_parts(addr as *const _, size as _)).unwrap()
        };
        let KernelMessage::IRQFired(irq) = msg;
        crate::acpi::ioapic::set_irq_mask(irq, false);
    }
    process.free_msg(msg_id);
    scheduler.msg_id_gen.free(msg_id);
    if pid != cur_pid {
        let process = scheduler.current_process().unwrap();
        unsafe {
            process.cr3.lock().unmap(addr, (size + 0xFFF) / 0x1000);
        }
    }

    ControlFlow::Continue(())
}
