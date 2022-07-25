//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;

use log::debug;
use sulfur_dioxide::tags::TagType;

use crate::{
    sys::{pmm::BitmapAllocator, terminal::Terminal},
    ACPIPlatform,
};

pub fn parse(tags: &'static [sulfur_dioxide::tags::TagType]) {
    let state = unsafe { &mut *crate::sys::state::SYS_STATE.get() };

    for tag in tags {
        match tag {
            TagType::SpecialisedSettings(settings) => {
                state.boot_settings = *settings;
                debug!("Got boot settings: {:X?}", settings);
            }
            TagType::MemoryMap(mmap) => {
                debug!("Got memory map: {:X?}", *mmap);
                state.pmm.write(BitmapAllocator::new(mmap));
            }
            TagType::FrameBuffer(fb_info) => {
                debug!("Got boot display: {:X?}", *fb_info);
                let mut terminal = Terminal::new(paper_fb::framebuffer::Framebuffer::new(
                    fb_info.base,
                    fb_info.resolution.width as usize,
                    fb_info.resolution.height as usize,
                    paper_fb::pixel::Bitmask {
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
            TagType::RSDPPtr(rsdp) => {
                debug!("Got ACPI RSDP: {:X?}", rsdp);
                state.acpi.write(ACPIPlatform::new(rsdp));
            }
            TagType::Module(module) => {
                debug!("Got module: {:#X?}", module);
                if state.modules.is_none() {
                    state.modules = Some(Vec::new());
                }
                state.modules.as_mut().unwrap().push(*module);
            }
        }
    }
}
