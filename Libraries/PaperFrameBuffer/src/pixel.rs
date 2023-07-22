// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PixelBitMask {
    RGBA,
    BGRA,
    Custom { r: u32, g: u32, b: u32, a: u32 },
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Colour {
    #[inline]
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    pub const fn as_u32(&self, bitmask: PixelBitMask) -> u32 {
        let (r_shift, g_shift, b_shift, a_shift) = match bitmask {
            PixelBitMask::BGRA => (16, 8, 0, 24),
            PixelBitMask::RGBA => (0, 8, 16, 24),
            PixelBitMask::Custom { r, g, b, a } => (
                r.leading_zeros(),
                g.leading_zeros(),
                b.leading_zeros(),
                a.leading_zeros(),
            ),
        };

        ((self.r as u32) << r_shift)
            | ((self.g as u32) << g_shift)
            | ((self.b as u32) << b_shift)
            | ((self.a as u32) << a_shift)
    }
}
