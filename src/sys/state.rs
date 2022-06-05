//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::vec::Vec;
use core::{cell::SyncUnsafeCell, mem::MaybeUninit};

use amd64::intrs::apic::LocalApic;
use kaboom::tags::{module::Module, SpecialisedSettings};

use super::{pmm::BitmapAllocator, terminal::Terminal, vmm::Pml4};
use crate::driver::acpi::{madt::Madt, Acpi};

pub static SYS_STATE: SyncUnsafeCell<SystemState> = SyncUnsafeCell::new(SystemState::new());

pub struct SystemState {
    pub modules: Option<Vec<Module>>,
    pub boot_settings: SpecialisedSettings,
    pub pmm: MaybeUninit<BitmapAllocator>,
    pub pml4: MaybeUninit<&'static mut Pml4>,
    pub terminal: Option<Terminal>,
    pub acpi: MaybeUninit<Acpi>,
    pub madt: MaybeUninit<Madt>,
    pub lapic: MaybeUninit<LocalApic>,
}

impl SystemState {
    pub const fn new() -> Self {
        Self {
            modules: None,
            boot_settings: SpecialisedSettings { verbose: false },
            pmm: MaybeUninit::uninit(),
            pml4: MaybeUninit::uninit(),
            terminal: None,
            acpi: MaybeUninit::uninit(),
            madt: MaybeUninit::uninit(),
            lapic: MaybeUninit::uninit(),
        }
    }
}
