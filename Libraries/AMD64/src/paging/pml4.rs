// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

pub trait PML4: Sized {
    const VIRT_OFF: u64;

    #[must_use]
    fn get_entry(&mut self, offset: u64) -> &mut super::PageTableEntry;
    #[must_use]
    fn alloc_entry(&self) -> u64;

    unsafe fn set(&mut self) {
        core::arch::asm!("mov cr3, {}", in(reg) self as *mut _ as u64 - Self::VIRT_OFF, options(nostack, preserves_flags));
    }

    #[must_use]
    unsafe fn get() -> &'static mut Self {
        let pml4: *mut Self;
        core::arch::asm!("mov {}, cr3", out(reg) pml4, options(nostack, preserves_flags));
        &mut *pml4
    }

    #[must_use]
    unsafe fn get_or_alloc_entry(
        &mut self,
        offset: u64,
        flags: super::PageTableEntry,
    ) -> &mut Self {
        if !self.get_entry(offset).present() {
            *self.get_entry(offset) = flags.with_address(self.alloc_entry() >> 12);
        }

        &mut *(((self.get_entry(offset).address() << 12) + Self::VIRT_OFF) as *mut Self)
    }

    unsafe fn get_or_null_entry(&mut self, offset: u64) -> Option<&mut Self> {
        let entry = self.get_entry(offset);

        if entry.present() {
            Some(&mut *(((entry.address() << 12) + Self::VIRT_OFF) as *mut Self))
        } else {
            None
        }
    }

    unsafe fn virt_to_entry_addr(&mut self, virt: u64) -> Option<u64> {
        let offs = super::PageTableOffsets::new(virt);
        let pdp = self.get_or_null_entry(offs.pml4)?;
        let pd = pdp.get_or_null_entry(offs.pdp)?;

        if pd.get_entry(offs.pd).huge_or_pat() {
            Some(pd.get_entry(offs.pd).address() << 12)
        } else {
            let pt = pd.get_or_null_entry(offs.pd)?;

            if pt.get_entry(offs.pt).present() {
                Some(pt.get_entry(offs.pt).address() << 12)
            } else {
                None
            }
        }
    }

    unsafe fn map_pages(&mut self, virt: u64, phys: u64, count: u64, flags: super::PageTableEntry) {
        for i in 0..count {
            let phys = phys + 0x1000 * i;
            let virt = virt + 0x1000 * i;
            let offs = super::PageTableOffsets::new(virt);
            let pdp = self.get_or_alloc_entry(offs.pml4, flags);
            let pd = pdp.get_or_alloc_entry(offs.pdp, flags);
            let pt = pd.get_or_alloc_entry(offs.pd, flags);
            *pt.get_entry(offs.pt) = flags.with_address(phys >> 12);
        }
    }

    unsafe fn unmap_pages(&mut self, virt: u64, count: u64) -> bool {
        for i in 0..count {
            let virt = virt + 0x1000 * i;
            let offs = super::PageTableOffsets::new(virt);
            let Some(pdp) = self.get_or_null_entry(offs.pml4) else { return false; };
            let Some(pd) = pdp.get_or_null_entry(offs.pdp) else { return false; };
            let Some(pt) = pd.get_or_null_entry(offs.pd) else { return false; };
            *pt.get_entry(offs.pt) = super::PageTableEntry::new();
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }

        true
    }

    unsafe fn map_huge_pages(
        &mut self,
        virt: u64,
        phys: u64,
        count: u64,
        flags: super::PageTableEntry,
    ) {
        for i in 0..count {
            let phys = phys + 0x20_0000 * i;
            let virt = virt + 0x20_0000 * i;
            let offs = super::PageTableOffsets::new(virt);
            let pdp = self.get_or_alloc_entry(offs.pml4, flags);
            let pd = pdp.get_or_alloc_entry(offs.pdp, flags);
            *pd.get_entry(offs.pd) = flags.with_huge_or_pat(true).with_address(phys >> 12);
        }
    }

    unsafe fn unmap_huge_pages(&mut self, virt: u64, count: u64) -> bool {
        for i in 0..count {
            let virt = virt + 0x20_0000 * i;
            let offs = super::PageTableOffsets::new(virt);
            let Some(pdp) = self.get_or_null_entry(offs.pml4) else { return false; };
            let Some(pd) = pdp.get_or_null_entry(offs.pdp) else { return false; };
            *pd.get_entry(offs.pt) = super::PageTableEntry::new();
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }

        true
    }

    unsafe fn map_higher_half(&mut self) {
        self.map_huge_pages(
            super::PHYS_VIRT_OFFSET,
            0,
            2048,
            super::PageTableEntry::new()
                .with_present(true)
                .with_writable(true),
        );
        self.map_huge_pages(
            super::KERNEL_VIRT_OFFSET,
            0,
            1024,
            super::PageTableEntry::new()
                .with_present(true)
                .with_writable(true),
        );
    }
}
