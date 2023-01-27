// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use sulphur_dioxide::MemoryEntry;

pub struct BitmapAllocator {
    bitmap: &'static mut [u64],
    highest_addr: u64,
    pub free_pages: u64,
    pub total_pages: u64,
    last_index: u64,
}

impl BitmapAllocator {
    #[inline]
    #[must_use]
    pub fn new(mmap: &'static [MemoryEntry]) -> Self {
        let mut highest_addr = 0;

        // Find the highest available address
        for mmap_ent in mmap {
            match mmap_ent {
                MemoryEntry::Usable(v) | MemoryEntry::BootLoaderReclaimable(v) => {
                    let top = v.base + v.length;
                    trace!("{:X?}, top: {:#X?}", mmap_ent, top);

                    if top > highest_addr {
                        highest_addr = top;
                    }
                }
                _ => {}
            }
        }

        let bitmap_sz = (highest_addr / 0x1000) / 8;
        trace!(
            "highest_page: {:#X?}, bitmap_sz: {:#X?}",
            highest_addr,
            bitmap_sz
        );

        let mut bitmap = Default::default();

        // Find a memory hole for the bitmap
        for mmap_ent in mmap {
            if let MemoryEntry::Usable(v) = mmap_ent {
                if v.length >= bitmap_sz {
                    bitmap = unsafe {
                        core::slice::from_raw_parts_mut(
                            (v.base + amd64::paging::PHYS_VIRT_OFFSET) as *mut _,
                            bitmap_sz as _,
                        )
                    };
                    bitmap.fill(!0u64);

                    break;
                }
            }
        }

        let mut free_pages = 0;

        // Populate the bitmap
        for mmap_ent in mmap {
            if let MemoryEntry::Usable(v) = mmap_ent {
                trace!("Base: {:#X?}, End: {:#X?}", v.base, v.base + v.length);

                let v = if v.base == bitmap.as_ptr() as u64 {
                    let mut v = *v;
                    v.base += bitmap_sz;
                    v.length -= bitmap_sz;
                    v
                } else {
                    *v
                };

                if (v.base / 0x1000) > 512 {
                    free_pages += v.length / 0x1000;
                }

                for i in 0..(v.length / 0x1000) {
                    crate::utils::bitmap::bit_reset(bitmap, (v.base + (i * 0x1000)) / 0x1000);
                }
            }
        }

        // Set all regions under 2 MiB as reserved cause firmware stores stuff here
        for i in 0..512 {
            crate::utils::bitmap::bit_set(bitmap, i);
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
        let mut p = 0;

        while self.last_index < limit {
            let set = crate::utils::bitmap::bit_test(self.bitmap, self.last_index);
            self.last_index += 1;
            if set {
                p = 0;
            } else {
                p += 1;

                if p == count {
                    let page = self.last_index - count;

                    // Mark memory hole as used
                    for i in page..self.last_index {
                        crate::utils::bitmap::bit_set(self.bitmap, i);
                    }

                    self.free_pages -= count;

                    return Some((page * 0x1000) as *mut _);
                }
            }
        }

        None
    }

    pub unsafe fn alloc(&mut self, count: u64) -> Option<*mut u8> {
        let l = self.last_index;

        self.internal_alloc(count, self.highest_addr / 0x1000).or({
            self.last_index = 0;
            self.internal_alloc(count, l)
        })
    }

    pub unsafe fn free(&mut self, ptr: *mut u8, count: u64) {
        let idx = ptr as u64 / 0x1000;

        // Mark memory hole as free
        for i in idx..(idx + count) {
            crate::utils::bitmap::bit_reset(self.bitmap, i);
        }

        self.free_pages += count;
    }
}
