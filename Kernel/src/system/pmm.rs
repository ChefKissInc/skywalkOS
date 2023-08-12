// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use sulphur_dioxide::{MemoryData, MemoryEntry};

pub struct BitmapAllocator {
    bitmap: &'static mut [u64],
    highest_addr: u64,
    pub free_pages: u64,
    pub total_pages: u64,
    last_index: u64,
}

impl BitmapAllocator {
    #[inline]
    pub fn new(mmap: &'static [MemoryEntry]) -> Self {
        let mut highest_addr = 0;

        for mmap_ent in mmap {
            let MemoryEntry::Usable(v) = mmap_ent else {
                continue;
            };

            let top = v.base + v.length;
            debug!("{mmap_ent:X?}, top: {top:#X?}");

            if top > highest_addr {
                highest_addr = top;
            }
        }

        let bitmap_sz = (highest_addr / 0x1000) / 8 + 8;
        debug!("Highest usable address: {highest_addr:#X?}, Bitmap size: {bitmap_sz} bytes");

        let mut bitmap = Default::default();

        for mmap_ent in mmap {
            let MemoryEntry::Usable(v) = mmap_ent else {
                continue;
            };

            if v.length >= bitmap_sz {
                bitmap = unsafe {
                    core::slice::from_raw_parts_mut(
                        (v.base + amd64::paging::PHYS_VIRT_OFFSET) as *mut _,
                        (bitmap_sz / 8) as _,
                    )
                };
                bitmap.fill(!0u64);

                break;
            }
        }

        let mut free_pages = 0;

        for mmap_ent in mmap {
            let MemoryEntry::Usable(v) = mmap_ent else {
                continue;
            };

            debug!("Base: {:#X?}, End: {:#X?}", v.base, v.base + v.length);

            let v = if v.base == (bitmap.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET) {
                trace!("Bitmap is here");
                MemoryData::new(v.base + bitmap_sz, v.length - bitmap_sz)
            } else {
                *v
            };

            if v.length == 0 {
                continue;
            }

            let base = v.base / 0x1000;

            let count = v.length / 0x1000;
            for i in base..base + count {
                crate::utils::bitmap::bit_reset(bitmap, i);
            }
            free_pages += count;
        }

        let total_pages = highest_addr / 0x1000;
        Self {
            bitmap,
            highest_addr,
            total_pages,
            free_pages,
            last_index: 0,
        }
    }

    unsafe fn internal_alloc(&mut self, count: u64, limit: u64) -> Option<*mut u8> {
        let mut n = 0;

        while self.last_index < limit {
            let set = crate::utils::bitmap::bit_test(self.bitmap, self.last_index);
            self.last_index += 1;
            if set {
                n = 0;
                continue;
            }

            n += 1;

            if n == count {
                let page = self.last_index - count;

                for i in page..self.last_index {
                    crate::utils::bitmap::bit_set(self.bitmap, i);
                }

                self.free_pages -= count;

                return Some((page * 0x1000) as *mut _);
            }
        }

        None
    }

    pub unsafe fn alloc(&mut self, count: u64) -> Option<*mut u8> {
        let l = self.last_index;

        self.internal_alloc(count, self.highest_addr / 0x1000)
            .or_else(|| {
                self.last_index = 0;
                self.internal_alloc(count, l)
            })
    }

    pub unsafe fn free(&mut self, ptr: *mut u8, count: u64) {
        let idx = ptr as u64 / 0x1000;

        for i in idx..(idx + count) {
            crate::utils::bitmap::bit_reset(self.bitmap, i);
        }

        self.free_pages += count;
    }

    pub fn is_allocated(&self, ptr: *mut u8, count: u64) -> bool {
        let idx = ptr as u64 / 0x1000;

        for i in idx..(idx + count) {
            if !crate::utils::bitmap::bit_test(self.bitmap, i) {
                return false;
            }
        }

        true
    }
}
