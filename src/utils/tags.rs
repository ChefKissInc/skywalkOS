//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::vec::Vec;

use kaboom::tags::TagType;
use log::debug;

use crate::{
    sys::{pmm::BitmapAllocator, terminal::Terminal},
    Acpi,
};

pub fn parse(tags: &'static [kaboom::tags::TagType]) {
    let state = unsafe { &mut *crate::sys::state::SYS_STATE.get() };

    for tag in tags {
        match tag {
            TagType::SpecialisedSettings(settings) => {
                debug!("Got boot settings: {:X?}", settings);
                state.boot_settings = *settings
            }
            TagType::MemoryMap(mmap) => {
                debug!("Got memory map: {:X?}", *mmap);
                state.pmm.write(BitmapAllocator::new(mmap));
            }
            TagType::FrameBuffer(fb_info) => {
                debug!("Got boot display: {:X?}", *fb_info);
                let mut terminal = Terminal::new(vesa::framebuffer::Framebuffer::new(
                    fb_info.base,
                    fb_info.resolution.width as usize,
                    fb_info.resolution.height as usize,
                    vesa::pixel::Bitmask {
                        r: fb_info.pixel_bitmask.red,
                        g: fb_info.pixel_bitmask.green,
                        b: fb_info.pixel_bitmask.blue,
                        a: fb_info.pixel_bitmask.alpha,
                    },
                    fb_info.pitch,
                ));
                terminal.clear();
                state.terminal = Some(terminal);
            }
            TagType::Acpi(rsdp) => {
                debug!("Got ACPI RSDP: {:X?}", rsdp);
                state.acpi.write(Acpi::new(*rsdp));
            }
            TagType::Module(module) => {
                debug!("Got module '{}' of type {:#X?}", module.name, module.type_);
                if state.modules.is_none() {
                    state.modules = Some(Vec::new());
                }
                state.modules.as_mut().unwrap().push(*module);
            }
        }
    }
}
