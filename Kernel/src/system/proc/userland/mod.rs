// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use amd64::paging::{pml4::PML4, PageTableEntry};
use tungstenkit::syscall::{KernelMessage, Message, SystemCall, SystemCallStatus};

use crate::system::{gdt::PrivilegeLevel, RegisterState};

pub mod allocations;
mod handlers;
pub mod page_table;

unsafe extern "C" fn irq_handler(state: &mut RegisterState) {
    let irq = (state.int_num - 0x20) as u8;
    crate::acpi::ioapic::set_irq_mask(irq, true);
    let sys_state = &mut *crate::system::state::SYS_STATE.get();
    let mut scheduler = sys_state.scheduler.as_ref().unwrap().lock();
    let proc_id = *scheduler.irq_handlers.get(&irq).unwrap();
    let s = postcard::to_allocvec(&KernelMessage::IRQFired(irq))
        .unwrap()
        .leak();
    let ptr = s.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
    let virt = ptr + tungstenkit::USER_PHYS_VIRT_OFFSET;
    let count = (s.len() as u64 + 0xFFF) / 0x1000;
    let mut usr_allocs = sys_state.usr_allocs.as_ref().unwrap().lock();
    usr_allocs.track(proc_id, virt, s.len() as u64);
    let msg = Message::new(
        scheduler.message_id_gen.next(),
        0,
        core::slice::from_raw_parts(virt as *const _, s.len() as _),
    );
    scheduler.message_sources.insert(msg.id, 0);
    let process = scheduler.processes.get_mut(&proc_id).unwrap();
    process.cr3.map_pages(
        virt,
        ptr,
        count,
        PageTableEntry::new().with_present(true).with_user(true),
    );
    usr_allocs.track_msg(msg.id, virt);
    process.messages.push_front(msg);
}

unsafe extern "C" fn syscall_handler(state: &mut RegisterState) {
    let sys_state = &mut *crate::system::state::SYS_STATE.get();
    let mut scheduler = sys_state.scheduler.as_ref().unwrap().lock();

    let Ok(v) = SystemCall::try_from(state.rdi) else {
        state.rax = SystemCallStatus::UnknownRequest.into();
        return;
    };

    state.rax = match v {
        SystemCall::KPrint => handlers::kprint(state).into(),
        SystemCall::ReceiveMessage => handlers::message::receive(&mut scheduler, state).into(),
        SystemCall::SendMessage => handlers::message::send(&mut scheduler, state).into(),
        SystemCall::Exit => {
            handlers::process_teardown(&mut scheduler);
            drop(scheduler);
            super::scheduler::schedule(state);
            return;
        }
        SystemCall::Yield => {
            drop(scheduler);
            super::scheduler::schedule(state);
            return;
        }
        SystemCall::RegisterProvider => handlers::provider::register(&mut scheduler, state).into(),
        SystemCall::GetProviderForProcess => {
            handlers::provider::get_for_process(&mut scheduler, state).into()
        }
        SystemCall::PortIn => handlers::port::port_in(state).into(),
        SystemCall::PortOut => handlers::port::port_out(state).into(),
        SystemCall::RegisterIRQHandler => 'a: {
            let irq = state.rsi as u8;
            if irq > 0xDF {
                break 'a SystemCallStatus::MalformedData.into();
            }
            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            if scheduler.irq_handlers.try_insert(irq, proc_id).is_err() {
                break 'a SystemCallStatus::InvalidRequest.into();
            }

            crate::acpi::ioapic::wire_legacy_irq(irq, false);
            crate::intrs::idt::set_handler(
                irq + 0x20,
                1,
                PrivilegeLevel::Supervisor,
                irq_handler,
                true,
                true,
            );
            SystemCallStatus::Success.into()
        }
        SystemCall::Allocate => handlers::alloc::alloc(&mut scheduler, state).into(),
        SystemCall::Free => handlers::alloc::free(&mut scheduler, state).into(),
        SystemCall::AckMessage => handlers::message::ack(&mut scheduler, state).into(),
        SystemCall::GetDTEntryInfo => {
            handlers::device_tree::get_entry_info(&mut scheduler, state).into()
        }
    };
}

pub fn setup() {
    let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    state.usr_allocs = Some(spin::Mutex::new(allocations::UserAllocationTracker::new()));
    crate::intrs::idt::set_handler(249, 1, PrivilegeLevel::User, syscall_handler, false, true);
}
