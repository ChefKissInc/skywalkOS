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
    for tag in tags {
        match tag {
            TagType::SpecialisedSettings(settings) => {
                debug!("Got boot settings: {:X?}", settings);
                crate::sys::state::SYS_STATE
                    .boot_settings
                    .call_once(|| *settings);
            }
            TagType::MemoryMap(mmap) => {
                debug!("Got memory map: {:X?}", *mmap);
                unsafe {
                    (&*crate::sys::state::SYS_STATE.pmm.get())
                        .call_once(|| BitmapAllocator::new(mmap));
                }
            }
            TagType::FrameBuffer(fb_info) => {
                debug!("Got boot display: {:X?}", *fb_info);
                let terminal = unsafe { &mut *crate::sys::state::SYS_STATE.terminal.get() };
                terminal.call_once(|| {
                    Terminal::new(vesa::framebuffer::Framebuffer::new(
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
                    ))
                });

                let terminal = terminal.get_mut().unwrap();
                terminal.clear();

                unsafe {
                    (&*crate::utils::logger::LOGGER.terminal.get()).call_once(|| terminal);
                }
            }
            TagType::Acpi(rsdp) => {
                debug!("Got ACPI RSDP: {:X?}", rsdp);
                unsafe {
                    (&*crate::sys::state::SYS_STATE.acpi.get()).call_once(|| Acpi::new(*rsdp));
                }
            }
            TagType::Module(module) => {
                debug!("Got module '{}' of type {:#X?}", module.name, module.type_);
                unsafe { &mut *crate::sys::state::SYS_STATE.modules.get() }.call_once(Vec::new);
                unsafe { &mut *crate::sys::state::SYS_STATE.modules.get() }
                    .get_mut()
                    .unwrap()
                    .push(*module);
            }
        }
    }
}
