// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

use alloc::{string::String, vec::Vec};
use core::cell::SyncUnsafeCell;

use hashbrown::HashMap;
use sulphur_dioxide::BootSettings;

use super::{
    pmm::BitmapAllocator,
    proc::{scheduler::Scheduler, userland::allocations::UserAllocationTracker},
    terminal::Terminal,
    vmm::PageTableLvl4,
};
use crate::{
    acpi::{apic::LocalAPIC, madt::MADTData, ACPIPlatform},
    utils::incr_id::IncrementalIDGen,
};

pub static SYS_STATE: SyncUnsafeCell<SystemState> = SyncUnsafeCell::new(SystemState::new());

#[derive(Debug, Default, Clone)]
pub struct OSDTEntry {
    pub parent: Option<u64>,
    pub id: u64,
    pub properties: HashMap<String, iridium_kit::dt::OSValue>,
    pub children: Vec<u64>,
}

pub struct SystemState {
    pub kern_symbols: spin::Once<&'static [sulphur_dioxide::KernSymbol]>,
    pub boot_settings: BootSettings,
    pub pmm: spin::Once<spin::Mutex<BitmapAllocator>>,
    pub pml4: spin::Once<&'static mut PageTableLvl4>,
    pub dc_cache: Option<Vec<u8>>,
    pub terminal: Option<Terminal>,
    pub acpi: spin::Once<ACPIPlatform>,
    pub madt: spin::Once<spin::Mutex<MADTData>>,
    pub lapic: spin::Once<LocalAPIC>,
    pub scheduler: spin::Once<spin::Mutex<Scheduler>>,
    pub interrupt_context: Option<super::RegisterState>,
    pub in_panic: bool,
    pub user_allocations: spin::Once<spin::Mutex<UserAllocationTracker>>,
    pub dt_index: spin::Once<spin::Mutex<HashMap<u64, OSDTEntry>>>,
    pub dt_id_gen: spin::Once<spin::Mutex<IncrementalIDGen>>,
}

impl SystemState {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            kern_symbols: spin::Once::new(),
            boot_settings: BootSettings { verbose: false },
            pmm: spin::Once::new(),
            pml4: spin::Once::new(),
            dc_cache: None,
            terminal: None,
            acpi: spin::Once::new(),
            madt: spin::Once::new(),
            lapic: spin::Once::new(),
            scheduler: spin::Once::new(),
            interrupt_context: None,
            in_panic: false,
            user_allocations: spin::Once::new(),
            dt_index: spin::Once::new(),
            dt_id_gen: spin::Once::new(),
        }
    }
}
