// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use modular_bitfield::prelude::*;

pub const PHYS_VIRT_OFFSET: u64 = 0xFFFF_8000_0000_0000;
pub const KERNEL_VIRT_OFFSET: u64 = 0xFFFF_FFFF_8000_0000;

#[derive(Debug)]
pub struct PageTableOffsets {
    pub pml4: u64,
    pub pdp: u64,
    pub pd: u64,
    pub pt: u64,
}

impl PageTableOffsets {
    #[inline]
    #[must_use]
    pub const fn new(virtual_address: u64) -> Self {
        Self {
            pml4: (virtual_address >> 39) & 0x1FF,
            pdp: (virtual_address >> 30) & 0x1FF,
            pd: (virtual_address >> 21) & 0x1FF,
            pt: (virtual_address >> 12) & 0x1FF,
        }
    }
}

#[bitfield(bits = 64)]
#[repr(u64)]
#[derive(Debug, Default, Clone, Copy)]
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
    pub available_to_os: B3,
    pub address: B40,
    pub available_to_os2: B11,
    pub no_execute: bool,
}

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

type AllocEntryFn<'a> = &'a dyn Fn() -> u64;

impl PageTable {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
        }
    }

    #[inline]
    unsafe fn set_and_return(
        &mut self,
        offset: u64,
        flags: PageTableEntry,
        phys: u64,
        virt_off: u64,
    ) -> &mut Self {
        let entry = &mut self.entries[offset as usize];
        *entry = flags.with_address(phys >> 12);

        &mut *(((entry.address() << 12) + virt_off) as *mut Self)
    }

    #[inline]
    unsafe fn get(&self, offset: u64, virt_off: u64) -> Option<&mut Self> {
        let entry = &self.entries[offset as usize];

        if entry.present() {
            return Some(&mut *(((entry.address() << 12) + virt_off) as *mut Self));
        }

        None
    }

    #[inline]
    pub unsafe fn set(&mut self, virt_off: u64) {
        core::arch::asm!("mov cr3, {}", in(reg) self as *mut _ as u64 - virt_off, options(nostack, preserves_flags));
    }

    #[inline]
    #[must_use]
    pub unsafe fn get_from_cr3() -> &'static mut Self {
        let pml4: *mut Self;
        core::arch::asm!("mov {}, cr3", out(reg) pml4, options(nostack, preserves_flags));
        &mut *pml4
    }

    pub unsafe fn map_pages(
        &mut self,
        alloc_entry: AllocEntryFn,
        virt: u64,
        virt_off: u64,
        phys: u64,
        count: u64,
        flags: PageTableEntry,
    ) {
        for i in 0..count {
            let phys = phys + 0x1000 * i;
            let virt = virt + 0x1000 * i;
            let offs = PageTableOffsets::new(virt);
            let pdp = if let Some(v) = self.get(offs.pml4, virt_off) {
                v
            } else {
                let address = alloc_entry();
                self.set_and_return(offs.pml4, flags, address, virt_off)
            };
            let pd = if let Some(v) = pdp.get(offs.pdp, virt_off) {
                v
            } else {
                let address = alloc_entry();
                pdp.set_and_return(offs.pdp, flags, address, virt_off)
            };
            let pt = if let Some(v) = pd.get(offs.pd, virt_off) {
                v
            } else {
                let address = alloc_entry();
                pd.set_and_return(offs.pd, flags, address, virt_off)
            };
            pt.entries[offs.pt as usize] = flags.with_address(phys >> 12);
        }
    }

    pub unsafe fn unmap_pages(&mut self, virt: u64, virt_off: u64, count: u64) {
        for i in 0..count {
            let virt = virt + 0x1000 * i;
            let offs = PageTableOffsets::new(virt);
            let Some(pdp) = self.get(offs.pml4, virt_off) else {
                continue;
            };
            let Some(pd) = pdp.get(offs.pdp, virt_off) else {
                continue;
            };
            let Some(pt) = pd.get(offs.pd, virt_off) else {
                continue;
            };
            pt.entries[offs.pt as usize] = PageTableEntry::new();
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }
    }

    pub unsafe fn map_huge_pages(
        &mut self,
        alloc_entry: AllocEntryFn,
        virt: u64,
        virt_off: u64,
        phys: u64,
        count: u64,
        flags: PageTableEntry,
    ) {
        for i in 0..count {
            let phys = phys + 0x20_0000 * i;
            let virt = virt + 0x20_0000 * i;
            let offs = PageTableOffsets::new(virt);
            let pdp = if let Some(v) = self.get(offs.pml4, virt_off) {
                v
            } else {
                let address = alloc_entry();
                self.set_and_return(offs.pml4, flags, address, virt_off)
            };
            let pd = if let Some(v) = pdp.get(offs.pdp, virt_off) {
                v
            } else {
                let address = alloc_entry();
                pdp.set_and_return(offs.pdp, flags, address, virt_off)
            };
            pd.entries[offs.pd as usize] = flags.with_huge_or_pat(true).with_address(phys >> 12);
        }
    }

    pub unsafe fn unmap_huge_pages(&mut self, virt: u64, virt_off: u64, count: u64) -> bool {
        for i in 0..count {
            let virt = virt + 0x20_0000 * i;
            let offs = PageTableOffsets::new(virt);
            let Some(pdp) = self.get(offs.pml4, virt_off) else {
                return false;
            };
            let Some(pd) = pdp.get(offs.pdp, virt_off) else {
                return false;
            };
            pd.entries[offs.pt as usize] = PageTableEntry::new();
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }

        true
    }

    pub unsafe fn map_higher_half(&mut self, alloc_entry: AllocEntryFn, virt_off: u64) {
        self.map_huge_pages(
            alloc_entry,
            PHYS_VIRT_OFFSET,
            virt_off,
            0,
            2048,
            PageTableEntry::new().with_present(true).with_writable(true),
        );
        self.map_huge_pages(
            alloc_entry,
            KERNEL_VIRT_OFFSET,
            virt_off,
            0,
            1024,
            PageTableEntry::new().with_present(true).with_writable(true),
        );
    }
}
