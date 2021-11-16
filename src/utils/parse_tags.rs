/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use kaboom::tags::TagType;
use log::info;

pub fn parse_tags(tags: &'static [kaboom::tags::TagType]) {
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
                        .init(*mmap);
                }
            }
            TagType::FrameBuffer(frame_buffer) => info!("Got frame-buffer: {:X?}", *frame_buffer),
            TagType::Acpi(rsdp) => info!("Got ACPI RSDP: {:X?}", *rsdp),
        }
    }
}
