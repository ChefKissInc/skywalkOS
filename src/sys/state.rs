// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;
use core::{cell::SyncUnsafeCell, mem::MaybeUninit};

use sulphur_dioxide::{boot_attrs::BootSettings, module::Module};

use super::{pmm::BitmapAllocator, proc::sched::Scheduler, terminal::Terminal, vmm::PageTableLvl4};
use crate::driver::acpi::{apic::LocalAPIC, madt::MADTData, ACPIPlatform};

pub static SYS_STATE: SyncUnsafeCell<SystemState> = SyncUnsafeCell::new(SystemState::new());

pub struct SystemState {
    pub kern_symbols: MaybeUninit<&'static [sulphur_dioxide::kern_sym::KernSymbol]>,
    pub boot_settings: BootSettings,
    pub pmm: MaybeUninit<spin::Mutex<BitmapAllocator>>,
    pub pml4: MaybeUninit<&'static mut PageTableLvl4>,
    pub modules: Option<Vec<Module>>,
    pub terminal: Option<Terminal>,
    pub acpi: MaybeUninit<ACPIPlatform>,
    pub madt: MaybeUninit<MADTData>,
    pub lapic: MaybeUninit<LocalAPIC>,
    pub scheduler: MaybeUninit<spin::Mutex<Scheduler>>,
}

impl SystemState {
    pub const fn new() -> Self {
        Self {
            kern_symbols: MaybeUninit::uninit(),
            boot_settings: BootSettings { verbose: false },
            pmm: MaybeUninit::uninit(),
            pml4: MaybeUninit::uninit(),
            modules: None,
            terminal: None,
            acpi: MaybeUninit::uninit(),
            madt: MaybeUninit::uninit(),
            lapic: MaybeUninit::uninit(),
            scheduler: MaybeUninit::uninit(),
        }
    }
}
