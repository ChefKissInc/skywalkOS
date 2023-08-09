// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::{string::String, vec::Vec};
use core::{cell::SyncUnsafeCell, ops::ControlFlow};

use fireworkkit::{
    msg::{KernelMessage, Message},
    TerminationReason,
};
use hashbrown::HashMap;

use crate::{
    system::{
        gdt::{PrivilegeLevel, SegmentSelector},
        proc::AllocationType,
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
    pub irq_handlers: HashMap<u8, u64>,
    pub message_sources: HashMap<u64, u64>,
    pub pid_gen: crate::utils::incr_id::IncrementalIDGen,
    pub tid_gen: crate::utils::incr_id::IncrementalIDGen,
    pub msg_id_gen: crate::utils::incr_id::IncrementalIDGen,
}

unsafe extern "sysv64" fn irq_handler(state: &mut RegisterState) {
    let irq = (state.int_num - 0x20) as u8;
    crate::acpi::ioapic::set_irq_mask(irq, true);
    let mut this = (*crate::system::state::SYS_STATE.get())
        .scheduler
        .as_ref()
        .unwrap()
        .lock();
    let pid = this.irq_handlers.get(&irq).copied().unwrap();
    let s: &mut [u8] = postcard::to_allocvec(&KernelMessage::IRQFired(irq))
        .unwrap()
        .leak();

    let virt = this
        .processes
        .get_mut(&pid)
        .unwrap()
        .track_kernelside_alloc(s.as_ptr() as _, s.len() as _);

    let msg = Message::new(
        this.msg_id_gen.next(),
        0,
        core::slice::from_raw_parts(virt as *const _, s.len() as _),
    );
    this.message_sources.insert(msg.id, 0);
    let process = this.processes.get_mut(&pid).unwrap();
    process.track_msg(msg.id, virt);

    let tids = process.thread_ids.clone();
    if super::userland::handlers::msg::handle_new(&mut this, pid, tids, msg).is_break() {
        this.schedule(state);
    }
}

extern "C" fn idle() {
    crate::hlt_loop!();
}

pub unsafe extern "sysv64" fn schedule(state: &mut RegisterState) {
    (*crate::system::state::SYS_STATE.get())
        .scheduler
        .as_ref()
        .unwrap()
        .lock()
        .schedule(state);
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
                in("ax") SegmentSelector::new(5, crate::system::gdt::PrivilegeLevel::Supervisor).0,
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
        unsafe { core::arch::asm!("int 128", options(nomem, nostack, preserves_flags)) }
    }

    pub fn spawn_proc(&mut self, path: String, exec_data: &[u8]) -> &mut super::Thread {
        let exec = elf::ElfBytes::<elf::endian::NativeEndian>::minimal_parse(exec_data).unwrap();
        assert_eq!(exec.ehdr.e_type, elf::abi::ET_DYN);
        assert_eq!(exec.ehdr.class, elf::file::Class::ELF64);
        assert_ne!(exec.ehdr.e_entry, 0);

        let max_vaddr = exec
            .segments()
            .unwrap()
            .iter()
            .map(|v| v.p_vaddr + v.p_memsz)
            .max()
            .unwrap();
        let data = vec![0; max_vaddr as usize].leak();
        for hdr in exec
            .segments()
            .unwrap()
            .iter()
            .filter(|v| v.p_type == elf::abi::PT_LOAD)
        {
            let fsz = hdr.p_filesz as usize;
            let foff = hdr.p_offset as usize;
            let ext_vaddr = hdr.p_vaddr as usize;
            data[ext_vaddr..ext_vaddr + fsz].copy_from_slice(&exec_data[foff..foff + fsz]);
        }

        let virt_addr = data.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET
            + fireworkkit::USER_PHYS_VIRT_OFFSET;
        for v in exec.section_headers().unwrap().iter() {
            let Ok(relas) = exec.section_data_as_relas(&v) else {
                continue;
            };
            for reloc in relas {
                let ptr = unsafe { &mut *data.as_mut_ptr().add(reloc.r_offset as _).cast::<u64>() };
                match reloc.r_type {
                    elf::abi::R_X86_64_NONE => {}
                    elf::abi::R_X86_64_RELATIVE => {
                        *ptr = virt_addr + reloc.r_addend.wrapping_abs() as u64
                    }
                    v => unimplemented!("{v:#X?}"),
                }
            }
        }

        let pid = self.pid_gen.next();
        let proc = self
            .processes
            .try_insert(pid, super::Process::new(pid, path, virt_addr))
            .unwrap();
        unsafe { proc.cr3.lock().map_higher_half() }
        proc.track_alloc(virt_addr, data.len() as _, AllocationType::Writable);
        let tid = self.tid_gen.next();
        let stack_addr = proc.allocate(super::STACK_SIZE).0;
        let thread = proc.new_thread(tid, virt_addr + exec.ehdr.e_entry, stack_addr);
        self.threads.try_insert(tid, thread).unwrap()
    }

    pub fn current_thread_mut(&mut self) -> Option<&mut super::Thread> {
        self.threads.get_mut(&self.current_tid?)
    }

    pub fn current_process(&self) -> Option<&super::Process> {
        self.processes.get(&self.current_pid?)
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

    pub unsafe fn schedule(&mut self, state: &mut RegisterState) {
        if let Some(old_thread) = self.current_thread_mut() {
            old_thread.regs = *state;
            if !old_thread.state.is_suspended() {
                old_thread.state = super::ThreadState::Inactive;
            }
        }

        let Some(thread) = self.next_thread_mut() else {
            *state = RegisterState {
                rip: idle as usize as _,
                cs: SegmentSelector::new(1, PrivilegeLevel::Supervisor).into(),
                rflags: 0x202,
                rsp: self.kern_stack.as_ptr() as u64 + self.kern_stack.len() as u64,
                ss: SegmentSelector::new(2, PrivilegeLevel::Supervisor).into(),
                ..Default::default()
            };
            (*crate::system::state::SYS_STATE.get())
                .pml4
                .as_ref()
                .unwrap()
                .lock()
                .set_cr3();
            self.current_tid = None;
            self.current_pid = None;
            return;
        };

        *state = thread.regs;
        thread.state = super::ThreadState::Active;
        let pid = thread.pid;
        let tid = Some(thread.id);
        self.processes.get_mut(&pid).unwrap().cr3.lock().set_cr3();
        self.current_tid = tid;
        self.current_pid = Some(pid);
    }

    pub fn register_irq(
        &mut self,
        state: &RegisterState,
    ) -> ControlFlow<Option<TerminationReason>> {
        let irq = state.rsi as u8;
        if irq > 0xDF {
            return ControlFlow::Break(Some(TerminationReason::MalformedArgument));
        }
        let pid = self.current_pid.unwrap();
        if self.irq_handlers.try_insert(irq, pid).is_err() {
            return ControlFlow::Break(Some(TerminationReason::AlreadyExists));
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

        ControlFlow::Continue(())
    }

    pub fn thread_teardown(&mut self) -> ControlFlow<Option<TerminationReason>> {
        let id = self.current_tid.take().unwrap();
        self.threads.remove(&id);
        self.tid_gen.free(id);

        let proc = self.current_process_mut().unwrap();
        proc.thread_ids.remove(&id);
        if proc.thread_ids.is_empty() {
            let pid = self.current_pid.take().unwrap();
            self.processes.remove(&pid);
            self.pid_gen.free(pid);
        }

        ControlFlow::Break(None)
    }

    pub fn process_teardown(&mut self) {
        self.current_tid = None;
        let pid = self.current_pid.take().unwrap();
        let proc = self.processes.remove(&pid).unwrap();
        for tid in &proc.thread_ids {
            self.threads.remove(tid);
            self.tid_gen.free(*tid);
        }
        self.pid_gen.free(pid);
    }
}
