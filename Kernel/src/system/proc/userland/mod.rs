// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::boxed::Box;
use core::fmt::Write;

use amd64::{
    io::port::PortIO,
    paging::{pml4::PML4, PageTableEntry},
};
use driver_core::syscall::{KernelMessage, Message, SystemCall, SystemCallStatus};

use crate::system::{gdt::PrivilegeLevel, RegisterState};

pub mod allocations;

// This isn't meant to be user-accessible.
// It is meant to track the allocations so that they are deallocated when the process exits.
#[repr(transparent)]
pub struct UserPageTableLvl4(amd64::paging::PageTable);

impl UserPageTableLvl4 {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(amd64::paging::PageTable::new())
    }
}

impl PML4 for UserPageTableLvl4 {
    const VIRT_OFF: u64 = amd64::paging::PHYS_VIRT_OFFSET;

    fn get_entry(&mut self, offset: u64) -> &mut amd64::paging::PageTableEntry {
        &mut self.0.entries[offset as usize]
    }

    fn alloc_entry(&self) -> u64 {
        let phys = Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as u64
            - amd64::paging::PHYS_VIRT_OFFSET;
        let state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
        let sys_state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
        let scheduler = sys_state.scheduler.get_mut().unwrap().get_mut();
        state.user_allocations.get_mut().unwrap().lock().track(
            scheduler.current_thread_id.unwrap(),
            phys + driver_core::USER_PHYS_VIRT_OFFSET,
            4096,
        );
        phys
    }
}

unsafe extern "C" fn irq_handler(state: &mut RegisterState) {
    let irq = (state.int_num - 0x20) as u8;
    crate::acpi::ioapic::set_irq_mask(irq, true);
    let sys_state = crate::system::state::SYS_STATE.get().as_mut().unwrap();
    let mut scheduler = sys_state.scheduler.get_mut().unwrap().lock();
    let proc_id = *scheduler.irq_handlers.get(&irq).unwrap();
    let s = postcard::to_allocvec(&KernelMessage::IRQFired(irq))
        .unwrap()
        .leak();
    let ptr = s.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
    let virt = ptr + driver_core::USER_PHYS_VIRT_OFFSET;
    let count = (s.len() as u64 + 0xFFF) / 0x1000;
    let mut user_allocations = sys_state.user_allocations.get_mut().unwrap().lock();
    user_allocations.track(proc_id, virt, s.len() as u64);
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
        PageTableEntry::new().with_user(true).with_present(true),
    );
    user_allocations.track_msg(msg.id, virt);
    process.messages.push_front(msg);
}

