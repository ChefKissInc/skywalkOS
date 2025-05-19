// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::boxed::Box;

use skybuffer::pixel::PixelFormat;
use skyliftkit::{FrameBufferInfo, ScreenRes};
use uefi::{
    boot::ScopedProtocol,
    proto::console::gop::{GraphicsOutput, ModeInfo, PixelFormat as GopPixelFormat},
};

#[inline]
fn is_bad_mode(gop: &ScopedProtocol<GraphicsOutput>, mode_info: &ModeInfo) -> bool {
    match mode_info.pixel_format() {
        GopPixelFormat::Bitmask => gop
            .current_mode_info()
            .pixel_bitmask()
            .is_none_or(|bitmask| {
                !PixelFormat::is_valid_component(bitmask.red)
                    || !PixelFormat::is_valid_component(bitmask.green)
                    || !PixelFormat::is_valid_component(bitmask.blue)
            }),
        GopPixelFormat::BltOnly => true,
        _ => false,
    }
}

#[inline]
fn fbinfo_from_gop(mut gop: ScopedProtocol<GraphicsOutput>) -> Option<Box<FrameBufferInfo>> {
    let mode_info = gop.current_mode_info();
    let pixel_format = match mode_info.pixel_format() {
        GopPixelFormat::Rgb => PixelFormat::RGB,
        GopPixelFormat::Bgr => PixelFormat::BGR,
        GopPixelFormat::Bitmask => {
            let Some(bitmask) = gop.current_mode_info().pixel_bitmask() else {
                unreachable!()
            };
            // Just in case the firmware is shit.
            match (bitmask.red, bitmask.green, bitmask.blue) {
                (0xFF, 0xFF00, 0xFF0000) => PixelFormat::RGB,
                (0xFF0000, 0xFF00, 0xFF) => PixelFormat::BGR,
                (r, g, b) => PixelFormat::from_bitmasks(r, g, b, None),
            }
        }
        GopPixelFormat::BltOnly => {
            return None;
        }
    };
    Some(Box::new(FrameBufferInfo {
        base: gop
            .frame_buffer()
            .as_mut_ptr()
            .map_addr(|v| v + amd64::paging::PHYS_VIRT_OFFSET as usize)
            .cast(),
        resolution: ScreenRes::new(mode_info.resolution()),
        pixel_format,
        pitch: gop.current_mode_info().stride(),
    }))
}

pub fn init() -> Option<Box<FrameBufferInfo>> {
    let Ok(handle) = uefi::boot::get_handle_for_protocol::<GraphicsOutput>() else {
        return None;
    };
    let mut gop: ScopedProtocol<GraphicsOutput> =
        uefi::boot::open_protocol_exclusive(handle).ok()?;
    if is_bad_mode(&gop, &gop.current_mode_info()) {
        let mode = gop
            .modes()
            .filter(|v| !is_bad_mode(&gop, v.info()))
            .max_by_key(|v| v.info().resolution().0)?;
        if let Err(e) = gop.set_mode(&mode) {
            warn!("Failed to set mode: {e}.");
        }
    }
    fbinfo_from_gop(gop)
}
