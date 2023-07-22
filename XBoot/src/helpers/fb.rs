// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::boxed::Box;

use paper_fb::pixel::PixelBitMask;
use sulphur_dioxide::{FrameBufferInfo, ScreenRes};
use uefi::{
    proto::console::gop::{GraphicsOutput, PixelFormat},
    table::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol},
};

fn fbinfo_from_gop(mut gop: ScopedProtocol<GraphicsOutput>) -> Option<Box<FrameBufferInfo>> {
    let mode_info = gop.current_mode_info();
    let pixel_bitmask = match mode_info.pixel_format() {
        PixelFormat::Rgb => PixelBitMask::RGBA,
        PixelFormat::Bgr => PixelBitMask::BGRA,
        PixelFormat::Bitmask => {
            gop.current_mode_info()
                .pixel_bitmask()
                .map(|v| PixelBitMask::Custom {
                    r: v.red,
                    g: v.green,
                    b: v.blue,
                    a: v.reserved,
                })?
        }
        PixelFormat::BltOnly => {
            return None;
        }
    };
    Some(Box::new(FrameBufferInfo {
        base: unsafe {
            gop.frame_buffer()
                .as_mut_ptr()
                .add(amd64::paging::PHYS_VIRT_OFFSET as _)
                .cast::<u32>()
        },
        resolution: ScreenRes::new(mode_info.resolution()),
        pixel_bitmask,
        pitch: gop.current_mode_info().stride(),
    }))
}

pub fn init() -> Option<Box<FrameBufferInfo>> {
    let bs = unsafe { uefi_services::system_table().as_mut().boot_services() };
    let handle = match bs.get_handle_for_protocol::<GraphicsOutput>() {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to get handle for GOP: {e}.");
            return None;
        }
    };
    let mut gop: ScopedProtocol<GraphicsOutput> = unsafe {
        bs.open_protocol(
            OpenProtocolParams {
                handle,
                agent: bs.image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
        .unwrap()
    };
    let mode = gop
        .modes()
        .filter(|v| v.info().pixel_format() != PixelFormat::BltOnly)
        .max_by_key(|v| v.info().resolution().0)?;
    if let Err(e) = gop.set_mode(&mode) {
        warn!("Failed to set mode: {e}.");
    }
    fbinfo_from_gop(gop)
}
