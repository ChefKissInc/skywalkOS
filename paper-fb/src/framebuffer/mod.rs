// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

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
    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
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
    #[inline]
    pub fn clear(&mut self, colour: u32) {
        self.base.fill(colour);
    }

    /// Plot a pixel at a specified coordinate on the frame-buffer
    /// # Errors
    ///
    /// This operation errors when X and Y coordinates are outside the screen boundaries
    #[inline]
    pub fn plot_pixel(&mut self, x: usize, y: usize, colour: u32) -> Result<()> {
        if x >= self.width || y >= self.height {
            Err(FramebufferError::OutOfBounds)
        } else {
            self.base[x + self.stride * y] = colour;

            Ok(())
        }
    }
}
