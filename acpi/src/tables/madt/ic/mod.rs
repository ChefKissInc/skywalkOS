// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use self::{
    ioapic::{InterruptSourceOverride, NMISource, IOAPIC},
    proc_lapic::{LocalAPICAddrOverride, LocalAPICNMI, ProcessorLocalAPIC},
};

pub mod ioapic;
pub mod proc_lapic;

#[derive(Debug)]
pub enum InterruptController {
    ProcessorLocalAPIC(&'static ProcessorLocalAPIC),
    InputOutputAPIC(&'static IOAPIC),
    InterruptSourceOverride(&'static InterruptSourceOverride),
    NMISource(&'static NMISource),
    LocalApicNMI(&'static LocalAPICNMI),
    LocalAPICAddrOverride(&'static LocalAPICAddrOverride),
    InputOutputSAPIC(&'static ICHeader),
    LocalSapic(&'static ICHeader),
    PlatformInterruptSrcs(&'static ICHeader),
    ProcessorLocalx2APIC(&'static ICHeader),
    Localx2APICNmi(&'static ICHeader),
    GICCPU(&'static ICHeader),
    GICDist(&'static ICHeader),
    GICMSIFrame(&'static ICHeader),
    GICRedist(&'static ICHeader),
    GICIts(&'static ICHeader),
    MPWakeup(&'static ICHeader),
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
