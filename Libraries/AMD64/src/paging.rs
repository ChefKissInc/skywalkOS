// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use modular_bitfield::prelude::*;

pub const PAGE_SIZE: u64 = 0x1000;

pub const PHYS_VIRT_OFFSET: u64 = 0xFFFF_8000_0000_0000;
pub const KERNEL_VIRT_OFFSET: u64 = 0xFFFF_FFFF_8000_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageTableIndices {
    pub pml4: u64,
    pub pdp: u64,
    pub pd: u64,
    pub pt: u64,
}

impl PageTableIndices {
    #[inline]
    #[must_use]
    pub const fn new(addr: u64) -> Self {
        Self {
            pml4: (addr >> 39) & 0x1FF,
            pdp: (addr >> 30) & 0x1FF,
            pd: (addr >> 21) & 0x1FF,
            pt: (addr >> 12) & 0x1FF,
        }
    }
}

#[bitfield(bits = 64)]
#[repr(u64)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PageTableEntry {
    pub present: bool,
    pub writable: bool,
    pub user: bool,
    pub pwt: bool,
    pub pcd: bool,
    #[skip(setters)]
    pub accessed: bool,
    #[skip(setters)]
    pub dirty: bool,
    pub huge_or_pat: bool,
    pub global: bool,
    #[skip]
    __: B2,
    pub pat: bool,
    pub address: B40,
    #[skip]
    __: B11,
    pub no_execute: bool,
}

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct PageTable<const VIRT_OFF: u64> {
    pub entries: [PageTableEntry; 512],
}

type AllocEntryFn<'a> = &'a dyn Fn() -> u64;

#[derive(Debug, Clone, Copy)]
pub struct PageTableFlags {
    pub present: bool,
    pub writable: bool,
    pub user: bool,
    pub pat_index: u8,
}

