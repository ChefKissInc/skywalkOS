use core::{fmt::Write, mem::size_of};

use amd64::paging::{pml4::PML4, PageTableEntry};
use cardboard_klib::{
    request::{KernelRequest, KernelRequestStatus},
    Message, MessageChannel, MessageChannelEntry,
};

use crate::sys::{gdt::PrivilegeLevel, RegisterState};

unsafe extern "C" fn syscall_handler(state: &mut RegisterState) {
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut scheduler = sys_state.scheduler.get_mut().unwrap().lock();

    if let Some(v) = (state.rdi as *const KernelRequest).as_ref() {
        match v {
            KernelRequest::Print(s) => {
                if s.as_ptr().is_null() {
                    error!("Failed to print message: invalid pointer");
                    state.rax = KernelRequestStatus::InvalidRequest.into();
                } else if let Ok(s) = core::str::from_utf8(s) {
                    let mut serial = crate::sys::io::serial::SERIAL.lock();
                    write!(serial, "{s}").unwrap();
                    if let Some(terminal) = &mut sys_state.terminal {
                        write!(terminal, "{s}").unwrap();
                    }
                    state.rax = KernelRequestStatus::Success.into();
                } else {
                    state.rax = KernelRequestStatus::MalformedData.into();
                }
            }
            KernelRequest::AcquireMsgChannelRef => {
                let proc_uuid = scheduler.current_thread_mut().unwrap().proc_uuid;
                let process = scheduler.processes.get_mut(&proc_uuid).unwrap();
                let phys = process.message_channel.as_ref() as *const _ as u64
                    - amd64::paging::PHYS_VIRT_OFFSET;
                process.cr3.map_pages(
                    phys,
                    phys,
                    (size_of::<MessageChannel>() as u64 + 0xFFF) / 0x1000,
                    PageTableEntry::new()
                        .with_user(true)
                        .with_writable(true)
                        .with_present(true),
                );
                state.rax = phys;
            }
            KernelRequest::RefreshMessageChannel => {
                let proc_uuid = scheduler.current_thread_mut().unwrap().proc_uuid;
                let process = scheduler.processes.get_mut(&proc_uuid).unwrap();
                state.rax = KernelRequestStatus::Success.into();
                if process.message_backlog.is_empty() {
                    return;
                }
                for ent in process
                    .message_channel
                    .data
                    .iter_mut()
                    .filter(|v| v.is_unoccupied())
                {
                    if let Some(v) = process.message_backlog.pop() {
                        *ent = MessageChannelEntry::Occupied(v);
                    } else {
                        break;
                    }
                }
            }
            KernelRequest::Exit => {
                let index = scheduler
                    .threads
                    .iter()
                    .position(|v| v.uuid == scheduler.current_thread_uuid.unwrap())
                    .unwrap_or_default();
                scheduler.threads.remove(index);
                scheduler.current_thread_uuid = None;
                state.rax = KernelRequestStatus::Success.into();
                drop(scheduler);
                super::sched::schedule(state);
            }
            KernelRequest::ScheduleNext => {
                drop(scheduler);
                state.rax = KernelRequestStatus::Success.into();
                super::sched::schedule(state);
            }
            KernelRequest::SendMessage(_target, data) => {
                let src_proc_uuid = scheduler.current_thread_mut().unwrap().proc_uuid;
                let proc_uuid = scheduler.next_thread_mut().unwrap().proc_uuid;
                let process = scheduler.processes.get_mut(&proc_uuid).unwrap();
                let addr = data.as_ptr() as u64;
                process.cr3.map_pages(
                    addr,
                    addr,
                    (data.len() as u64 + 0xFFF) / 0x1000,
                    PageTableEntry::new().with_user(true).with_present(true),
                );
                state.rax = KernelRequestStatus::Success.into();
                let msg = Message::new(src_proc_uuid, data);
                for ent in &mut process.message_channel.data {
                    if ent.is_unoccupied() {
                        *ent = MessageChannelEntry::Occupied(msg);
                        return;
                    }
                }
                process.message_backlog.push(msg);
            }
        }
    } else {
        state.rax = KernelRequestStatus::InvalidRequest.into();
    }
}

pub fn setup() {
    crate::driver::intrs::idt::set_handler(
        249,
        1,
        PrivilegeLevel::User,
        syscall_handler,
        false,
        true,
    );
}
