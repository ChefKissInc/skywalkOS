// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use amd64::{
    msr::{apic::APICBase, ModelSpecificReg},
    paging::PageTableFlags,
};
use num_enum::IntoPrimitive;

use crate::system::{gdt::PrivilegeLevel, RegisterState};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// 3 bits
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

impl DeliveryMode {
    #[inline]
    const fn into_bits(self) -> u64 {
        self as _
    }

    #[inline]
    const fn from_bits(value: u64) -> Self {
        match value {
            0b000 => Self::Fixed,
            0b001 => Self::LowestPriority,
            0b010 => Self::Smi,
            0b100 => Self::Nmi,
            0b101 => Self::Init,
            0b110 => Self::StartUp,
            0b111 => Self::ExtInt,
            _ => panic!("Invalid value for DeliveryMode"),
        }
    }

    #[inline]
    const fn into_bits_32(self) -> u32 {
        self as _
    }

    #[inline]
    const fn from_bits_32(value: u32) -> Self {
        Self::from_bits(value as _)
    }
}

#[bitfield(u32)]
pub struct ErrorStatus {
    pub send_checksum_err: bool,
    pub recv_checksum_err: bool,
    pub send_accept_err: bool,
    pub recv_accept_err: bool,
    pub redir_ipi: bool,
    pub send_illegal_vec: bool,
    pub recv_illegal_vec: bool,
    pub illegal_reg_addr: bool,
    #[bits(24)]
    __: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// 2 bits
pub enum IntCmdDestShorthand {
    #[default]
    None = 0b00,
    ToSelf,
    ToAllInclSelf,
    ToAllExclSelf,
}

impl IntCmdDestShorthand {
    const fn into_bits(self) -> u64 {
        self as _
    }

    const fn from_bits(value: u64) -> Self {
        match value {
            0b00 => Self::None,
            0b01 => Self::ToSelf,
            0b10 => Self::ToAllInclSelf,
            0b11 => Self::ToAllExclSelf,
            _ => panic!("Invalid value for IntCmdDestShorthand"),
        }
    }
}

#[bitfield(u32)]
pub struct SpuriousIntrVector {
    pub vector: u8,
    pub apic_soft_enable: bool,
    pub focus_proc_check: bool,
    #[bits(2)]
    __: u8,
    pub eoi_broadcast_suppress: bool,
    #[bits(19)]
    __: u32,
}

#[bitfield(u64)]
pub struct InterruptCommand {
    pub vector: u8,
    #[bits(3)]
    pub delivery_mode: DeliveryMode,
    pub logical_dest: bool,
    pub delivery_pending: bool,
    __: bool,
    pub assert: bool,
    pub level_trigger: bool,
    #[bits(2)]
    __: u8,
    #[bits(2)]
    pub dest_shorthand: IntCmdDestShorthand,
    #[bits(36)]
    __: u64,
    pub dest: u8,
}

#[bitfield(u32)]
pub struct LocalAPICVer {
    pub ver: u8,
    __: u8,
    pub max_lvt_entry: u8,
    pub support_eoi_suppression: bool,
    #[bits(7)]
    __: u8,
}

impl LocalAPIC {
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self { addr }
    }

    pub fn write_reg<T: Into<u64>, V: Into<u32>>(&self, reg: T, value: V) {
        unsafe { ((self.addr + reg.into()) as *mut u32).write_volatile(value.into()) }
    }

    pub fn read_reg<T: Into<u64>, R: From<u32>>(&self, reg: T) -> R {
        unsafe { ((self.addr + reg.into()) as *const u32).read_volatile() }.into()
    }

    pub fn read_ver(&self) -> LocalAPICVer {
        self.read_reg(LocalAPICReg::Ver)
    }

    pub fn send_eoi(&self) {
        self.write_reg(LocalAPICReg::EndOfInterrupt, 0u32);
    }

    pub fn set_timer_divide(&self, value: u32) {
        self.write_reg(LocalAPICReg::TimerDivideConfiguration, value);
    }

    pub fn set_timer_init_count(&self, value: u32) {
        self.write_reg(LocalAPICReg::TimerInitialCount, value);
    }

    pub fn read_timer_counter(&self) -> u32 {
        self.read_reg(LocalAPICReg::TimerCurrentCount)
    }

    pub fn read_timer(&self) -> lvt::TimerLVT {
        self.read_reg(LocalAPICReg::LVTTimer)
    }

    pub fn write_timer(&self, val: lvt::TimerLVT) {
        self.write_reg(LocalAPICReg::LVTTimer, val);
    }

    pub fn read_lint(&self, lint1: bool) -> lvt::LocalVectorTable {
        self.read_reg(if lint1 {
            LocalAPICReg::LVTLint1
        } else {
            LocalAPICReg::LVTLint0
        })
    }

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

    pub fn reset_error(&self) {
        self.write_reg(LocalAPICReg::ErrorStatus, 0u32);
    }

    pub fn error(&self) -> ErrorStatus {
        self.read_reg(LocalAPICReg::ErrorStatus)
    }

    pub fn write_spurious_intr_vec(&self, val: SpuriousIntrVector) {
        self.write_reg(LocalAPICReg::SpuriousInterruptVector, val);
    }

    pub fn enable(&self) {
        self.write_spurious_intr_vec(
            SpuriousIntrVector::new()
                .with_vector(0xFD)
                .with_apic_soft_enable(true),
        );
    }

    pub fn setup_timer(&self, timer: &impl crate::timer::Timer) {
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

unsafe extern "sysv64" fn lapic_error_handler(_state: &mut RegisterState) {
    let lapic = (*crate::system::state::SYS_STATE.get())
        .lapic
        .as_ref()
        .unwrap();
    // Pentium errata 3AP
    if lapic.read_ver().max_lvt_entry() > 3 {
        lapic.reset_error();
    }
    error!("APIC error: {:#X?}", lapic.error());
}

unsafe extern "sysv64" fn spurious_vector_handler(_state: &mut RegisterState) {
    error!("Spurious APIC vector");
}

pub fn setup(state: &mut crate::system::state::SystemState) {
    let addr = unsafe {
        let mut madt = state.madt.as_ref().unwrap().lock();
        let base = APICBase::read();
        if base.apic_global_enable() && base.apic_base() != 0 {
            debug!("APIC already enabled, base is {base:#X?}");
            madt.lapic_addr = base.apic_base() << 12;
        } else {
            debug!("Old APIC base is {base:#X?}");
            let base = base
                .with_apic_global_enable(true)
                .with_apic_base(madt.lapic_addr >> 12);
            debug!("New APIC base is {base:#X?}");
            base.write();
        }
        madt.lapic_addr
    };
    let pml4 = state.pml4.as_ref().unwrap();

    let virt_addr = addr + amd64::paging::PHYS_VIRT_OFFSET;
    unsafe {
        pml4.lock().map_mmio(
            virt_addr,
            addr,
            1,
            PageTableFlags::new_present().with_writable(true),
        );
    }
    debug!("LAPIC address is {addr:#X?}");
    let lapic = LocalAPIC::new(virt_addr);
    let ver = lapic.read_ver();
    debug!("LAPIC version is {ver:#X?}");

    // Do not trust LAPIC to be empty at boot
    if ver.max_lvt_entry() > 2 {
        lapic.write_reg(
            LocalAPICReg::LVTError,
            lvt::LocalVectorTable::new().with_mask(true),
        );
        crate::interrupts::idt::set_handler(
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

    crate::interrupts::idt::set_handler(
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

    state.lapic = Some(lapic);
}
