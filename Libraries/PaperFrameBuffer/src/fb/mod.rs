// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub mod shapes;

#[derive(Debug, PartialEq, Eq)]
pub struct FrameBuffer {
    pub base: &'static mut [u32],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bitmask: crate::pixel::PixelBitMask,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FBError {
    OutOfBounds,
}

pub type Result<T> = core::result::Result<T, FBError>;

impl FrameBuffer {
    #[inline]
    pub unsafe fn new(
        data: *mut u32,
        width: usize,
        height: usize,
        stride: usize,
        bitmask: crate::pixel::PixelBitMask,
    ) -> Self {
        Self {
            base: core::slice::from_raw_parts_mut(data, height * stride),
            width,
            height,
            stride,
            bitmask,
        }
    }

    pub fn clear(&mut self, colour: u32) {
        self.base.fill(colour);
    }

    pub fn plot_pixel(&mut self, x: usize, y: usize, colour: u32) -> Result<()> {
        if x >= self.width || y >= self.height {
            Err(FBError::OutOfBounds)
        } else {
            self.base[x + self.stride * y] = colour;

            Ok(())
        }
    }
}
