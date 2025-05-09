// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

impl crate::fb::FrameBuffer {
    pub fn draw_line(
        &mut self,
        x: usize,
        y: usize,
        len: usize,
        horizontal: bool,
        colour: u32,
    ) -> crate::fb::Result<()> {
        if x + len >= self.width || y + len >= self.height {
            Err(crate::fb::FBError::OutOfBounds)
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
