//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;
use core::{cell::SyncUnsafeCell, mem::MaybeUninit};

use sulfur_dioxide::tags::{module::Module, SpecialisedSettings};

use super::{pmm::BitmapAllocator, proc::sched::Scheduler, terminal::Terminal, vmm::PageTableLvl4};
use crate::driver::acpi::{apic::LocalAPIC, madt::MADTData, ACPIPlatform};

pub static SYS_STATE: SyncUnsafeCell<SystemState> = SyncUnsafeCell::new(SystemState::new());

pub struct SystemState {
    pub modules: Option<Vec<Module>>,
    pub boot_settings: SpecialisedSettings,
    pub pmm: MaybeUninit<spin::Mutex<BitmapAllocator>>,
    pub pml4: MaybeUninit<&'static mut PageTableLvl4>,
    pub terminal: Option<Terminal>,
    pub acpi: MaybeUninit<ACPIPlatform>,
    pub madt: MaybeUninit<MADTData>,
    pub lapic: MaybeUninit<LocalAPIC>,
    pub scheduler: MaybeUninit<spin::Mutex<Scheduler>>,
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
            scheduler: MaybeUninit::uninit(),
        }
    }
}
