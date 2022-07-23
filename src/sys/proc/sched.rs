use alloc::collections::VecDeque;
use core::{
    cell::SyncUnsafeCell,
    /* fmt::Write, */ mem::{size_of, MaybeUninit},
};

use amd64::paging::{pml4::PML4, PageTableEntry};
use log::info;

use crate::{
    driver::{
        pci::{PCICfgOffset, PCICommand, PCIController, PCIIOAccessSize},
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
    let mut this = (*crate::sys::state::SYS_STATE.get())
        .scheduler
        .assume_init_mut()
        .lock();

    if !this.first_launch {
        let old_thread = this.current_thread();
        old_thread.regs = *state;
        old_thread.state = super::ThreadState::Inactive;
    } else {
        this.first_launch = false;
    }

    let thread = this.find_next_thread();
    *state = thread.regs;
    thread.state = super::ThreadState::Active;
    *(*TSS.get()).assume_init_mut() =
        TaskSegmentSelector::new(thread.kern_rsp.as_ptr() as usize + thread.kern_rsp.len());

    this.processes[0].cr3.set();
}

fn test_thread() {
    unsafe {
        loop {
            // writeln!(
            //     &mut crate::sys::io::serial::SERIAL.lock(),
            //     "hi from thread 0"
            // )
            // .unwrap();

            core::arch::asm!("hlt");
        }
    }
}

fn test_thread1() {
    unsafe {
        loop {
            // writeln!(
            //     &mut crate::sys::io::serial::SERIAL.lock(),
            //     "hi from thread 1"
            // )
            // .unwrap();
            core::arch::asm!("hlt");
        }
    }
}

fn test_thread2() {
    let state = unsafe { &mut *crate::sys::state::SYS_STATE.get() };
    let acpi = unsafe { state.acpi.assume_init_mut() };

    let pci = PCIController::new(acpi.find("MCFG"));
    let ac97 = pci
        .find(
            |addr| pci.get_io(addr),
            move |dev| unsafe {
                dev.cfg_read::<_, u32>(PCICfgOffset::ClassCode, PCIIOAccessSize::Word) == 0x0401
            },
        )
        .map(crate::driver::audio::ac97::AC97::new)
        .map(|v| unsafe { (*crate::driver::audio::ac97::INSTANCE.get()).write(v) });

    let sata_device = pci
        .find(
            |v| pci.get_io(v),
            move |dev| unsafe {
                dev.cfg_read::<_, u32>(PCICfgOffset::ClassCode, PCIIOAccessSize::Word) == 0x0106
            },
        )
        .unwrap();

    unsafe {
        sata_device.cfg_write::<_, u16>(
            PCICfgOffset::Command,
            PCICommand::from(
                sata_device.cfg_read::<_, u32>(PCICfgOffset::Command, PCIIOAccessSize::Word) as u16,
            )
            .with_pio(false)
            .with_bus_master(true)
            .with_disable_intrs(true)
            .into(),
            PCIIOAccessSize::Word,
        );

        let addr: u32 =
            sata_device.cfg_read::<_, u32>(PCICfgOffset::BaseAddr5, PCIIOAccessSize::DWord) & !0xF;
        let addr_virt = addr as usize + amd64::paging::PHYS_VIRT_OFFSET;
        let hba_mem = &mut *(addr_virt as *mut crate::driver::storage::ahci::hba::HBAMemory);
        state.pml4.assume_init_mut().map_mmio(
            addr_virt,
            addr as usize,
            (size_of::<crate::driver::storage::ahci::hba::HBAMemory>() + 0xFFF) / 0x1000,
            PageTableEntry::new().with_present(true).with_writable(true),
        );

        hba_mem.set_global_host_control(hba_mem.global_host_control().with_ahci_enable(true));

        info!("{:#X?}", hba_mem);
        if hba_mem.cap_ext().bios_handoff() {
            info!("Doing BIOS/OS Handoff");
        }

        for i in 0..32 {
            if let Some(port) = hba_mem.get_port_ref(i) {
                info!("Port {}: {:#X?}", i, port);
            }
        }
    }

    if let Some(terminal) = &mut state.terminal {
        let ps2ctl = crate::PS2Ctl::new();
        ps2ctl.init();
        unsafe {
            (*crate::driver::keyboard::ps2::INSTANCE.get()).write(ps2ctl);
        }

        crate::terminal_loop::terminal_loop(acpi, &pci, terminal, ac97);
    }
}

impl Scheduler {
    pub fn new(timer: &impl Timer) -> Self {
        let mut processes = VecDeque::new();
        let mut kern_proc = super::Process::new(0, "", "");
        let kern_thread = super::Thread::new(0, test_thread as usize);
        unsafe {
            *(*TSS.get()).assume_init_mut() = TaskSegmentSelector::new(
                kern_thread.kern_rsp.as_ptr() as usize + kern_thread.kern_rsp.len(),
            );
            let entries = &mut *crate::sys::gdt::ENTRIES.get();
            let tss = (*TSS.get()).as_ptr() as usize;
            entries[entries.len() - 2].set_base(tss as u32);
            entries[entries.len() - 2].attrs.set_present(true);

            entries.last_mut().unwrap().limit_low = (tss >> 32) as u16;
            entries.last_mut().unwrap().base_low = (tss >> 48) as u16;

            core::arch::asm!(
                "ltr ax",
                in("ax") crate::sys::gdt::SegmentSelector::new(3, crate::sys::gdt::PrivilegeLevel::Hypervisor).0,
            );
        }
        kern_proc.threads.push_back(kern_thread);
        let kern_thread = super::Thread::new(1, test_thread1 as usize);
        kern_proc.threads.push_back(kern_thread);
        let kern_thread = super::Thread::new(2, test_thread2 as usize);
        kern_proc.threads.push_back(kern_thread);
        processes.push_back(kern_proc);

        let lapic = unsafe {
            (*crate::sys::state::SYS_STATE.get())
                .lapic
                .assume_init_ref()
        };

        lapic.setup_timer(timer);

        crate::driver::intrs::idt::set_handler(128, schedule, true, true);
        crate::driver::acpi::ioapic::wire_legacy_irq(128 - 0x20, false);

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
        let t = proc.threads.pop_front().unwrap();
        proc.threads.push_back(t);
        proc.threads.front_mut().unwrap()
    }
}
