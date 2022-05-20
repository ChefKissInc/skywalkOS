//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::vec::Vec;
use core::cell::SyncUnsafeCell;

use amd64::sys::apic::LocalApic;
use kaboom::tags::{module::Module, SpecialisedSettings};

use super::{pmm::BitmapAllocator, terminal::Terminal, vmm::Pml4};
use crate::driver::acpi::{madt::Madt, Acpi};

pub static SYS_STATE: SyncUnsafeCell<SystemState> = SyncUnsafeCell::new(SystemState::new());

pub struct SystemState {
    pub modules: spin::Once<Vec<Module>>,
    pub boot_settings: SpecialisedSettings,
    pub pmm: spin::Once<BitmapAllocator>,
    pub pml4: spin::Once<&'static mut Pml4>,
    pub terminal: spin::Once<Terminal>,
    pub acpi: spin::Once<Acpi>,
    pub madt: spin::Once<Madt>,
    pub lapic: spin::Once<LocalApic>,
}

impl SystemState {
    pub const fn new() -> Self {
        Self {
            modules: spin::Once::new(),
            boot_settings: SpecialisedSettings { verbose: false },
            pmm: spin::Once::new(),
            pml4: spin::Once::new(),
            terminal: spin::Once::new(),
            acpi: spin::Once::new(),
            madt: spin::Once::new(),
            lapic: spin::Once::new(),
        }
    }
}
