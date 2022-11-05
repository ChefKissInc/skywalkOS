use core::{fmt::Write, mem::size_of};

use amd64::paging::{pml4::PML4, PageTableEntry};
use cardboard_klib::{
    request::{KernelRequest, KernelRequestStatus},
    MessageChannel,
};

use crate::sys::{gdt::PrivilegeLevel, RegisterState};

unsafe extern "C" fn syscall_handler(state: &mut RegisterState) {
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut scheduler = sys_state.scheduler.get_mut().unwrap().lock();

    if let Some(v) = (state.rdi as *const KernelRequest).as_ref() {
        match v {
            KernelRequest::Print(s) => {
                if s.as_ptr().is_null() {
                    error!(target: "ThreadMessage", "Failed to print message: invalid pointer");
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
                let thread = scheduler.current_thread_mut().unwrap();
                let phys = thread.message_channel.as_ref() as *const _ as u64
                    - amd64::paging::PHYS_VIRT_OFFSET;
                let proc_uuid = thread.proc_uuid;
                let process = scheduler.processes.get_mut(&proc_uuid).unwrap();
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
            KernelRequest::Exit => {
                trace!(target: "ThreadMessage", "Thread requested to exit");
                state.rax = KernelRequestStatus::Unimplemented.into();
            }
            KernelRequest::ScheduleNext => {
                trace!(target: "ThreadMessage", "Thread requested to get skipped");
                state.rax = KernelRequestStatus::Success.into();
                drop(scheduler);
                super::sched::schedule(state);
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
