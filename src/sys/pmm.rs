/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use core::cell::UnsafeCell;

use amd64::utilities::alignment::Alignment;
use log::info;
// use log::{debug, info};

extern "C" {
    static __kernel_top: u64;
}

#[derive(Debug)]
pub enum AllocInitError {
    UnknownError,
}

pub struct BitmapAllocator {
    bitmap: &'static mut [u64],
    highest_page: usize,
    last_index: usize,
}

impl BitmapAllocator {
    pub fn new(
        mmap: &'static [UnsafeCell<kaboom::tags::MemoryEntry>],
    ) -> Result<Self, AllocInitError> {
        // SAFETY: Linker symbol; immutable
        let alloc_base =
            unsafe { &__kernel_top } as *const _ as u64 - amd64::paging::KERNEL_VIRT_OFFSET;
        info!("alloc_base: {}", alloc_base);

        // Find the highest available address
        // SAFETY: Uhhhhh...uhhhhhhh...uhhhhhhhhhhhh
        // end my suffering
        let mut highest_page = 0u64;

        for mmap_ent in mmap {
            if let kaboom::tags::MemoryEntry::Usable(v) = unsafe { &*mmap_ent.get() } {
                let mut base = v.base.align_up(0x1000);
                let mut pages = v.pages - ((base - v.base) / 0x1000).align_down(0x1000);
                let top = base + pages;
                info!(
                    "Base: {}, v.base: {}, pages: {}, v.pages: {}, top: {}",
                    base, v.base, pages, v.pages, top
                );

                if base < alloc_base {
                    if top > alloc_base {
                        pages -= alloc_base - base;
                        base = alloc_base;
                    } else {
                        unsafe {
                            mmap_ent
                                .get()
                                .write(kaboom::tags::MemoryEntry::BadMemory(*v));
                        }

                        continue;
                    }
                }

                unsafe {
                    mmap_ent.get().write(kaboom::tags::MemoryEntry::Usable(
                        kaboom::tags::MemoryData { base, pages },
                    ));
                }

                if top > highest_page {
                    highest_page = top;
                }
            }
        }

        info!("Memory Map after processing part 1: {:X?}", mmap);

        let bitmap_sz = (highest_page / 0x1000) / 8;
        info!("highest_page: {}, bitmap_sz: {}", highest_page, bitmap_sz);

        // Find a memory hole for the bitmap
        for mmap_ent in mmap {
            if let kaboom::tags::MemoryEntry::Usable(v) = unsafe { *mmap_ent.get() } {
                if v.pages >= bitmap_sz {
                    unsafe {
                        let ptr = (v.base + amd64::paging::PHYS_VIRT_OFFSET) as *mut u64;
                        let bitmap = core::slice::from_raw_parts_mut(ptr, bitmap_sz as usize);
                        info!("Ptr: {:#X?}", ptr);
                        info!(":confused_blink: {:#X?}", mmap_ent.get());
                        info!(":confused_blink: {:#X?}", mmap_ent.get().read());

                        mmap_ent.get().write(kaboom::tags::MemoryEntry::Usable(
                            kaboom::tags::MemoryData {
                                base: v.base + bitmap_sz,
                                pages: v.pages - bitmap_sz,
                            },
                        ));

                        for i in 0..bitmap_sz {
                            bitmap.as_mut_ptr().add(i as usize).write(!0u64);
                        }

                        info!("Memory Map after processing part 2: {:X?}", mmap);

                        // Populate the bitmap
                        for mmap_ent in mmap {
                            if let kaboom::tags::MemoryEntry::Usable(v) = &*mmap_ent.get() {
                                let base = v.base as usize / 0x1000;
                                let end = base + v.pages as usize;
                                info!("Base: {}, End: {}", base, end);

                                // I don't understand how this works
                                for j in 0..(v.pages) {
                                    let idx = (v.base + j * 0x1000) as usize / 0x1000;
                                    bitmap[idx / 64] &= !(1u64 << (idx % 64));
                                }
                            }
                        }

                        // if cfg!(debug_assertions) {
                        //     for i in 0..bitmap_sz {
                        //         debug!(
                        //             "{:b}",
                        //             bitmap.as_mut_ptr().add(i as usize).as_ref().unwrap()
                        //         );
                        //     }
                        // }

                        return Ok(Self {
                            bitmap,
                            highest_page: highest_page.try_into().unwrap(),
                            last_index: 0,
                        });
                    }
                }
            }
        }

        Err(AllocInitError::UnknownError)
    }

    unsafe fn internal_alloc(&mut self, count: usize, limit: usize) -> Result<*mut u8, ()> {
        let mut p = 0usize;

        while self.last_index < limit {
            self.last_index += 1;

            if (self.bitmap[self.last_index / 64] & !(1u64 << (self.last_index % 64))) == 0 {
                p += 1;

                if p == count {
                    let page = self.last_index - count;

                    // Mark memory hole as used
                    for i in page..self.last_index {
                        self.bitmap[i / 64] |= 1u64 << (i % 64);
                    }

                    return Ok(core::mem::transmute(page * 0x1000));
                }
            } else {
                p = 0;
            }
        }

        Err(())
    }

    pub unsafe fn alloc(&mut self, count: usize) -> Result<*mut u8, ()> {
        let l = self.last_index;

        if let Ok(ret) = self.internal_alloc(count, self.highest_page / 0x1000) {
            Ok(ret)
        } else {
            self.last_index = 0;
            self.internal_alloc(count, l)
        }
    }

    pub unsafe fn free(&mut self, ptr: *mut u8, count: usize) {
        let idx = ptr as usize / 0x1000;

        // Mark memory hole as free
        for i in idx..idx + count {
            self.bitmap[i / 64] &= !(1u64 << (i % 64));
        }
    }
}
