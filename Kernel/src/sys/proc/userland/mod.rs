// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::boxed::Box;
use core::fmt::Write;

use amd64::{
    io::port::PortIO,
    paging::{pml4::PML4, PageTableEntry},
};
use kernel::{KernelMessage, Message, SystemCall, SystemCallStatus};

use crate::sys::{gdt::PrivilegeLevel, RegisterState};

pub mod allocations;

pub const USER_PHYS_VIRT_OFFSET: u64 = 0xC0000000;

// This isn't meant to be user-accessible.
// It is meant to track the allocations so that they are deallocated when the process exits.
#[derive(Debug)]
pub struct UserPageTableLvl4(u64, amd64::paging::PageTable);

impl UserPageTableLvl4 {
    pub const fn new(proc_id: u64) -> Self {
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
        state
            .user_allocations
            .get_mut()
            .unwrap()
            .lock()
            .track(self.0, phys, 4096);
        phys
    }
}

unsafe extern "C" fn irq_handler(state: &mut RegisterState) {
    let irq = (state.int_num - 0x20) as u8;
    crate::driver::acpi::ioapic::set_irq_mask(irq, true);
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut scheduler = sys_state.scheduler.get_mut().unwrap().lock();
    let proc_id = *scheduler.irq_handlers.get(&irq).unwrap();
    let s = postcard::to_allocvec(&KernelMessage::IRQFired(irq))
        .unwrap()
        .leak();
    let ptr = s.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
    let virt = ptr + USER_PHYS_VIRT_OFFSET;
    let count = (s.len() as u64 + 0xFFF) / 0x1000;
    let mut user_allocations = sys_state.user_allocations.get_mut().unwrap().lock();
    user_allocations.track(proc_id, virt, s.len() as u64);
    let msg = Message::new(
        0,
        core::slice::from_raw_parts(virt as *const _, s.len() as _),
    );
    scheduler.message_sources.insert(msg.id, 0);
    let process = scheduler.processes.get_mut(&proc_id).unwrap();
    process.cr3.map_pages(
        virt,
        ptr,
        count,
        PageTableEntry::new().with_user(true).with_present(true),
    );
    user_allocations.track_message(msg.id, virt);
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
            write!(crate::sys::io::serial::SERIAL.lock(), "{s}").unwrap();
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
            state.rdi = msg.id;
            state.rsi = msg.proc_id;
            state.rdx = msg.data.as_ptr() as u64;
            state.rcx = msg.data.len() as u64;
            SystemCallStatus::Success.into()
        }
        SystemCall::Exit => {
            let id = scheduler.current_thread_id.unwrap();
            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            let index = scheduler.thread_ids.iter().position(|v| *v == id).unwrap();
            scheduler.threads.remove(&id);
            scheduler.thread_ids.remove(index);
            scheduler.current_thread_id = None;
            if !scheduler.threads.iter().any(|(_, v)| v.proc_id == proc_id) {
                sys_state
                    .user_allocations
                    .get_mut()
                    .unwrap()
                    .lock()
                    .free_proc(proc_id);
            }
            drop(scheduler);
            super::scheduler::schedule(state);
            return;
        }
        SystemCall::Skip => {
            drop(scheduler);
            super::scheduler::schedule(state);
            return;
        }
        SystemCall::SendMessage => 'a: {
            let src = scheduler.current_thread_mut().unwrap().proc_id;
            if !scheduler.processes.contains_key(&state.rsi) {
                break 'a SystemCallStatus::MalformedData.into();
            }
            let addr = state.rcx + USER_PHYS_VIRT_OFFSET;
            let msg = Message::new(
                src,
                core::slice::from_raw_parts(addr as *const _, state.r8 as _),
            );
            scheduler.message_sources.insert(msg.id, src);
            let Some(process) = scheduler.processes.get_mut(&state.rsi) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
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
            if scheduler.providers.contains_key(&state.rsi) {
                break 'a SystemCallStatus::MalformedData.into();
            }
            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            if scheduler.providers.try_insert(state.rsi, proc_id).is_err() {
                break 'a SystemCallStatus::InvalidRequest.into();
            }
            SystemCallStatus::Success.into()
        }
        SystemCall::GetProvidingProcess => 'a: {
            if !scheduler.providers.contains_key(&state.rsi) {
                break 'a SystemCallStatus::MalformedData.into();
            }
            let Some(&proc_id) = scheduler.providers.get(&state.rsi) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            state.rdi = proc_id;
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
            let addr = sys_state
                .user_allocations
                .get_mut()
                .unwrap()
                .lock()
                .allocate(proc_id, size);

            process.cr3.map_pages(
                addr,
                addr - USER_PHYS_VIRT_OFFSET,
                (size + 0xFFF) / 0x1000,
                PageTableEntry::new()
                    .with_user(true)
                    .with_writable(true)
                    .with_present(true),
            );

            core::ptr::write_bytes(addr as *mut u8, 0, ((size + 0xFFF) / 0x1000 * 0x1000) as _);

            state.rdi = addr;
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
        SystemCall::Ack => 'a: {
            if state.rsi == 0 {
                break 'a SystemCallStatus::MalformedData.into();
            }
            let Some(&src) = scheduler.message_sources.get(&state.rsi) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            let mut user_allocations = sys_state.user_allocations.get_mut().unwrap().lock();
            if src == 0 {
                let addr = *user_allocations
                    .message_allocations
                    .get(&state.rsi)
                    .unwrap();
                let size = user_allocations.allocations.get(&addr).unwrap().1;
                let data = addr as *const u8;
                let msg: KernelMessage =
                    postcard::from_bytes(core::slice::from_raw_parts(data, size as _)).unwrap();
                let KernelMessage::IRQFired(irq) = msg;
                crate::driver::acpi::ioapic::set_irq_mask(irq, false);
            }
            user_allocations.free_message(state.rsi);
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
