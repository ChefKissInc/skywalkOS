// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

pub mod shapes;

#[derive(Debug, PartialEq, Eq)]
pub struct Framebuffer {
    pub base: &'static mut [u32],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bitmask: crate::pixel::Bitmask,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FramebufferError {
    OutOfBounds,
}

pub type Result<T> = core::result::Result<T, FramebufferError>;

impl Framebuffer {
    #[inline]
    #[must_use]
    pub unsafe fn new(
        data: *mut u32,
        width: usize,
        height: usize,
        stride: usize,
        bitmask: crate::pixel::Bitmask,
    ) -> Self {
        Self {
            base: core::slice::from_raw_parts_mut(data, height * stride),
            width,
            height,
            stride,
            bitmask,
        }
    }

    /// Clears the entire frame-buffer contents with the specified colour

    pub fn clear(&mut self, colour: u32) {
        self.base.fill(colour);
    }

    /// Plot a pixel at a specified coordinate on the frame-buffer
    /// # Errors
    ///
    /// This operation errors when X and Y coordinates are outside the screen boundaries

    pub fn plot_pixel(&mut self, x: usize, y: usize, colour: u32) -> Result<()> {
        if x >= self.width || y >= self.height {
            Err(FramebufferError::OutOfBounds)
        } else {
            self.base[x + self.stride * y] = colour;

            Ok(())
        }
    }
}
