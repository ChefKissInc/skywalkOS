//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::vec::Vec;
use core::cell::UnsafeCell;

use amd64::sys::apic::LocalApic;
use kaboom::tags::{module::Module, SpecialisedSettings};

use super::{pmm::BitmapAllocator, terminal::Terminal, vmm::Pml4};
use crate::driver::acpi::{madt::Madt, Acpi};

pub static SYS_STATE: SystemState = SystemState::new();

#[derive(Debug)]
pub struct SystemState {
    pub modules: UnsafeCell<spin::Once<Vec<Module>>>,
    pub boot_settings: spin::Once<SpecialisedSettings>,
    pub pmm: UnsafeCell<spin::Once<BitmapAllocator>>,
    pub pml4: UnsafeCell<spin::Once<&'static mut Pml4>>,
    pub terminal: UnsafeCell<spin::Once<Terminal>>,
    pub acpi: UnsafeCell<spin::Once<Acpi>>,
    pub madt: UnsafeCell<spin::Once<Madt>>,
    pub lapic: UnsafeCell<spin::Once<LocalApic>>,
}

unsafe impl Sync for SystemState {}

impl SystemState {
    pub const fn new() -> Self {
        Self {
            modules: UnsafeCell::new(spin::Once::new()),
            boot_settings: spin::Once::new(),
            pmm: UnsafeCell::new(spin::Once::new()),
            pml4: UnsafeCell::new(spin::Once::new()),
            terminal: UnsafeCell::new(spin::Once::new()),
            acpi: UnsafeCell::new(spin::Once::new()),
            madt: UnsafeCell::new(spin::Once::new()),
            lapic: UnsafeCell::new(spin::Once::new()),
        }
    }
}
