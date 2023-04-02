// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::{string::String, vec::Vec};
use core::cell::SyncUnsafeCell;

use amd64::paging::pml4::PML4;
use hashbrown::HashMap;

use crate::{
    system::{
        gdt::{PrivilegeLevel, SegmentSelector},
        tss::TaskSegmentSelector,
        RegisterState,
    },
    timer::Timer,
};

static TSS: SyncUnsafeCell<TaskSegmentSelector> = SyncUnsafeCell::new(TaskSegmentSelector::new(0));

pub struct Scheduler {
    pub processes: HashMap<u64, super::Process>,
    pub threads: HashMap<u64, super::Thread>,
    pub current_tid: Option<u64>,
    pub current_pid: Option<u64>,
    pub kern_stack: Vec<u8>,
    pub providers: HashMap<u64, u64>,
    pub irq_handlers: HashMap<u8, u64>,
    pub message_sources: HashMap<u64, u64>,
    pub pid_gen: crate::utils::incr_id::IncrementalIDGen,
    pub tid_gen: crate::utils::incr_id::IncrementalIDGen,
    pub msg_id_gen: crate::utils::incr_id::IncrementalIDGen,
}

extern "C" fn idle() {
    crate::hlt_loop!();
}

pub unsafe extern "sysv64" fn schedule(state: &mut RegisterState) {
    let sys_state = &mut *crate::system::state::SYS_STATE.get();
    let mut this = sys_state.scheduler.as_ref().unwrap().lock();

    if let Some(old_thread) = this.current_thread_mut() {
        old_thread.regs = *state;
        if !old_thread.state.is_suspended() {
            old_thread.state = super::ThreadState::Inactive;
        }
    }

    if let Some(thread) = this.next_thread_mut() {
        *state = thread.regs;
        thread.state = super::ThreadState::Active;
        let pid = thread.pid;
        let tid = Some(thread.id);
        this.processes.get_mut(&pid).unwrap().cr3.set();
        this.current_tid = tid;
        this.current_pid = Some(pid);
    } else {
        *state = RegisterState {
            rip: idle as usize as _,
            cs: SegmentSelector::new(1, PrivilegeLevel::Supervisor).0.into(),
            rflags: 0x202,
            rsp: this.kern_stack.as_ptr() as u64 + this.kern_stack.len() as u64,
            ss: SegmentSelector::new(2, PrivilegeLevel::Supervisor).0.into(),
            ..Default::default()
        };
        sys_state.pml4.as_mut().unwrap().set();
        this.current_tid = None;
        this.current_pid = None;
    }
}

impl Scheduler {
    #[inline]
    pub fn new(timer: &impl Timer) -> Self {
        let kern_stack = vec![0; 0x14000];

        unsafe {
            let gdt = &mut *crate::system::gdt::GDT.get();
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
                in("ax") crate::system::gdt::SegmentSelector::new(5, crate::system::gdt::PrivilegeLevel::Supervisor).0,
                options(nostack, preserves_flags),
            );
        }

        let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
        state.lapic.as_ref().unwrap().setup_timer(timer);

        crate::intrs::idt::set_handler(128, 1, PrivilegeLevel::Supervisor, schedule, true, true);
        crate::acpi::ioapic::wire_legacy_irq(96, false);

        Self {
            processes: HashMap::new(),
            threads: HashMap::new(),
            current_tid: None,
            current_pid: None,
            kern_stack,
            providers: HashMap::new(),
            irq_handlers: HashMap::new(),
            message_sources: HashMap::new(),
            pid_gen: crate::utils::incr_id::IncrementalIDGen::new(),
            tid_gen: crate::utils::incr_id::IncrementalIDGen::new(),
            msg_id_gen: crate::utils::incr_id::IncrementalIDGen::new(),
        }
    }

    pub fn unmask() {
        crate::sti!();
        let state = unsafe { &*crate::system::state::SYS_STATE.get() };
        let lapic = state.lapic.as_ref().unwrap();
        lapic.write_timer(lapic.read_timer().with_mask(false));
        unsafe { core::arch::asm!("int 128", options(nostack, preserves_flags)) }
    }

    pub fn spawn_proc(&mut self, exec_data: &[u8]) -> &mut super::Thread {
        let exec = goblin::elf::Elf::parse(exec_data).unwrap();
        assert_eq!(exec.header.e_type, goblin::elf::header::ET_DYN);
        assert!(exec.is_64);
        assert_ne!(exec.entry, 0);

        let max_vaddr = exec
            .program_headers
            .iter()
            .map(|v| v.p_vaddr + v.p_memsz)
            .max()
            .unwrap();
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
            + tungstenkit::USER_PHYS_VIRT_OFFSET;
        for reloc in exec.dynrelas.iter() {
            let ptr = unsafe { &mut *data.as_mut_ptr().add(reloc.r_offset as _).cast::<u64>() };
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

        let pid = self.pid_gen.next();
        self.processes
            .insert(pid, super::Process::new(pid, String::new()));
        let proc = self.processes.get_mut(&pid).unwrap();
        unsafe { proc.cr3.map_higher_half() }
        proc.track_alloc(virt_addr, data.len() as _, Some(true));
        let tid = self.tid_gen.next();
        proc.tids.push(tid);
        self.threads.insert(
            tid,
            super::Thread::new(
                tid,
                pid,
                virt_addr + exec.entry,
                proc.allocate(super::STACK_SIZE),
            ),
        );

        self.threads.get_mut(&tid).unwrap()
    }

    pub fn current_thread_mut(&mut self) -> Option<&mut super::Thread> {
        self.threads.get_mut(&self.current_tid?)
    }

    pub fn current_process_mut(&mut self) -> Option<&mut super::Process> {
        self.processes.get_mut(&self.current_pid?)
    }

    pub fn next_thread_mut(&mut self) -> Option<&mut super::Thread> {
        let mut i = self.current_tid.map(|v| v + 1).unwrap_or_default() as usize;
        if i > self.threads.len() {
            i = 0;
        }

        self.threads
            .values_mut()
            .skip(i)
            .find(|v| v.state.is_inactive())
    }
}
