// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use amd64::paging::PAGE_SIZE;
use skyliftkit::{MemoryData, MemoryEntry};

pub struct BitmapAllocator {
    bitmap: &'static mut [u64],
    highest_addr: u64,
    pub free_pages: u64,
    last_index: u64,
}

impl BitmapAllocator {
    #[inline]
    pub fn new(mmap: &'static [MemoryEntry]) -> Self {
        let highest_addr = mmap
            .iter()
            .flat_map(|v| {
                let MemoryEntry::Usable(v) = v else {
                    return None;
                };
                Some(v.base + v.length)
            })
            .max()
            .unwrap();

        let bitmap_sz = (highest_addr / PAGE_SIZE).div_ceil(8);
        trace!(
            "Highest usable address: {highest_addr:#X}, Bitmap size: {bitmap_sz} bytes, {} \
             entries",
            bitmap_sz / 8
        );

        let mut bitmap: &mut [u64] = Default::default();

        let mut free_pages = 0;

        for v in mmap {
            trace!("{v:X?}");

            let MemoryEntry::Usable(v) = v else {
                continue;
            };

            let v = {
                if v.length == 0 {
                    continue;
                }
                if v.base <= 0x20_0000 {
                    if v.base + v.length > 0x20_0000 {
                        MemoryData::new(0x20_0000, v.length - 0x20_0000)
                    } else {
                        continue;
                    }
                } else if bitmap.is_empty() && v.length >= bitmap_sz {
                    bitmap = unsafe {
                        core::slice::from_raw_parts_mut(
                            (v.base + amd64::paging::PHYS_VIRT_OFFSET) as *mut _,
                            (bitmap_sz / 8) as _,
                        )
                    };
                    bitmap.fill(!0u64);

                    trace!("Placing bitmap at {:#X}", v.base);
                    MemoryData::new(
                        (v.base + bitmap_sz + (PAGE_SIZE - 1)) & !PAGE_SIZE,
                        (v.length - bitmap_sz + (PAGE_SIZE - 1)) & !PAGE_SIZE,
                    )
                } else {
                    *v
                }
            };

            let (base, count) = (v.base / PAGE_SIZE, v.length / PAGE_SIZE);
            for i in base..(base + count) {
                crate::bitmap::bit_reset(bitmap, i);
            }
            free_pages += count;
        }

        assert!(!bitmap.is_empty());

        Self {
            bitmap,
            highest_addr,
            free_pages,
            last_index: 0,
        }
    }

    unsafe fn internal_alloc(&mut self, count: u64, limit: u64) -> *mut u8 {
        let mut n = 0;

        while self.last_index < limit {
            let set = crate::bitmap::bit_test(self.bitmap, self.last_index);
            self.last_index += 1;

            if set {
                n = 0;
                continue;
            }

            n += 1;

            if n == count {
                let page = self.last_index - count;

                for i in page..self.last_index {
                    crate::bitmap::bit_set(self.bitmap, i);
                }

                self.free_pages -= count;

                return (page * PAGE_SIZE) as *mut _;
            }
        }

        core::ptr::null_mut()
    }

    pub unsafe fn alloc(&mut self, count: u64) -> *mut u8 {
        let l = self.last_index;

        let ret = self.internal_alloc(count, self.highest_addr / PAGE_SIZE);
        if ret.is_null() {
            self.last_index = 0;
            self.internal_alloc(count, l)
        } else {
            ret
        }
    }

    pub unsafe fn free(&mut self, ptr: *mut u8, count: u64) {
        assert_eq!(ptr as u64 & (PAGE_SIZE - 1), 0);

        let idx = ptr as u64 / PAGE_SIZE;

        for i in idx..(idx + count) {
            crate::bitmap::bit_reset(self.bitmap, i);
        }

        self.free_pages += count;
    }

    pub fn is_allocated(&self, ptr: *mut u8, count: u64) -> bool {
        assert_eq!(ptr as u64 & (PAGE_SIZE - 1), 0);

        let idx = ptr as u64 / PAGE_SIZE;

        for i in idx..(idx + count) {
            if !crate::bitmap::bit_test(self.bitmap, i) {
                return false;
            }
        }

        true
    }
}
