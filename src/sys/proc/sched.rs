use alloc::collections::VecDeque;
use core::{cell::SyncUnsafeCell, fmt::Write, mem::MaybeUninit};

use amd64::paging::pml4::PML4;

use crate::{
    driver::{
        keyboard::ps2::PS2Ctl,
        pci::{PCICfgOffset, PCIController},
        timer::Timer,
    },
    sys::{tss::TaskSegmentSelector, RegisterState},
};

static TSS: SyncUnsafeCell<MaybeUninit<TaskSegmentSelector>> =
    SyncUnsafeCell::new(MaybeUninit::uninit());

pub struct Scheduler {
    pub processes: VecDeque<super::Process>,
    pub first_launch: bool,
}

unsafe extern "sysv64" fn schedule(state: &mut RegisterState) {
    let sys_state = crate::sys::state::SYS_STATE.get().as_mut().unwrap();
    let mut this = sys_state.scheduler.assume_init_mut().lock();

    if this.first_launch {
        this.first_launch = false;
    } else {
        let old_thread = this.current_thread();
        old_thread.regs = *state;
        old_thread.state = super::ThreadState::Inactive;
    }

    let thread = this.find_next_thread();
    *state = thread.regs;
    thread.state = super::ThreadState::Active;
    *(*TSS.get()).assume_init_mut() =
        TaskSegmentSelector::new(thread.kern_rsp.as_ptr() as u64 + thread.kern_rsp.len() as u64);

    this.processes[0].cr3.set();
}

fn test_thread1() -> ! {
    unsafe {
        loop {
            writeln!(
                &mut crate::sys::io::serial::SERIAL.lock(),
                "hi from thread 0"
            )
            .unwrap();

            core::arch::asm!("hlt");
        }
    }
}

fn test_thread2() -> ! {
    let state = unsafe { &mut *crate::sys::state::SYS_STATE.get() };
    let acpi = unsafe { state.acpi.assume_init_mut() };

    let pci = PCIController::new(acpi.find("MCFG"));
    let ac97 = pci
        .find(move |dev| unsafe { dev.cfg_read16::<_, u16>(PCICfgOffset::ClassCode) == 0x0401 })
        .map(|v| unsafe {
            (*crate::driver::audio::ac97::INSTANCE.get())
                .write(crate::driver::audio::ac97::AC97::new(&v))
        });

    if let Some(terminal) = &mut state.terminal {
        let ps2ctl = PS2Ctl::new();
        ps2ctl.init();
        unsafe {
            (*crate::driver::keyboard::ps2::INSTANCE.get()).write(ps2ctl);
        }

        crate::terminal_loop::terminal_loop(acpi, &pci, terminal, ac97);
    } else {
        loop {
            unsafe { core::arch::asm!("hlt") }
        }
    }
}

impl Scheduler {
    pub fn new(timer: &impl Timer) -> Self {
        let mut processes = VecDeque::new();
        let mut kern_proc = super::Process::new(0, "", "");
        let kern_thread = super::Thread::new(0, test_thread1 as usize);
        unsafe {
            *(*TSS.get()).assume_init_mut() = TaskSegmentSelector::new(
                kern_thread.kern_rsp.as_ptr() as u64 + kern_thread.kern_rsp.len() as u64,
            );
            let entries = &mut *crate::sys::gdt::ENTRIES.get();
            let tss = (*TSS.get()).as_ptr() as u64;
            entries[3].set_base((tss & 0xFFFF_FFFF) as u32);
            entries[3].attrs.set_present(true);
            entries[4].limit_low = ((tss >> 32) & 0xFFFF) as u16;
            entries[4].base_low = (tss >> 48) as u16;

            core::arch::asm!(
                "ltr ax",
                in("ax") crate::sys::gdt::SegmentSelector::new(3, crate::sys::gdt::PrivilegeLevel::Supervisor).0,
            );
        }
        kern_proc.threads.push_back(kern_thread);

        let kern_thread = super::Thread::new(1, test_thread2 as usize);
        kern_proc.threads.push_back(kern_thread);
        processes.push_back(kern_proc);

        let lapic = unsafe {
            (*crate::sys::state::SYS_STATE.get())
                .lapic
                .assume_init_ref()
        };

        lapic.setup_timer(timer);

        crate::driver::intrs::idt::set_handler(128, schedule, true, true);
        crate::driver::acpi::ioapic::wire_legacy_irq(96, false);

        Self {
            processes,
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

    pub fn current_thread(&mut self) -> &mut super::Thread {
        self.processes[0].threads.front_mut().unwrap()
    }

    pub fn find_next_thread(&mut self) -> &mut super::Thread {
        let proc = &mut self.processes[0];
        let old = proc.threads.pop_front().unwrap();
        proc.threads.push_back(old);
        proc.threads.front_mut().unwrap()
    }
}
