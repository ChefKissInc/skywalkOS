use core::fmt::Write;

use amd64::{
    io::port::PortIO,
    paging::{pml4::PML4, PageTableEntry},
};
use cardboard_klib::{KernelMessage, SystemCall, SystemCallStatus};

use crate::sys::{gdt::PrivilegeLevel, RegisterState};

unsafe extern "C" fn irq_handler(state: &mut RegisterState) {
    let irq = (state.int_num - 0x20) as u8;
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut scheduler = sys_state.scheduler.get_mut().unwrap().lock();
    let proc_uuid = *scheduler.irq_handlers.get(&irq).unwrap();
    let process = scheduler.processes.get_mut(&proc_uuid).unwrap();
    let s = postcard::to_allocvec(&KernelMessage::IRQFired(irq))
        .unwrap()
        .leak();
    let ptr = s.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
    let len = s.len() as u64;
    process.cr3.map_pages(
        ptr,
        ptr,
        (len + 0xFFF) / 0x1000,
        PageTableEntry::new().with_user(true).with_present(true),
    );
    process.messages.push_front((uuid::Uuid::nil(), ptr, len));
}

unsafe extern "C" fn syscall_handler(state: &mut RegisterState) {
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut scheduler = sys_state.scheduler.get_mut().unwrap().lock();

    let Ok(v) = SystemCall::try_from(state.rdi) else {
        state.rax = SystemCallStatus::UnknownRequest.into();
        return;
    };

    match v {
        SystemCall::KPrint => {
            let s = core::slice::from_raw_parts(state.rsi as *const u8, state.rdx as usize);
            if s.as_ptr().is_null() {
                state.rax = SystemCallStatus::MalformedData.into();
                return;
            }
            let Ok(s) = core::str::from_utf8(s) else {
                state.rax = SystemCallStatus::MalformedData.into();
                return;
            };
            let mut serial = crate::sys::io::serial::SERIAL.lock();
            write!(serial, "{s}").unwrap();
            if let Some(terminal) = &mut sys_state.terminal {
                write!(terminal, "{s}").unwrap();
            }
            state.rax = 0;
        }
        SystemCall::ReceiveMessage => {
            let proc_uuid = scheduler.current_thread_mut().unwrap().proc_uuid;
            let process = scheduler.processes.get_mut(&proc_uuid).unwrap();
            let Some((source, ptr, len)) = process.messages.pop_back() else {
                state.rax = SystemCallStatus::DoNothing.into();
                return;
            };
            let (uuid_hi, uuid_lo) = source.as_u64_pair();
            state.rax = 0;
            state.rdi = uuid_hi;
            state.rsi = uuid_lo;
            state.rdx = ptr;
            state.rcx = len;
        }
        SystemCall::Exit => {
            let index = scheduler
                .threads
                .iter()
                .position(|v| v.uuid == scheduler.current_thread_uuid.unwrap())
                .unwrap();
            scheduler.threads.remove(index);
            scheduler.current_thread_uuid = None;
            state.rax = 0;
            drop(scheduler);
            super::sched::schedule(state);
        }
        SystemCall::Skip => {
            state.rax = 0;
            drop(scheduler);
            super::sched::schedule(state);
        }
        SystemCall::SendMessage => {
            let src = scheduler.current_thread_mut().unwrap().proc_uuid;
            let dest = uuid::Uuid::from_u64_pair(state.rsi, state.rdx);
            if dest.is_nil() {
                state.rax = SystemCallStatus::MalformedData.into();
                return;
            }
            let Some(process) = scheduler.processes.get_mut(&dest) else {
                state.rax = SystemCallStatus::MalformedData.into();
                return;
            };
            process.messages.push_front((src, state.rcx, state.r8));
            state.rax = 0;
        }
        SystemCall::RegisterProvider => {
            let provider_uuid = uuid::Uuid::from_u64_pair(state.rsi, state.rdx);
            if provider_uuid.is_nil() {
                state.rax = SystemCallStatus::MalformedData.into();
                return;
            }
            let proc_uuid = scheduler.current_thread_mut().unwrap().proc_uuid;
            if scheduler
                .providers
                .try_insert(provider_uuid, proc_uuid)
                .is_err()
            {
                state.rax = SystemCallStatus::InvalidRequest.into();
                return;
            }
            state.rax = 0;
        }
        SystemCall::GetProvidingProcess => {
            let provider_uuid = uuid::Uuid::from_u64_pair(state.rsi, state.rdx);
            if provider_uuid.is_nil() {
                state.rax = SystemCallStatus::MalformedData.into();
                return;
            }
            let Some(proc_uuid) = scheduler.providers.get(&provider_uuid) else {
                state.rax = SystemCallStatus::MalformedData.into();
                return;
            };
            let (hi, lo) = proc_uuid.as_u64_pair();
            state.rax = 0;
            state.rdi = hi;
            state.rsi = lo;
        }
        SystemCall::PortInByte => {
            let port = state.rsi as u16;
            state.rax = 0;
            state.rdi = u8::read(port) as u64;
        }
        SystemCall::PortInWord => {
            let port = state.rsi as u16;
            state.rax = 0;
            state.rdi = u16::read(port) as u64;
        }
        SystemCall::PortInDWord => {
            let port = state.rsi as u16;
            state.rax = 0;
            state.rdi = u32::read(port) as u64;
        }
        SystemCall::PortOutByte => {
            let port = state.rsi as u16;
            let value = state.rdx as u8;
            state.rax = 0;
            u8::write(port, value);
        }
        SystemCall::PortOutWord => {
            let port = state.rsi as u16;
            let value = state.rdx as u16;
            state.rax = 0;
            u16::write(port, value);
        }
        SystemCall::PortOutDWord => {
            let port = state.rsi as u16;
            let value = state.rdx as u32;
            state.rax = 0;
            u32::write(port, value);
        }
        SystemCall::RegisterIRQHandler => {
            let irq = state.rsi as u8;
            if state.rdx == 0 {
                state.rax = SystemCallStatus::MalformedData.into();
                return;
            }

            let proc_uuid = scheduler.current_thread_mut().unwrap().proc_uuid;
            if scheduler.irq_handlers.try_insert(irq, proc_uuid).is_err() {
                state.rax = SystemCallStatus::InvalidRequest.into();
                return;
            }

            crate::driver::acpi::ioapic::wire_legacy_irq(irq, false);
            crate::driver::intrs::idt::set_handler(
                irq + 0x20,
                0,
                PrivilegeLevel::Supervisor,
                irq_handler,
                true,
                true,
            )
        }
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
