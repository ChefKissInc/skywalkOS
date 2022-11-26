// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::boxed::Box;
use core::{fmt::Write, mem::size_of};

use amd64::{
    io::port::PortIO,
    paging::{pml4::PML4, PageTableEntry},
};
use cardboard_klib::{KernelMessage, Message, SystemCall, SystemCallStatus};

use crate::sys::{gdt::PrivilegeLevel, RegisterState};

pub mod allocations;

pub const USER_PHYS_VIRT_OFFSET: u64 = 0xC0000000;

// This isn't meant to be user-accessible.
// It is meant to track the allocations so that they are deallocated when the process exits.
#[derive(Debug)]
pub struct UserPageTableLvl4(uuid::Uuid, amd64::paging::PageTable);

impl UserPageTableLvl4 {
    pub const fn new(proc_id: uuid::Uuid) -> Self {
        Self(proc_id, amd64::paging::PageTable::new())
    }
}

impl PML4 for UserPageTableLvl4 {
    const VIRT_OFF: u64 = amd64::paging::PHYS_VIRT_OFFSET;

    #[inline]
    fn get_entry(&mut self, offset: u64) -> &mut amd64::paging::PageTableEntry {
        &mut self.1.entries[offset as usize]
    }

    #[inline]
    fn alloc_entry(&self) -> u64 {
        let phys = Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as u64
            - amd64::paging::PHYS_VIRT_OFFSET;
        let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
        state.user_allocations.get_mut().unwrap().lock().track(
            self.0,
            phys,
            size_of::<amd64::paging::PageTable>() as _,
        );
        phys
    }
}

unsafe extern "C" fn irq_handler(state: &mut RegisterState) {
    let irq = (state.int_num - 0x20) as u8;
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut scheduler = sys_state.scheduler.get_mut().unwrap().lock();
    let proc_id = *scheduler.irq_handlers.get(&irq).unwrap();
    let process = scheduler.processes.get_mut(&proc_id).unwrap();
    let s = postcard::to_allocvec(&KernelMessage::IRQFired(irq))
        .unwrap()
        .leak();
    let ptr = s.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
    let virt = ptr + USER_PHYS_VIRT_OFFSET;
    let len = s.len() as u64;
    let count = (len + 0xFFF) / 0x1000;
    process.cr3.map_pages(
        virt,
        ptr,
        count,
        PageTableEntry::new().with_user(true).with_present(true),
    );
    sys_state
        .user_allocations
        .get_mut()
        .unwrap()
        .lock()
        .track(proc_id, virt, count);
    let msg = Message::new(
        uuid::Uuid::nil(),
        core::slice::from_raw_parts((ptr + USER_PHYS_VIRT_OFFSET) as *const _, len as _),
    );
    sys_state
        .user_allocations
        .get_mut()
        .unwrap()
        .lock()
        .track_message(msg.id, virt);
    process.messages.push_front(msg);
}

