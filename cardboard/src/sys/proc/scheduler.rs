// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;
use core::cell::SyncUnsafeCell;

use amd64::paging::{pml4::PML4, PageTableEntry};
use hashbrown::HashMap;

use crate::{
    driver::timer::Timer,
    sys::{gdt::PrivilegeLevel, tss::TaskSegmentSelector, RegisterState},
};

static TSS: SyncUnsafeCell<TaskSegmentSelector> = SyncUnsafeCell::new(TaskSegmentSelector::new(0));

pub struct Scheduler {
    pub processes: HashMap<uuid::Uuid, super::Process>,
    pub threads: Vec<super::Thread>,
    pub current_thread_id: Option<uuid::Uuid>,
    pub kern_stack: Vec<u8>,
    pub providers: HashMap<uuid::Uuid, uuid::Uuid>,
    pub irq_handlers: HashMap<u8, uuid::Uuid>,
}

pub unsafe extern "C" fn schedule(state: &mut RegisterState) {
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut this = sys_state.scheduler.get_mut().unwrap().lock();

    if let Some(old_thread) = this.current_thread_mut() {
        old_thread.regs = *state;
        old_thread.state = super::ThreadState::Inactive;
    }

    let thread = this.next_thread_mut().unwrap();
    *state = thread.regs;
    thread.state = super::ThreadState::Active;
    let proc_id = thread.proc_id;
    let new = Some(thread.id);
    this.processes.get_mut(&proc_id).unwrap().cr3.set();

    this.current_thread_id = new;
}

impl Scheduler {
    pub fn new(timer: &impl Timer) -> Self {
        let kern_stack = vec![0; 0x14000];

        unsafe {
            let gdt = &mut *crate::sys::gdt::GDT.get();
            (*TSS.get()) =
                TaskSegmentSelector::new(kern_stack.as_ptr() as u64 + kern_stack.len() as u64);
            let tss_addr = TSS.get() as u64;
            gdt.task_segment.base_low = tss_addr as u16;
            gdt.task_segment.base_middle = (tss_addr >> 16) as u8;
            gdt.task_segment.attrs.set_present(true);
            gdt.task_segment.base_high = (tss_addr >> 24) as u8;
            gdt.task_segment.base_upper = (tss_addr >> 32) as u32;

            core::arch::asm!(
                "ltr ax",
                in("ax") crate::sys::gdt::SegmentSelector::new(5, crate::sys::gdt::PrivilegeLevel::Supervisor).0,
            );
        }

        let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
        let lapic = state.lapic.get_mut().unwrap();

        lapic.setup_timer(timer);

        crate::driver::intrs::idt::set_handler(
            128,
            1,
            PrivilegeLevel::Supervisor,
            schedule,
            true,
            true,
        );
        crate::driver::acpi::ioapic::wire_legacy_irq(96, false);

        Self {
            processes: HashMap::new(),
            threads: Vec::new(),
            current_thread_id: None,
            kern_stack,
            providers: HashMap::new(),
            irq_handlers: HashMap::new(),
        }
    }

    pub fn unmask() {
        crate::sti!();
        let state = unsafe { crate::sys::state::SYS_STATE.get().as_ref().unwrap() };
        let lapic = state.lapic.get().unwrap();
        lapic.write_timer(lapic.read_timer().with_mask(false));
    }

    pub fn spawn_proc(&mut self, exec_data: &[u8]) {
        let exec = goblin::elf::Elf::parse(exec_data).unwrap();
        assert_eq!(exec.header.e_type, goblin::elf::header::ET_DYN);
        assert!(exec.is_64);
        assert_ne!(exec.entry, 0);

        let max_vaddr = exec
            .program_headers
            .iter()
            .map(|v| v.p_vaddr + v.p_memsz)
            .max()
            .unwrap_or_default();
        let mut data = vec![0; max_vaddr as usize];
        for hdr in exec
            .program_headers
            .iter()
            .filter(|v| v.p_type == goblin::elf::program_header::PT_LOAD)
        {
            let fsz = hdr.p_filesz as usize;
            let foff = hdr.p_offset as usize;
            let ext_vaddr = hdr.p_vaddr as usize;
            data[ext_vaddr..ext_vaddr + fsz].copy_from_slice(&exec_data[foff..foff + fsz]);
        }

        let data = data.leak();
        let virt_addr = data.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET
            + super::userland::USER_PHYS_VIRT_OFFSET;
        for reloc in exec.dynrelas.iter() {
            let ptr = unsafe {
                ((data.as_ptr() as u64 + reloc.r_offset) as *mut u64)
                    .as_mut()
                    .unwrap()
            };
            let target = reloc.r_addend.map_or_else(
                || virt_addr + *ptr,
                |addend| {
                    if addend.is_negative() {
                        virt_addr - addend.wrapping_abs() as u64
                    } else {
                        virt_addr + addend as u64
                    }
                },
            );
            *ptr = target;
        }
        let rip = virt_addr + exec.entry;
        let proc_id = uuid::Uuid::new_v4();
        let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
        let count = (data.len() as u64 + 0xFFF) / 0x1000;
        state
            .user_allocations
            .get_mut()
            .unwrap()
            .lock()
            .track(proc_id, virt_addr, count);
        self.processes
            .insert(proc_id, super::Process::new(proc_id, "", ""));
        let proc = self.processes.get_mut(&proc_id).unwrap();
        unsafe {
            proc.cr3.map_higher_half();
        }
        let thread = super::Thread::new(proc_id, rip);
        unsafe {
            proc.cr3.map_pages(
                virt_addr,
                virt_addr - super::userland::USER_PHYS_VIRT_OFFSET,
                count,
                PageTableEntry::new()
                    .with_user(true)
                    .with_writable(true)
                    .with_present(true),
            );
            let stack_addr = thread.stack.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
            proc.cr3.map_pages(
                stack_addr + super::userland::USER_PHYS_VIRT_OFFSET,
                stack_addr,
                (thread.stack.len() as u64 + 0xFFF) / 0x1000,
                PageTableEntry::new()
                    .with_user(true)
                    .with_writable(true)
                    .with_present(true),
            );
        }
        self.threads.push(thread);
    }

    pub fn current_thread_mut(&mut self) -> Option<&mut super::Thread> {
        let id = self.current_thread_id?;
        self.threads.iter_mut().find(|v| v.id == id)
    }

    pub fn next_thread_mut(&mut self) -> Option<&mut super::Thread> {
        let mut i = self
            .current_thread_id
            .and_then(|id| self.threads.iter().position(|v| v.id == id).map(|i| i + 1))
            .unwrap_or_default();
        if i >= self.threads.len() {
            i = 0;
        }

        self.threads[i..]
            .iter_mut()
            .find(|v| v.state == super::ThreadState::Inactive)
    }
}
