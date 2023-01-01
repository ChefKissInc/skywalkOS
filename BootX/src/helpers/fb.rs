// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::boxed::Box;

use sulphur_dioxide::fb::{FrameBufferInfo, PixelBitmask, PixelFormat, ScreenRes};
use uefi::{proto::console::gop::GraphicsOutput, table::boot::ScopedProtocol};

pub fn fbinfo_from_gop(mut gop: ScopedProtocol<GraphicsOutput>) -> Box<FrameBufferInfo> {
    Box::new(FrameBufferInfo {
        base: (gop.frame_buffer().as_mut_ptr() as u64 + amd64::paging::PHYS_VIRT_OFFSET)
            as *mut u32,
        resolution: ScreenRes::new(gop.current_mode_info().resolution()),
        pixel_format: match gop.current_mode_info().pixel_format() {
            uefi::proto::console::gop::PixelFormat::Rgb => PixelFormat::RedGreenBlue,
            uefi::proto::console::gop::PixelFormat::Bgr => PixelFormat::BlueGreenRed,
            uefi::proto::console::gop::PixelFormat::Bitmask => PixelFormat::Bitmask,
            uefi::proto::console::gop::PixelFormat::BltOnly => panic!(),
        },
        pixel_bitmask: match gop.current_mode_info().pixel_format() {
            uefi::proto::console::gop::PixelFormat::Rgb => PixelBitmask {
                red: 0xFF00_0000,
                green: 0x00FF_0000,
                blue: 0x0000_FF00,
                alpha: 0x0000_00FF,
            },
            uefi::proto::console::gop::PixelFormat::Bgr => PixelBitmask {
                red: 0x0000_FF00,
                green: 0x00FF_0000,
                blue: 0xFF00_0000,
                alpha: 0x0000_00FF,
            },
            uefi::proto::console::gop::PixelFormat::Bitmask => gop
                .current_mode_info()
                .pixel_bitmask()
                .map(|v| PixelBitmask {
                    red: v.red,
                    green: v.green,
                    blue: v.blue,
                    alpha: v.reserved,
                })
                .unwrap(),
            uefi::proto::console::gop::PixelFormat::BltOnly => {
                panic!("Blt-only mode not supported.")
            }
        },
        pitch: gop.current_mode_info().stride(),
    })
}
