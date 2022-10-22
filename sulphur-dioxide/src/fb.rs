// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[repr(C)]
#[derive(Debug)]
pub enum PixelFormat {
    RedGreenBlue,
    BlueGreenRed,
    Bitmask,
}

#[repr(C)]
#[derive(Debug)]
pub struct PixelBitmask {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub alpha: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct ScreenRes {
    pub width: usize,
    pub height: usize,
}

impl ScreenRes {
    #[must_use]
    pub const fn new(res: (usize, usize)) -> Self {
        Self {
            width: res.0,
            height: res.1,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FrameBufferInfo {
    pub resolution: ScreenRes,
    pub pixel_format: PixelFormat,
    pub pixel_bitmask: PixelBitmask,
    pub pitch: usize,
    pub base: *mut u32,
}
