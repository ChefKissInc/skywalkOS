// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::{string::String, vec::Vec};
use core::cell::SyncUnsafeCell;

use hashbrown::HashMap;

use super::{
    pmm::BitmapAllocator,
    proc::{scheduler::Scheduler, userland::allocations::UserAllocationTracker},
    terminal::Terminal,
    vmm::PageTableLvl4,
};
use crate::{
    acpi::{apic::LocalAPIC, madt::MADTData, ACPIState},
    utils::incr_id::IncrementalIDGen,
};

pub static SYS_STATE: SyncUnsafeCell<SystemState> = SyncUnsafeCell::new(SystemState::new());

#[derive(Debug, Default, Clone)]
pub struct OSDTEntry {
    pub parent: Option<u64>,
    pub id: u64,
    pub properties: HashMap<String, tungstenkit::dt::OSValue>,
    pub children: Vec<u64>,
}

pub struct SystemState {
    pub kern_symbols: Option<&'static [sulphur_dioxide::KernSymbol]>,
    pub verbose: bool,
    pub pmm: Option<spin::Mutex<BitmapAllocator>>,
    pub pml4: Option<&'static mut PageTableLvl4>,
    pub terminal: Option<Terminal>,
    pub acpi: Option<ACPIState>,
    pub madt: Option<spin::Mutex<MADTData>>,
    pub lapic: Option<LocalAPIC>,
    pub scheduler: Option<spin::Mutex<Scheduler>>,
    pub interrupt_context: Option<super::RegisterState>,
    pub in_panic: core::sync::atomic::AtomicBool,
    pub usr_allocs: Option<spin::Mutex<UserAllocationTracker>>,
    pub dt_index: Option<spin::RwLock<HashMap<u64, spin::Mutex<OSDTEntry>>>>,
    pub dt_id_gen: Option<spin::Mutex<IncrementalIDGen>>,
    pub tkcache: Option<spin::Mutex<tungstenkit::TKCache>>,
}

impl SystemState {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            kern_symbols: None,
            verbose: cfg!(debug_assertions),
            pmm: None,
            pml4: None,
            terminal: None,
            acpi: None,
            madt: None,
            lapic: None,
            scheduler: None,
            interrupt_context: None,
            in_panic: core::sync::atomic::AtomicBool::new(false),
            usr_allocs: None,
            dt_index: None,
            dt_id_gen: None,
            tkcache: None,
        }
    }
}