unsafe extern "C" fn syscall_handler(state: &mut RegisterState) {
    let sys_state = crate::system::state::SYS_STATE.get().as_mut().unwrap();
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
            write!(crate::system::serial::SERIAL.lock(), "{s}").unwrap();
            if let Some(terminal) = &mut sys_state.terminal {
                write!(terminal, "{s}").unwrap();
            }
            SystemCallStatus::Success.into()
        }
        SystemCall::ReceiveMessage => {
            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            let process = scheduler.processes.get_mut(&proc_id).unwrap();
            if let Some(msg) = process.messages.pop_back() {
                state.rdi = msg.id;
                state.rsi = msg.proc_id;
                state.rdx = msg.data.as_ptr() as u64;
                state.rcx = msg.data.len() as u64;
            } else {
                state.rdi = 0;
            }
            SystemCallStatus::Success.into()
        }
        SystemCall::Exit => {
            let id = scheduler.current_thread_id.unwrap();
            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            let index = scheduler.thread_ids.iter().position(|v| *v == id).unwrap();
            scheduler.threads.remove(&id);
            scheduler.thread_ids.remove(index);
            scheduler.thread_id_gen.free(id);
            scheduler.current_thread_id = None;
            if !scheduler.threads.iter().any(|(_, v)| v.proc_id == proc_id) {
                sys_state
                    .user_allocations
                    .get_mut()
                    .unwrap()
                    .lock()
                    .free_proc(proc_id);
                scheduler.processes.remove(&proc_id);
                scheduler.proc_id_gen.free(proc_id);
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
            let addr = state.rcx;
            let msg = Message::new(
                scheduler.message_id_gen.next(),
                src,
                core::slice::from_raw_parts(addr as *const _, state.r8 as _),
            );
            scheduler.message_sources.insert(msg.id, src);
            let process = scheduler.processes.get_mut(&state.rsi).unwrap();
            sys_state
                .user_allocations
                .get_mut()
                .unwrap()
                .lock()
                .track_msg(msg.id, addr);
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
            state.rdi = if let Some(&proc_id) = scheduler.providers.get(&state.rsi) {
                proc_id
            } else {
                0
            };
            SystemCallStatus::Success.into()
        }
        SystemCall::PortIn => 'a: {
            let port = state.rsi as u16;
            let Ok(access_size) = driver_core::syscall::AccessSize::try_from(state.rdx) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            state.rdi = match access_size {
                driver_core::syscall::AccessSize::Byte => u8::read(port) as u64,
                driver_core::syscall::AccessSize::Word => u16::read(port) as u64,
                driver_core::syscall::AccessSize::DWord => u32::read(port) as u64,
            };
            SystemCallStatus::Success.into()
        }
        SystemCall::PortOut => 'a: {
            let port = state.rsi as u16;
            let Ok(access_size) = driver_core::syscall::AccessSize::try_from(state.rcx) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            match access_size {
                driver_core::syscall::AccessSize::Byte => u8::write(port, state.rdx as u8),
                driver_core::syscall::AccessSize::Word => u16::write(port, state.rdx as u16),
                driver_core::syscall::AccessSize::DWord => u32::write(port, state.rdx as u32),
            };
            SystemCallStatus::Success.into()
        }
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
                addr - driver_core::USER_PHYS_VIRT_OFFSET,
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
            let id = state.rsi;

            let Some(&src) = scheduler.message_sources.get(&id) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            let mut user_allocations = sys_state.user_allocations.get_mut().unwrap().lock();
            if src == 0 {
                let addr = *user_allocations.message_allocations.get(&id).unwrap();
                let size = user_allocations.allocations.get(&addr).unwrap().1;
                let data = addr as *const u8;
                let msg: KernelMessage =
                    postcard::from_bytes(core::slice::from_raw_parts(data, size as _)).unwrap();
                let KernelMessage::IRQFired(irq) = msg;
                crate::acpi::ioapic::set_irq_mask(irq, false);
            }
            user_allocations.free_msg(id);
            scheduler.message_id_gen.free(id);
            SystemCallStatus::Success.into()
        }
        SystemCall::GetRegistryEntryInfo => 'a: {
            let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
            let registry_tree_index = sys_state.registry_tree_index.get().unwrap().lock();
            let Some(registry_entry) = registry_tree_index.get(&state.rsi) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            let Ok(info_type) = driver_core::syscall::BCRegistryEntryInfoType::try_from(state.rdx) else {
                break 'a SystemCallStatus::MalformedData.into();
            };
            let data = match info_type {
                driver_core::syscall::BCRegistryEntryInfoType::Parent => {
                    postcard::to_allocvec(&registry_entry.parent)
                }
                driver_core::syscall::BCRegistryEntryInfoType::PropertyNamed => {
                    let Ok(k) = core::str::from_utf8(core::slice::from_raw_parts(
                        state.rcx as *const u8,
                        state.r8 as usize,
                    )) else {
                        break 'a SystemCallStatus::MalformedData.into();
                    };
                    postcard::to_allocvec(&registry_entry.properties.get(k))
                }
            }
            .unwrap()
            .leak();
            let ptr = data.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
            let virt = ptr + driver_core::USER_PHYS_VIRT_OFFSET;
            let count = (data.len() as u64 + 0xFFF) / 0x1000;
            let mut user_allocations = sys_state.user_allocations.get_mut().unwrap().lock();
            user_allocations.track(proc_id, virt, data.len() as u64);

            let process = scheduler.processes.get_mut(&proc_id).unwrap();
            process.cr3.map_pages(
                virt,
                ptr,
                count,
                PageTableEntry::new().with_user(true).with_present(true),
            );
            state.rdi = virt;
            state.rsi = data.len() as u64;
            SystemCallStatus::Success.into()
        }
    };
}

pub fn setup() {
    let state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
    state
        .user_allocations
        .call_once(|| spin::Mutex::new(allocations::UserAllocationTracker::new()));
    crate::intrs::idt::set_handler(249, 1, PrivilegeLevel::User, syscall_handler, false, true);
}