impl PageTableFlags {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            present: false,
            writable: false,
            user: false,
            pat_index: 0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn with_present(mut self, present: bool) -> Self {
        self.present = present;
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_user(mut self, user: bool) -> Self {
        self.user = user;
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_pat_entry(mut self, pat_entry: u8) -> Self {
        assert!(pat_entry < 8);
        self.pat_index = pat_entry;
        self
    }

    #[inline]
    #[must_use]
    pub const fn new_present() -> Self {
        Self::new().with_present(true)
    }

    #[inline]
    #[must_use]
    pub fn as_entry(self, pte: bool) -> PageTableEntry {
        let pat = (self.pat_index & 0b100) != 0;
        PageTableEntry::new()
            .with_present(self.present)
            .with_writable(self.writable)
            .with_user(self.user)
            .with_pwt((self.pat_index & 0b001) != 0)
            .with_pcd((self.pat_index & 0b010) != 0)
            .with_huge_or_pat(pte && pat)
            .with_pat(!pte && pat)
    }
}

impl<const VIRT_OFF: u64> PageTable<VIRT_OFF> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
        }
    }

    #[inline]
    unsafe fn get(&self, offset: u64) -> Option<&mut Self> {
        let entry = &self.entries[offset as usize];

        if entry.present() {
            return Some(&mut *(((entry.address() << 12) + VIRT_OFF) as *mut Self));
        }

        None
    }

    #[inline]
    unsafe fn get_or_alloc(
        &mut self,
        alloc_entry: AllocEntryFn,
        offset: u64,
        flags: PageTableFlags,
    ) -> &mut Self {
        let entry = &mut self.entries[offset as usize];

        if !entry.present() {
            *entry = flags
                .as_entry(false)
                .with_present(true)
                .with_address(alloc_entry() >> 12);
        }

        &mut *(((entry.address() << 12) + VIRT_OFF) as *mut Self)
    }

    unsafe fn get_and_update_or_alloc(
        &mut self,
        alloc_entry: AllocEntryFn,
        offset: u64,
        flags: PageTableFlags,
    ) -> &mut Self {
        let entry = &mut self.entries[offset as usize];

        if !entry.present() {
            *entry = flags
                .as_entry(false)
                .with_present(true)
                .with_address(alloc_entry() >> 12);
        } else {
            *entry = flags
                .as_entry(false)
                .with_present(true)
                .with_address(entry.address());
        }

        &mut *(((entry.address() << 12) + VIRT_OFF) as *mut Self)
    }

    #[inline]
    pub unsafe fn set_cr3(&mut self) {
        core::arch::asm!("mov cr3, {}", in(reg) self as *mut _ as u64 - VIRT_OFF, options(nomem, nostack, preserves_flags));
    }

    #[inline]
    #[must_use]
    pub unsafe fn from_cr3() -> &'static mut Self {
        let pml4: *mut Self;
        core::arch::asm!("mov {}, cr3", out(reg) pml4, options(nomem, nostack, preserves_flags));
        &mut *pml4
    }

    pub unsafe fn virt_to_phys(&mut self, virt: u64) -> Option<u64> {
        let offs = PageTableIndices::new(virt);
        let pdp = self.get(offs.pml4)?;
        let pd = pdp.get(offs.pdp)?;
        let pt = pd.get(offs.pd)?;

        let ent = &pt.entries[offs.pt as usize];
        if ent.present() {
            return Some((ent.address() << 12) + (virt & 0xFFF));
        }

        None
    }

    pub unsafe fn map(
        &mut self,
        alloc_entry: AllocEntryFn,
        virt: u64,
        phys: u64,
        count: u64,
        flags: PageTableFlags,
    ) {
        for (phys, virt) in (0..count).map(|i| (phys + PAGE_SIZE * i, virt + PAGE_SIZE * i)) {
            let offs = PageTableIndices::new(virt);
            let pdp = self.get_or_alloc(alloc_entry, offs.pml4, flags);
            let pd = pdp.get_or_alloc(alloc_entry, offs.pdp, flags);
            let pt = pd.get_or_alloc(alloc_entry, offs.pd, flags);
            assert!(!pt.entries[offs.pt as usize].present());
            pt.entries[offs.pt as usize] = flags.as_entry(true).with_address(phys >> 12);
        }
    }

    pub unsafe fn map_or_update(
        &mut self,
        alloc_entry: AllocEntryFn,
        virt: u64,
        phys: u64,
        count: u64,
        flags: PageTableFlags,
    ) {
        for (phys, virt) in (0..count).map(|i| (phys + PAGE_SIZE * i, virt + PAGE_SIZE * i)) {
            let offs = PageTableIndices::new(virt);
            let pdp = self.get_and_update_or_alloc(alloc_entry, offs.pml4, flags);
            let pd = pdp.get_and_update_or_alloc(alloc_entry, offs.pdp, flags);
            let pt = pd.get_and_update_or_alloc(alloc_entry, offs.pd, flags);
            pt.entries[offs.pt as usize] = flags.as_entry(true).with_address(phys >> 12);
        }
    }

    pub unsafe fn unmap(&mut self, virt: u64, count: u64) -> bool {
        for virt in (0..count).map(|i| virt + PAGE_SIZE * i) {
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
            let offs = PageTableIndices::new(virt);
            let Some(pdp) = self.get(offs.pml4) else {
                return false;
            };
            let Some(pd) = pdp.get(offs.pdp) else {
                return false;
            };
            let Some(pt) = pd.get(offs.pd) else {
                return false;
            };
            assert!(pt.entries[offs.pt as usize].present());
            pt.entries[offs.pt as usize] = PageTableEntry::new();
        }

        true
    }

    pub unsafe fn map_higher_half(&mut self, alloc_entry: AllocEntryFn) {
        self.map(
            alloc_entry,
            PHYS_VIRT_OFFSET + PAGE_SIZE,
            PAGE_SIZE,
            0xFFFFF,
            PageTableFlags::new_present().with_writable(true),
        );
        self.map(
            alloc_entry,
            KERNEL_VIRT_OFFSET + PAGE_SIZE,
            PAGE_SIZE,
            0x7FFFF,
            PageTableFlags::new_present().with_writable(true),
        );
    }
}
