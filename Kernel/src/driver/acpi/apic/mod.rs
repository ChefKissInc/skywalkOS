// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use amd64::registers::msr::{apic::APICBase, ModelSpecificReg};
use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;

use crate::sys::{gdt::PrivilegeLevel, RegisterState};

pub mod lvt;

pub struct LocalAPIC {
    addr: u64,
}

#[derive(Debug, IntoPrimitive)]
#[repr(u64)]
pub enum LocalAPICReg {
    ID = 0x20,
    Ver = 0x30,
    TaskPriority = 0x80,
    ArbitrationPriority = 0x90,
    ProcessorPriority = 0xA0,
    EndOfInterrupt = 0xB0,
    RemoteRead = 0xC0,
    LogicalDestination = 0xD0,
    DestinationFormat = 0xE0,
    SpuriousInterruptVector = 0xF0,
    InService = 0x100,
    TriggerMode = 0x180,
    InterruptRequest = 0x200,
    ErrorStatus = 0x280,
    LvtCorrectedMachineCheck = 0x2F0,
    InterruptCommand = 0x300,
    InterruptCommand2 = 0x310,
    LVTTimer = 0x320,
    LVTThermalSensor = 0x330,
    LVTPerfCounter = 0x340,
    LVTLint0 = 0x350,
    LVTLint1 = 0x360,
    LVTError = 0x370,
    TimerInitialCount = 0x380,
    TimerCurrentCount = 0x390,
    TimerDivideConfiguration = 0x3E0,
}

#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[bits = 3]
#[repr(u8)]
pub enum DeliveryMode {
    #[default]
    Fixed = 0b000,
    LowestPriority = 0b001,
    Smi = 0b010,
    Nmi = 0b100,
    Init = 0b101,
    StartUp = 0b110,
    ExtInt = 0b111,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub struct ErrorStatus {
    pub send_checksum_err: bool,
    pub recv_checksum_err: bool,
    pub send_accept_err: bool,
    pub recv_accept_err: bool,
    pub redir_ipi: bool,
    pub send_illegal_vec: bool,
    pub recv_illegal_vec: bool,
    pub illegal_reg_addr: bool,
    #[skip]
    __: B24,
}

#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
#[repr(u8)]
pub enum IntCmdDestShorthand {
    #[default]
    None = 0b00,
    ToSelf,
    ToAllInclSelf,
    ToAllExclSelf,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub struct SpuriousIntrVector {
    pub vector: u8,
    pub apic_soft_enable: bool,
    pub focus_proc_check: bool,
    #[skip]
    __: B2,
    pub eoi_broadcast_suppress: bool,
    #[skip]
    __: B19,
}

#[bitfield(bits = 64)]
#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub struct InterruptCommand {
    pub vector: u8,
    pub delivery_mode: DeliveryMode,
    pub logical_dest: bool,
    pub delivery_pending: bool,
    #[skip]
    __: bool,
    pub assert: bool,
    pub level_trigger: bool,
    #[skip]
    __: B2,
    pub dest_shorthand: IntCmdDestShorthand,
    #[skip]
    __: B36,
    pub dest: u8,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub struct LocalAPICVer {
    #[skip(setters)]
    pub ver: u8,
    #[skip]
    __: u8,
    #[skip(setters)]
    pub max_lvt_entry: u8,
    #[skip(setters)]
    pub support_eoi_suppression: bool,
    #[skip]
    __: B7,
}

impl LocalAPIC {
    #[must_use]
    pub const fn new(addr: u64) -> Self {
        Self { addr }
    }

    #[inline]
    pub fn write_reg<T: Into<u64>, V: Into<u32>>(&self, reg: T, value: V) {
        unsafe { ((self.addr + reg.into()) as *mut u32).write_volatile(value.into()) }
    }

    #[inline]
    pub fn read_reg<T: Into<u64>, R: From<u32>>(&self, reg: T) -> R {
        unsafe { ((self.addr + reg.into()) as *const u32).read_volatile() }.into()
    }

    #[inline]
    pub fn read_ver(&self) -> LocalAPICVer {
        self.read_reg(LocalAPICReg::Ver)
    }

    #[inline]
    pub fn send_eoi(&self) {
        self.write_reg(LocalAPICReg::EndOfInterrupt, 0u32);
    }

    #[inline]
    pub fn set_timer_divide(&self, value: u32) {
        self.write_reg(LocalAPICReg::TimerDivideConfiguration, value);
    }

    #[inline]
    pub fn set_timer_init_count(&self, value: u32) {
        self.write_reg(LocalAPICReg::TimerInitialCount, value);
    }

    #[inline]
    pub fn read_timer_counter(&self) -> u32 {
        self.read_reg(LocalAPICReg::TimerCurrentCount)
    }

    #[inline]
    pub fn read_timer(&self) -> lvt::TimerLVT {
        self.read_reg(LocalAPICReg::LVTTimer)
    }

    #[inline]
    pub fn write_timer(&self, val: lvt::TimerLVT) {
        self.write_reg(LocalAPICReg::LVTTimer, val);
    }

    #[inline]
    pub fn read_lint(&self, lint1: bool) -> lvt::LocalVectorTable {
        self.read_reg(if lint1 {
            LocalAPICReg::LVTLint1
        } else {
            LocalAPICReg::LVTLint0
        })
    }

