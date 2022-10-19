use alloc::{collections::VecDeque, vec::Vec};
use core::cell::SyncUnsafeCell;

use amd64::paging::{pml4::PML4, PageTableEntry};
use hashbrown::HashMap;

use crate::{
    driver::timer::Timer,
    sys::{tss::TaskSegmentSelector, RegisterState},
};

static TSS: SyncUnsafeCell<TaskSegmentSelector> = SyncUnsafeCell::new(TaskSegmentSelector::new(0));

pub struct Scheduler {
    pub processes: HashMap<uuid::Uuid, super::Process>,
    pub threads: VecDeque<super::Thread>,
    pub first_launch: bool,
}

unsafe extern "sysv64" fn schedule(state: &mut RegisterState) {
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut this = sys_state.scheduler.assume_init_mut().lock();

    if this.first_launch {
        this.first_launch = false;
    } else {
        let old_thread = this.current_thread().unwrap();
        old_thread.regs = *state;
        old_thread.state = super::ThreadState::Inactive;
    }
    let mut thread = this.find_next_thread().unwrap();
    while thread.state == super::ThreadState::Blocked {
        thread = this.find_next_thread().unwrap();
    }
    *TSS.get() = TaskSegmentSelector::new(
        thread.kern_stack.as_ptr() as u64 + thread.kern_stack.len() as u64,
    );
    *state = thread.regs;
    thread.state = super::ThreadState::Active;
    let proc_uuid = thread.proc_uuid;
    this.processes.get_mut(&proc_uuid).unwrap().cr3.set();
}

impl Scheduler {
    pub fn new(timer: &impl Timer) -> Self {
        unsafe {
            let gdt = &mut *crate::sys::gdt::GDT.get();
            let tss = TSS.get() as u64;
            gdt.task_segment.base_low = (tss & 0xFFFF) as u16;
            gdt.task_segment.base_middle = ((tss >> 16) & 0xFF) as u8;
            gdt.task_segment.attrs.set_present(true);
            gdt.task_segment.base_high = ((tss >> 24) & 0xFF) as u8;
            gdt.task_segment.base_upper = (tss >> 32) as u32;

            core::arch::asm!(
                "ltr ax",
                in("ax") crate::sys::gdt::SegmentSelector::new(5, crate::sys::gdt::PrivilegeLevel::Supervisor).0,
            );
        }

        let lapic = unsafe {
            (*crate::sys::state::SYS_STATE.get())
                .lapic
                .assume_init_ref()
        };

        lapic.setup_timer(timer);

        crate::driver::intrs::idt::set_handler(128, 1, schedule, true, true);
        crate::driver::acpi::ioapic::wire_legacy_irq(96, false);

        Self {
            processes: HashMap::new(),
            threads: VecDeque::new(),
            first_launch: true,
        }
    }

    pub fn start() {
        unsafe {
            let lapic = (*crate::sys::state::SYS_STATE.get())
                .lapic
                .assume_init_ref();
            lapic.write_timer(lapic.read_timer().with_mask(false));
        }
    }

    pub fn spawn_proc(&mut self, exec_data: &[u8]) {
        let exec = goblin::elf::Elf::parse(exec_data).unwrap();
        assert_eq!(exec.header.e_type, goblin::elf::header::ET_DYN);
        assert!(exec.is_64);
        assert_ne!(exec.entry, 0);

        let mut data = Vec::new();
        for hdr in exec
            .program_headers
            .iter()
            .filter(|v| v.p_type == goblin::elf::program_header::PT_LOAD)
        {
            let max_vaddr = (hdr.p_vaddr + hdr.p_memsz).try_into().unwrap();
            if data.len() < max_vaddr {
                data.resize(max_vaddr, 0u8);
            }
            let fsz: usize = hdr.p_filesz.try_into().unwrap();
            let foff: usize = hdr.p_offset.try_into().unwrap();
            let ext_vaddr: usize = hdr.p_vaddr.try_into().unwrap();
            data[ext_vaddr..ext_vaddr + fsz].copy_from_slice(&exec_data[foff..foff + fsz]);
        }

        let data = data.leak();
        let phys_addr = data.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
        let rip = phys_addr + exec.entry;
        let proc_uuid = uuid::Uuid::new_v4();
        let mut proc = super::Process::new("", "");
        let thread = super::Thread::new(proc_uuid, rip);
        unsafe {
            if self.first_launch {
                *TSS.get() = TaskSegmentSelector::new(
                    thread.kern_stack.as_ptr() as u64 + thread.kern_stack.len() as u64,
                );
            }

            proc.cr3.map_pages(
                phys_addr,
                phys_addr,
                (data.len() as u64 + 0xFFF) / 0x1000,
                PageTableEntry::new()
                    .with_user(true)
                    .with_writable(true)
                    .with_present(true),
            );
            let stack_addr = thread.stack.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
            proc.cr3.map_pages(
                stack_addr,
                stack_addr,
                (0x14000 + 0xFFF) / 0x1000,
                PageTableEntry::new()
                    .with_user(true)
                    .with_writable(true)
                    .with_present(true),
            );
        }
        self.processes.insert(proc_uuid, proc);
        self.threads.push_back(thread);
    }

    pub fn current_thread(&mut self) -> Option<&mut super::Thread> {
        self.threads.front_mut()
    }

    pub fn find_next_thread(&mut self) -> Option<&mut super::Thread> {
        let old = self.threads.pop_front()?;
        self.threads.push_back(old);
        self.threads.front_mut()
    }
}
