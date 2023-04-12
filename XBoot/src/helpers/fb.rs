// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::boxed::Box;

use paper_fb::pixel::PixelBitMask;
use sulphur_dioxide::{FrameBufferInfo, ScreenRes};
use uefi::{proto::console::gop::GraphicsOutput, table::boot::ScopedProtocol};

pub fn fbinfo_from_gop(mut gop: ScopedProtocol<GraphicsOutput>) -> Box<FrameBufferInfo> {
    Box::new(FrameBufferInfo {
        base: unsafe {
            gop.frame_buffer()
                .as_mut_ptr()
                .add(amd64::paging::PHYS_VIRT_OFFSET as _)
                .cast::<u32>()
        },
        resolution: ScreenRes::new(gop.current_mode_info().resolution()),
        pixel_bitmask: match gop.current_mode_info().pixel_format() {
            uefi::proto::console::gop::PixelFormat::Rgb => PixelBitMask::RGBA,
            uefi::proto::console::gop::PixelFormat::Bgr => PixelBitMask::BGRA,
            uefi::proto::console::gop::PixelFormat::Bitmask => gop
                .current_mode_info()
                .pixel_bitmask()
                .map(|v| PixelBitMask::Custom {
                    r: v.red,
                    g: v.green,
                    b: v.blue,
                    a: v.reserved,
                })
                .unwrap(),
            uefi::proto::console::gop::PixelFormat::BltOnly => {
                panic!("Blt-only mode not supported.");
            }
        },
        pitch: gop.current_mode_info().stride(),
    })
}
