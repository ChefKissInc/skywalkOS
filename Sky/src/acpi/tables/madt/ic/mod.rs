// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use self::{
    ioapic::{InputOutputAPIC, IntrSourceOverride, NMISource},
    proc_lapic::{LocalAPICAddrOverride, LocalAPICNMI, ProcessorLocalAPIC},
};

pub mod ioapic;
pub mod proc_lapic;

#[derive(Debug)]
pub enum InterruptController {
    ProcessorLocalAPIC(&'static ProcessorLocalAPIC),
    InputOutputAPIC(&'static InputOutputAPIC),
    IntrSourceOverride(&'static IntrSourceOverride),
    NMISource(&'static NMISource),
    LocalAPICNMI(&'static LocalAPICNMI),
    LocalAPICAddrOverride(&'static LocalAPICAddrOverride),
    InputOutputSAPIC(&'static ICHeader),
    LocalSAPIC(&'static ICHeader),
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
    #[inline]
    pub fn length(self) -> usize {
        self.length.into()
    }
}
