/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use kaboom::tags::memory_map::MemoryEntry;
use log::info;

extern "C" {
    static __kernel_top: u64;
}

#[inline]
unsafe fn bit_test(bitmap: &mut [u64], index: usize) -> bool {
    (bitmap[index / 64] & (1u64 << (index % 64))) != 0
}

#[inline]
unsafe fn bit_set(bitmap: &mut [u64], index: usize) {
    bitmap[index / 64] |= 1u64 << (index % 64);
}

#[inline]
unsafe fn bit_reset(bitmap: &mut [u64], index: usize) {
    bitmap[index / 64] &= !(1u64 << (index % 64));
}

#[derive(Debug)]
pub struct BitmapAllocator {
    bitmap: &'static mut [u64],
    highest_page: usize,
    last_index: usize,
}

impl BitmapAllocator {
    pub const fn new() -> Self {
        Self {
            bitmap: &mut [],
            highest_page: 0,
            last_index: 0,
        }
    }

    pub unsafe fn init(&mut self, mmap: &'static [MemoryEntry]) {
        let alloc_base = &__kernel_top as *const _ as usize - amd64::paging::KERNEL_VIRT_OFFSET;
        info!("alloc_base: {:#X?}", alloc_base);

        // Find the highest available address
        for mmap_ent in mmap {
            match mmap_ent {
                MemoryEntry::Usable(v) | MemoryEntry::BootLoaderReclaimable(v) => {
                    let top = v.base + v.length;
                    info!("{:X?}, top: {:#X?}", mmap_ent, top);

                    if top > self.highest_page {
                        self.highest_page = top;
                    }
                }
                _ => {}
            }
        }

        let bitmap_sz = (self.highest_page / 0x1000) / 8;
        info!(
            "highest_page: {:#X?}, bitmap_sz: {:#X?}",
            self.highest_page, bitmap_sz
        );

        // Find a memory hole for the bitmap
        for mmap_ent in mmap {
            if let MemoryEntry::Usable(v) = mmap_ent {
                if v.length >= bitmap_sz {
                    self.bitmap = core::slice::from_raw_parts_mut(
                        (v.base + amd64::paging::PHYS_VIRT_OFFSET) as *mut _,
                        bitmap_sz,
                    );
                    self.bitmap.fill(!0u64);

                    break;
                }
            }
        }

        // Populate the bitmap
        for mmap_ent in mmap {
            if let MemoryEntry::Usable(v) = mmap_ent {
                info!("Base: {:#X?}, End: {:#X?}", v.base, v.base + v.length);

                let v = if v.base == self.bitmap.as_ptr() as usize {
                    let mut v = *v;
                    v.base += bitmap_sz;
                    v.length -= bitmap_sz;
                    v
                } else {
                    *v
                };

                for i in 0..(v.length / 0x1000) {
                    bit_reset(self.bitmap, (v.base + (i * 0x1000)) / 0x1000);
                }
            }
        }

        // Set all regions under 2 MiB as reserved cause firmware stores stuff here
        for i in 0..512 {
            bit_set(self.bitmap, i);
        }

        // info!("{:X?}", self.bitmap);
    }

    unsafe fn internal_alloc(&mut self, count: usize, limit: usize) -> Option<*mut u8> {
        // info!("{:X?}", self);
        let mut p = 0;

        while self.last_index < limit {
            let set = bit_test(self.bitmap, self.last_index);
            self.last_index += 1;
            if !set {
                p += 1;

                if p == count {
                    let page = self.last_index - count;

                    // Mark memory hole as used
                    for i in page..self.last_index {
                        bit_set(self.bitmap, i);
                    }

                    return Some(core::mem::transmute(page * 0x1000));
                }
            } else {
                p = 0;
            }
        }

        None
    }

    pub unsafe fn alloc(&mut self, count: usize) -> Option<*mut u8> {
        let l = self.last_index;

        if let Some(ret) = self.internal_alloc(count, self.highest_page / 0x1000) {
            Some(ret)
        } else {
            self.last_index = 0;
            self.internal_alloc(count, l)
        }
    }

    pub unsafe fn free(&mut self, ptr: *mut u8, count: usize) {
        // info!("{:X?}", self);
        let idx = ptr as usize / 0x1000;

        // Mark memory hole as free
        for i in idx..(idx + count) {
            bit_reset(self.bitmap, i);
        }
    }
}
