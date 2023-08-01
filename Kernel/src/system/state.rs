// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::{string::String, vec::Vec};
use core::cell::SyncUnsafeCell;

use hashbrown::HashMap;

use super::{
    pmm::BitmapAllocator, proc::scheduler::Scheduler, terminal::Terminal, vmm::PageTableLvl4,
};
use crate::{
    acpi::{apic::LocalAPIC, madt::MADTData, ACPIState},
    utils::incr_id::IncrementalIDGen,
};

pub static SYS_STATE: SyncUnsafeCell<SystemState> = SyncUnsafeCell::new(SystemState::new());

#[derive(Debug, Default, Clone)]
pub struct OSDTEntry {
    pub parent: Option<fireworkkit::osdtentry::OSDTEntry>,
    pub id: u64,
    pub properties: HashMap<String, fireworkkit::osvalue::OSValue>,
    pub children: Vec<fireworkkit::osdtentry::OSDTEntry>,
}

pub struct SystemState {
    pub kern_symbols: Option<&'static [sulphur_dioxide::KernSymbol]>,
    pub verbose: bool,
    pub serial_enabled: bool,
    pub pmm: Option<spin::Mutex<BitmapAllocator>>,
    pub pml4: Option<&'static mut PageTableLvl4>,
    pub terminal: Option<Terminal>,
    pub acpi: Option<ACPIState>,
    pub madt: Option<spin::Mutex<MADTData>>,
    pub lapic: Option<LocalAPIC>,
    pub scheduler: Option<spin::Mutex<Scheduler>>,
    pub interrupt_context: Option<super::RegisterState>,
    pub in_panic: core::sync::atomic::AtomicBool,
    pub dt_index: Option<spin::RwLock<HashMap<u64, spin::Mutex<OSDTEntry>>>>,
    pub dt_id_gen: Option<spin::Mutex<IncrementalIDGen>>,
    pub fkcache: Option<spin::Mutex<fireworkkit::FKCache>>,
}

impl SystemState {
    #[inline]
    pub const fn new() -> Self {
        Self {
            kern_symbols: None,
            verbose: cfg!(debug_assertions),
            serial_enabled: false,
            pmm: None,
            pml4: None,
            terminal: None,
            acpi: None,
            madt: None,
            lapic: None,
            scheduler: None,
            interrupt_context: None,
            in_panic: core::sync::atomic::AtomicBool::new(false),
            dt_index: None,
            dt_id_gen: None,
            fkcache: None,
        }
    }
}
