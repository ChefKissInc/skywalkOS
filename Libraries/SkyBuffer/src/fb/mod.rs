// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub mod shapes;

#[derive(Debug, PartialEq, Eq)]
pub struct FrameBuffer {
    pub base: *mut u32,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: crate::pixel::PixelFormat,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FBError {
    OutOfBounds,
}

pub type Result<T> = core::result::Result<T, FBError>;

impl FrameBuffer {
    /// # Safety
    /// The caller must ensure that the memory address is valid and that the width, height and stride values are within bounds of the memory data.
    #[inline]
    pub const unsafe fn new(
        base: *mut u32,
        width: usize,
        height: usize,
        stride: usize,
        format: crate::pixel::PixelFormat,
    ) -> Self {
        Self {
            base,
            width,
            height,
            stride,
            format,
        }
    }

    pub fn clear(&mut self, colour: u32) {
        for i in 0..self.height * self.stride {
            unsafe { self.base.add(i).write_volatile(colour) };
        }
    }

    pub fn plot_pixel(&mut self, x: usize, y: usize, colour: u32) -> Result<()> {
        if x >= self.width || y >= self.height {
            Err(FBError::OutOfBounds)
        } else {
            unsafe { self.base.add(x + self.stride * y).write_volatile(colour) };

            Ok(())
        }
    }
}