unsafe extern "C" fn syscall_handler(state: &mut RegisterState) {
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut scheduler = sys_state.scheduler.get_mut().unwrap().lock();

    let Ok(v) = SystemCall::try_from(state.rdi) else {
        state.rax = SystemCallStatus::UnknownRequest.into();
        return;
    };

    state.rax = match v {
        SystemCall::KPrint => 'a: {
            let s = core::slice::from_raw_parts(state.rsi as *const u8, state.rdx as usize);
            if s.as_ptr().is_null() {
                break 'a SystemCallStatus::MalformedData.into();
            }
            let Ok(s) = core::str::from_utf8(s) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            #[cfg(debug_assertions)]
            let mut serial = crate::sys::io::serial::SERIAL.lock();
            #[cfg(debug_assertions)]
            write!(serial, "{s}").unwrap();
            if let Some(terminal) = &mut sys_state.terminal {
                write!(terminal, "{s}").unwrap();
            }
            SystemCallStatus::Success.into()
        }
        SystemCall::ReceiveMessage => 'a: {
            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            let process = scheduler.processes.get_mut(&proc_id).unwrap();
            let Some(msg) = process.messages.pop_back() else {
                break 'a SystemCallStatus::DoNothing.into();
            };
            let (id_upper, id_lower) = msg.id.as_u64_pair();
            let (proc_id_upper, proc_id_lower) = msg.proc_id.as_u64_pair();
            state.rdi = id_upper;
            state.rsi = id_lower;
            state.rdx = proc_id_upper;
            state.rcx = proc_id_lower;
            state.r8 = msg.data.as_ptr() as u64;
            state.r9 = msg.data.len() as u64;
            SystemCallStatus::Success.into()
        }
        SystemCall::Exit => {
            let index = scheduler
                .threads
                .iter()
                .position(|v| v.id == scheduler.current_thread_id.unwrap())
                .unwrap();
            scheduler.threads.remove(index);
            scheduler.current_thread_id = None;
            state.rax = SystemCallStatus::Success.into();
            drop(scheduler);
            super::scheduler::schedule(state);
            return;
        }
        SystemCall::Skip => {
            state.rax = SystemCallStatus::Success.into();
            drop(scheduler);
            super::scheduler::schedule(state);
            return;
        }
        SystemCall::SendMessage => 'a: {
            let src = scheduler.current_thread_mut().unwrap().proc_id;
            let dest = uuid::Uuid::from_u64_pair(state.rsi, state.rdx);
            if dest.is_nil() {
                break 'a SystemCallStatus::MalformedData.into();
            }
            let Some(process) = scheduler.processes.get_mut(&dest) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            let addr = state.rcx + USER_PHYS_VIRT_OFFSET;
            let msg = Message::new(
                src,
                core::slice::from_raw_parts(addr as *const _, state.r8 as _),
            );
            sys_state
                .user_allocations
                .get_mut()
                .unwrap()
                .lock()
                .track_message(msg.id, addr);
            process.messages.push_front(msg);
            SystemCallStatus::Success.into()
        }
        SystemCall::RegisterProvider => 'a: {
            let provider = uuid::Uuid::from_u64_pair(state.rsi, state.rdx);
            if provider.is_nil() {
                break 'a SystemCallStatus::MalformedData.into();
            }
            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            if scheduler.providers.try_insert(provider, proc_id).is_err() {
                break 'a SystemCallStatus::InvalidRequest.into();
            }
            SystemCallStatus::Success.into()
        }
        SystemCall::GetProvidingProcess => 'a: {
            let provider = uuid::Uuid::from_u64_pair(state.rsi, state.rdx);
            if provider.is_nil() {
                break 'a SystemCallStatus::MalformedData.into();
            }
            let Some(proc_id) = scheduler.providers.get(&provider) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            let (hi, lo) = proc_id.as_u64_pair();
            state.rdi = hi;
            state.rsi = lo;
            SystemCallStatus::Success.into()
        }
        SystemCall::PortInByte => {
            let port = state.rsi as u16;
            state.rdi = u8::read(port) as u64;
            SystemCallStatus::Success.into()
        }
        SystemCall::PortInWord => {
            let port = state.rsi as u16;
            state.rdi = u16::read(port) as u64;
            SystemCallStatus::Success.into()
        }
        SystemCall::PortInDWord => {
            let port = state.rsi as u16;
            state.rdi = u32::read(port) as u64;
            SystemCallStatus::Success.into()
        }
        SystemCall::PortOutByte => {
            let port = state.rsi as u16;
            let value = state.rdx as u8;
            u8::write(port, value);
            SystemCallStatus::Success.into()
        }
        SystemCall::PortOutWord => {
            let port = state.rsi as u16;
            let value = state.rdx as u16;
            u16::write(port, value);
            SystemCallStatus::Success.into()
        }
        SystemCall::PortOutDWord => {
            let port = state.rsi as u16;
            let value = state.rdx as u32;
            u32::write(port, value);
            SystemCallStatus::Success.into()
        }
        SystemCall::RegisterIRQHandler => 'a: {
            let irq = state.rsi as u8;
            if state.rdx == 0 {
                break 'a SystemCallStatus::MalformedData.into();
            }

            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            if scheduler.irq_handlers.try_insert(irq, proc_id).is_err() {
                break 'a SystemCallStatus::InvalidRequest.into();
            }

            crate::driver::acpi::ioapic::wire_legacy_irq(irq, false);
            crate::driver::intrs::idt::set_handler(
                irq + 0x20,
                0,
                PrivilegeLevel::Supervisor,
                irq_handler,
                true,
                true,
            );
            SystemCallStatus::Success.into()
        }
        SystemCall::Allocate => {
            let size = state.rsi;
            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            let process = scheduler.processes.get_mut(&proc_id).unwrap();
            state.rdi = sys_state
                .user_allocations
                .get_mut()
                .unwrap()
                .lock()
                .allocate(proc_id, &mut process.cr3, size);
            SystemCallStatus::Success.into()
        }
        SystemCall::Free => {
            let ptr = state.rsi;
            sys_state
                .user_allocations
                .get_mut()
                .unwrap()
                .lock()
                .free(ptr);
            SystemCallStatus::Success.into()
        }
        SystemCall::Ack => {
            let id = uuid::Uuid::from_u64_pair(state.rsi, state.rdx);
            sys_state
                .user_allocations
                .get_mut()
                .unwrap()
                .lock()
                .free_message(id);
            SystemCallStatus::Success.into()
        }
    };
}

pub fn setup() {
    let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
    state
        .user_allocations
        .call_once(|| spin::Mutex::new(allocations::UserAllocationTracker::new()));
    crate::driver::intrs::idt::set_handler(
        249,
        1,
        PrivilegeLevel::User,
        syscall_handler,
        false,
        true,
    );
}
