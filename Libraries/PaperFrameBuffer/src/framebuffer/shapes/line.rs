// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

impl crate::framebuffer::Framebuffer {
    /// # Errors
    ///
    /// This operation errors if the X and Y coordinates or, depending on the `horizontal` argument,
    /// the X or Y coordinate plus the length of the line are larger than the screen bounds

    pub fn draw_line(
        &mut self,
        x: usize,
        y: usize,
        len: usize,
        horizontal: bool,
        colour: u32,
    ) -> crate::framebuffer::Result<()> {
        if x + len >= self.width || y + len >= self.height {
            Err(crate::framebuffer::FramebufferError::OutOfBounds)
        } else {
            if horizontal {
                for i in 0..len {
                    self.plot_pixel(x + i, y, colour)?;
                }
            } else {
                for i in 0..len {
                    self.plot_pixel(x, y + i, colour)?;
                }
            }

            Ok(())
        }
    }
}
