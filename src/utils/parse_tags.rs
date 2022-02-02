/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use kaboom::tags::TagType;
use log::info;

pub fn parse_tags(
    tags: &'static [kaboom::tags::TagType],
) -> Option<vesa::framebuffer::Framebuffer> {
    let mut fb = None;
    for tag in tags {
        match tag {
            TagType::CommandLine(cmdline) => info!("Found command line arguments: {}", *cmdline),
            TagType::MemoryMap(mmap) => {
                info!("Got memory map: {:X?}", *mmap);

                unsafe {
                    crate::sys::allocator::GLOBAL_ALLOCATOR
                        .0
                        .get()
                        .as_mut()
                        .unwrap()
                        .init(mmap);
                }
            }
            TagType::FrameBuffer(fb_info) => {
                info!("Got frame-buffer: {:X?}", *fb_info);
                fb = Some(vesa::framebuffer::Framebuffer::new(
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
            }
            TagType::Acpi(rsdp) => info!("Got ACPI RSDP: {:X?}", *rsdp),
        }
    }
    fb
}
