//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::vec::Vec;

use kaboom::tags::TagType;
use log::info;

use crate::{
    sys::{pmm::BitmapAllocator, terminal::Terminal},
    Acpi,
};

pub fn parse_tags(tags: &'static [kaboom::tags::TagType]) {
    for tag in tags {
        match tag {
            TagType::CommandLine(cmdline) => info!("Found command line arguments: {}", *cmdline),
            TagType::MemoryMap(mmap) => {
                info!("Got memory map: {:X?}", *mmap);

                unsafe { crate::sys::state::SYS_STATE.pmm.get().as_mut().unwrap() }
                    .call_once(|| BitmapAllocator::new(mmap));
            }
            TagType::FrameBuffer(fb_info) => {
                info!("Got frame-buffer: {:X?}", *fb_info);
                unsafe {
                    crate::sys::state::SYS_STATE
                        .terminal
                        .get()
                        .as_mut()
                        .unwrap()
                }
                .call_once(|| {
                    Terminal::new(vesa::framebuffer::Framebuffer::new(
                        (fb_info.base as usize + amd64::paging::PHYS_VIRT_OFFSET) as *mut _,
                        fb_info.resolution.width as usize,
                        fb_info.resolution.height as usize,
                        vesa::pixel::Bitmask {
                            r: fb_info.pixel_bitmask.red,
                            g: fb_info.pixel_bitmask.green,
                            b: fb_info.pixel_bitmask.blue,
                            a: fb_info.pixel_bitmask.alpha,
                        },
                        fb_info.pitch,
                    ))
                });
            }
            TagType::Acpi(rsdp) => {
                info!("Got ACPI RSDP: {:X?}", rsdp);
                unsafe { crate::sys::state::SYS_STATE.acpi.get().as_mut().unwrap() }
                    .call_once(|| Acpi::new(*rsdp));
            }
            TagType::Module { name, size, addr } => {
                let modules =
                    unsafe { crate::sys::state::SYS_STATE.modules.get().as_mut().unwrap() };
                if modules.get().is_none() {
                    modules.call_once(Vec::new);
                }
                modules.get_mut().unwrap().push(crate::sys::state::Module {
                    name,
                    size: *size,
                    addr: *addr,
                });
            }
        }
    }
}
