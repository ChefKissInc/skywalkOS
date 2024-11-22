// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub const PAGE_SIZE: u64 = 0x1000;
pub const PAGE_MASK: u64 = 0xFFF;

pub const PHYS_VIRT_OFFSET: u64 = 0xFFFF_8000_0000_0000;
pub const KERNEL_VIRT_OFFSET: u64 = 0xFFFF_FFFF_8000_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageTableIndices {
    pub pml4: usize,
    pub pdp: usize,
    pub pd: usize,
    pub pt: usize,
}

impl PageTableIndices {
    #[inline]
    #[must_use]
    pub const fn new(addr: u64) -> Self {
        Self {
            pml4: ((addr >> 39) & 0x1FF) as usize,
            pdp: ((addr >> 30) & 0x1FF) as usize,
            pd: ((addr >> 21) & 0x1FF) as usize,
            pt: ((addr >> 12) & 0x1FF) as usize,
        }
    }
}

#[bitfield(u64)]
#[derive(PartialEq, Eq)]
pub struct PageTableEntry {
    pub present: bool,
    pub writable: bool,
    pub user: bool,
    pub pwt: bool,
    pub pcd: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub huge_or_pat: bool,
    pub global: bool,
    #[bits(2)]
    __: u8,
    pub pat: bool,
    #[bits(40)]
    pub address: u64,
    #[bits(11)]
    __: u16,
    pub no_execute: bool,
}

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct PageTable<const VIRT_OFF: u64> {
    pub entries: [PageTableEntry; 512],
}

type AllocEntryFn<'a> = &'a dyn Fn() -> u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub const fn as_entry(self, pte: bool) -> PageTableEntry {
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

    #[inline]
    pub fn update_entry(self, entry: &mut PageTableEntry, pte: bool) {
        let pat = (self.pat_index & 0b100) != 0;
        entry.set_present(entry.present() || self.present);
        entry.set_writable(entry.writable() || self.writable);
        entry.set_user(entry.user() || self.user);
        entry.set_pwt((self.pat_index & 0b001) != 0);
        entry.set_pcd((self.pat_index & 0b010) != 0);
        entry.set_huge_or_pat(pte && pat);
        entry.set_pat(!pte && pat);
    }

    #[inline]
    #[must_use]
    pub const fn from_entry(entry: &PageTableEntry, pte: bool) -> Self {
        Self::new()
            .with_present(entry.present())
            .with_writable(entry.writable())
            .with_user(entry.user())
            .with_pat_entry(
                (entry.pwt() as u8)
                    | ((entry.pcd() as u8) << 1)
                    | ((((entry.huge_or_pat() && pte) || entry.pat()) as u8) << 2),
            )
    }
}

impl Default for PageTableFlags {
    fn default() -> Self {
        Self::new()
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
    unsafe fn get(&self, offset: usize) -> Option<&mut Self> {
        let entry = &self.entries[offset];

        if entry.present() {
            return Some(&mut *(((entry.address() << 12) + VIRT_OFF) as *mut Self));
        }

        None
    }

    #[inline]
    #[must_use]
    unsafe fn get_and_update_or_alloc(
        &mut self,
        alloc_entry: AllocEntryFn,
        offset: usize,
        flags: PageTableFlags,
    ) -> &mut Self {
        let entry = &mut self.entries[offset];

        if entry.present() {
            flags.update_entry(entry, false);
        } else {
            *entry = flags
                .as_entry(false)
                .with_present(true)
                .with_address(alloc_entry() >> 12)
        };

        &mut *(((entry.address() << 12) + VIRT_OFF) as *mut Self)
    }

    #[inline]
    pub unsafe fn set_cr3(&mut self) {
        core::arch::asm!("mov cr3, {}", in(reg) self as *mut _ as u64 - VIRT_OFF, options(nostack, preserves_flags));
    }

    #[inline]
    #[must_use]
    pub unsafe fn from_cr3() -> &'static mut Self {
        let pml4: u64;
        core::arch::asm!("mov {}, cr3", out(reg) pml4, options(nostack, preserves_flags));
        &mut *((pml4 + VIRT_OFF) as *mut Self)
    }

    #[inline]
    pub unsafe fn virt_to_phys(&mut self, virt: u64) -> Option<(u64, PageTableFlags)> {
        let offs = PageTableIndices::new(virt);
        let pdp = self.get(offs.pml4)?;
        let pd = pdp.get(offs.pdp)?;
        let pt = pd.get(offs.pd)?;

        let ent = &pt.entries[offs.pt];
        if ent.present() {
            return Some((
                (ent.address() << 12) | (virt & PAGE_MASK),
                PageTableFlags::from_entry(ent, true),
            ));
        }

        None
    }

    #[inline]
    pub unsafe fn map(
        &mut self,
        alloc_entry: AllocEntryFn,
        virt: u64,
        phys: u64,
        count: u64,
        flags: PageTableFlags,
    ) {
        assert_ne!(count, 0);
        for (phys, virt) in (0..count).map(|i| (phys + PAGE_SIZE * i, virt + PAGE_SIZE * i)) {
            let offs = PageTableIndices::new(virt);
            let pdp = self.get_and_update_or_alloc(alloc_entry, offs.pml4, flags);
            let pd = pdp.get_and_update_or_alloc(alloc_entry, offs.pdp, flags);
            let pt = pd.get_and_update_or_alloc(alloc_entry, offs.pd, flags);
            pt.entries[offs.pt] = flags.as_entry(true).with_address(phys >> 12);
        }
    }

    #[inline]
    pub unsafe fn unmap(&mut self, virt: u64, count: u64) {
        assert_ne!(count, 0);
        for virt in (0..count).map(|i| virt + PAGE_SIZE * i) {
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
            let offs = PageTableIndices::new(virt);
            let pdp = self.get(offs.pml4).unwrap();
            let pd = pdp.get(offs.pdp).unwrap();
            let pt = pd.get(offs.pd).unwrap();
            assert!(pt.entries[offs.pt].present());
            pt.entries[offs.pt] = PageTableEntry::new();
        }
    }

    #[inline]
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

impl<const VIRT_OFF: u64> Default for PageTable<VIRT_OFF> {
    fn default() -> Self {
        Self::new()
    }
}