    #[inline]
    pub fn write_lint(&self, lint1: bool, val: lvt::LocalVectorTable) {
        self.write_reg(
            if lint1 {
                LocalAPICReg::LVTLint1
            } else {
                LocalAPICReg::LVTLint0
            },
            val,
        );
    }

    #[inline]
    pub fn reset_error(&self) {
        self.write_reg(LocalAPICReg::ErrorStatus, 0u32);
    }

    #[inline]
    pub fn error(&self) -> ErrorStatus {
        self.read_reg(LocalAPICReg::ErrorStatus)
    }

    // #[inline]
    // pub fn read_icr(&self) -> InterruptCommand {
    //     (((self.read_reg::<_, u64>(LocalAPICReg::InterruptCommand2)) << 32)
    //         | (self.read_reg::<_, u64>(LocalAPICReg::InterruptCommand)))
    //     .into()
    // }

    // #[inline]
    // pub fn write_icr(&self, val: InterruptCommand) {
    //     let val: u64 = val.into();
    //     let a = val as u32;
    //     let b = (val >> 32) as u32;
    //     self.write_reg(LocalAPICReg::InterruptCommand2, b);
    //     self.write_reg(LocalAPICReg::InterruptCommand, a);
    // }

    #[inline]
    pub fn write_spurious_intr_vec(&self, val: SpuriousIntrVector) {
        self.write_reg(LocalAPICReg::SpuriousInterruptVector, val);
    }

    #[inline]
    pub fn enable(&self) {
        self.write_spurious_intr_vec(
            SpuriousIntrVector::new()
                .with_vector(0xFD)
                .with_apic_soft_enable(true),
        );
    }

    #[inline]
    pub fn setup_timer(&self, timer: &impl crate::driver::timer::Timer) {
        self.set_timer_divide(0x3);
        self.set_timer_init_count(0xFFFF_FFFF);

        self.write_timer(self.read_timer().with_mask(false));
        timer.sleep(10);
        self.write_timer(self.read_timer().with_mask(true));

        let ticks_per_ms = (0xFFFF_FFFF - self.read_timer_counter()) / 10;
        self.write_timer(
            lvt::TimerLVT::new()
                .with_vector(128)
                .with_mask(true)
                .with_mode(lvt::TimerMode::Periodic),
        );
        self.set_timer_divide(0x3);
        self.set_timer_init_count(ticks_per_ms);
    }
}

unsafe extern "C" fn lapic_error_handler(_state: &mut RegisterState) {
    let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
    let lapic = state.lapic.get().unwrap();
    // Pentium errata 3AP
    if lapic.read_ver().max_lvt_entry() > 3 {
        lapic.reset_error();
    }
    error!("APIC error: {:#X?}", lapic.error());
}

unsafe extern "C" fn spurious_vector_handler(_state: &mut RegisterState) {
    error!("Spurious APIC vector");
}

pub fn setup(state: &mut crate::sys::state::SystemState) {
    let addr = state.madt.get().unwrap().lock().lapic_addr;
    unsafe { APICBase::read().with_apic_base(addr).write() }
    let pml4 = state.pml4.get_mut().unwrap();

    let virt_addr = addr + amd64::paging::PHYS_VIRT_OFFSET;
    unsafe {
        pml4.map_mmio(
            virt_addr,
            addr,
            1,
            amd64::paging::PageTableEntry::new()
                .with_present(true)
                .with_writable(true),
        );
    }
    debug!("LAPIC address is {addr:#X?}");
    let lapic = state.lapic.call_once(|| LocalAPIC::new(virt_addr));
    let ver = lapic.read_ver();
    debug!("LAPIC version is {ver:#X?}");

    // Do not trust LAPIC to be empty at boot
    if ver.max_lvt_entry() > 2 {
        lapic.write_reg(
            LocalAPICReg::LVTError,
            lvt::LocalVectorTable::new().with_mask(true),
        );
        crate::driver::intrs::idt::set_handler(
            0xFE,
            0,
            PrivilegeLevel::Supervisor,
            lapic_error_handler,
            false,
            true,
        );
    }

    lapic.write_timer(lapic.read_timer().with_mask(true));
    lapic.write_lint(false, lapic.read_lint(false).with_mask(true));
    lapic.write_lint(true, lapic.read_lint(true).with_mask(true));
    if ver.max_lvt_entry() > 3 {
        lapic.write_reg(
            LocalAPICReg::LVTPerfCounter,
            lvt::LocalVectorTable::new().with_mask(true),
        );
    }

    if ver.max_lvt_entry() > 4 {
        lapic.write_reg(
            LocalAPICReg::LVTThermalSensor,
            lvt::LocalVectorTable::new().with_mask(true),
        );
    }

    lapic.enable();

    crate::driver::intrs::idt::set_handler(
        0xFD,
        0,
        PrivilegeLevel::Supervisor,
        spurious_vector_handler,
        true,
        true,
    );

    // Set up virtual wire
    lapic.write_lint(
        false,
        lvt::LocalVectorTable::new().with_delivery_mode(DeliveryMode::ExtInt),
    );
    lapic.write_lint(
        true,
        lvt::LocalVectorTable::new().with_delivery_mode(DeliveryMode::Nmi),
    );

    if ver.max_lvt_entry() > 2 {
        lapic.write_reg(
            LocalAPICReg::LVTError,
            lvt::LocalVectorTable::new().with_vector(0xFE),
        );
    }
}
