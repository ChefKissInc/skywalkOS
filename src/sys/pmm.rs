/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use core::cell::UnsafeCell;

use log::info;

extern "C" {
    static __kernel_top: u64;
}

#[derive(Debug)]
pub struct BitmapAllocator {
    bitmap: *mut i64,
    highest_page: usize,
    last_index: usize,
}

impl BitmapAllocator {
    pub const fn new() -> Self {
        Self {
            bitmap: core::ptr::null_mut(),
            highest_page: 0,
            last_index: 0,
        }
    }

    pub unsafe fn init(
        &mut self,
        mmap: &'static [UnsafeCell<kaboom::tags::memory_map::MemoryEntry>],
    ) {
        let alloc_base = &__kernel_top as *const _ as usize - amd64::paging::KERNEL_VIRT_OFFSET;
        info!("alloc_base: {:#X?}", alloc_base);

        // Find the highest available address
        for mmap_ent in mmap {
            match mmap_ent.get().as_mut().unwrap() {
                kaboom::tags::memory_map::MemoryEntry::Usable(v)
                | kaboom::tags::memory_map::MemoryEntry::BootLoaderReclaimable(v) => {
                    let top = v.base + v.length;
                    info!(
                        "v.base: {:#X?}, v.pages: {:#X?}, top: {:#X?}",
                        v.base, v.length, top
                    );

                    if v.base < alloc_base {
                        if top > alloc_base {
                            v.length -= alloc_base - v.base;
                            v.base = alloc_base;
                        } else {
                            mmap_ent
                                .get()
                                .write(kaboom::tags::memory_map::MemoryEntry::BadMemory(*v));

                            continue;
                        }
                    }

                    if top > self.highest_page {
                        self.highest_page = top as usize;
                    }
                }
                _ => {}
            }
        }

        let bitmap_sz = self.highest_page / 0x1000 / 8;
        info!(
            "highest_page: {:#X?}, bitmap_sz: {:#X?}",
            self.highest_page, bitmap_sz
        );

        // Find a memory hole for the bitmap
        for mmap_ent in mmap {
            if let kaboom::tags::memory_map::MemoryEntry::Usable(v) =
                mmap_ent.get().as_mut().unwrap()
            {
                if v.length >= bitmap_sz {
                    self.bitmap = (v.base + amd64::paging::PHYS_VIRT_OFFSET) as *mut i64;

                    mmap_ent
                        .get()
                        .write(kaboom::tags::memory_map::MemoryEntry::Usable(
                            kaboom::tags::memory_map::MemoryData {
                                base: v.base + bitmap_sz,
                                length: v.length - bitmap_sz,
                            },
                        ));

                    self.bitmap.write_bytes(0xFF, bitmap_sz);

                    break;
                }
            }
        }

        // Populate the bitmap
        for mmap_ent in mmap {
            if let kaboom::tags::memory_map::MemoryEntry::Usable(v) =
                mmap_ent.get().as_mut().unwrap()
            {
                info!("Base: {:#X?}, End: {:#X?}", v.base, v.base + v.length);

                for j in 0..(v.length / 0x1000) {
                    core::arch::x86_64::_bittestandreset64(
                        self.bitmap,
                        ((v.base + j * 0x1000) / 0x1000).try_into().unwrap(),
                    );
                }
            }
        }

        // let mut serial = super::io::serial::SERIAL.lock();
        // for i in 0..bitmap_sz {
        //     write!(serial, "{:b}", self.bitmap.add(i).read()).unwrap();
        // }
        info!("");
    }

    unsafe fn internal_alloc(&mut self, count: usize, limit: usize) -> Option<*mut u8> {
        let mut p = 0usize;

        while self.last_index < limit {
            let res =
                core::arch::x86_64::_bittest64(self.bitmap, self.last_index.try_into().unwrap())
                    == 0;
            self.last_index += 1;
            if res {
                p += 1;

                if p == count {
                    let page = self.last_index - count;

                    // Mark memory hole as used
                    for i in page..self.last_index {
                        core::arch::x86_64::_bittestandset64(self.bitmap, i.try_into().unwrap());
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
        let idx = ptr as usize / 0x1000;

        // Mark memory hole as free
        for i in idx..(idx + count) {
            core::arch::x86_64::_bittestandreset64(self.bitmap, i.try_into().unwrap());
        }
    }
}
