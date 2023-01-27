// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use self::{
    ioapic::{InterruptSourceOverride, IoApic, NMISource},
    proc_lapic::{LocalAPICAddrOverride, LocalAPICNMI, ProcessorLocalAPIC},
};

pub mod ioapic;
pub mod proc_lapic;

#[derive(Debug)]
pub enum InterruptController {
    ProcessorLocalAPIC(&'static ProcessorLocalAPIC),
    InputOutputAPIC(&'static IoApic),
    InterruptSourceOverride(&'static InterruptSourceOverride),
    NmiSrc(&'static NMISource),
    LApicNmi(&'static LocalAPICNMI),
    LApicAddrOverride(&'static LocalAPICAddrOverride),
    IoSapic(&'static ICHeader),
    LocalSapic(&'static ICHeader),
    PlatformInterruptSrcs(&'static ICHeader),
    ProcessorLocalx2APIC(&'static ICHeader),
    Localx2APICNmi(&'static ICHeader),
    GicCpu(&'static ICHeader),
    GicDist(&'static ICHeader),
    GicMsiFrame(&'static ICHeader),
    GicRedist(&'static ICHeader),
    GicIts(&'static ICHeader),
    MpWakeup(&'static ICHeader),
    Reserved(&'static ICHeader),
    OemReserved(&'static ICHeader),
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ICHeader {
    pub type_: u8,
    length: u8,
}

impl ICHeader {
    #[must_use]
    pub fn length(&self) -> usize {
        self.length.into()
    }
}
