// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PixelFormat {
    RGB,
    BGR,
    RGBA,
    BGRA,
    BitMask {
        r: u32,
        g: u32,
        b: u32,
        a: Option<u32>,
    },
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
    pub const fn as_u32(self, format: PixelFormat) -> u32 {
        let (r, g, b, a) = (self.r as u32, self.g as u32, self.b as u32, self.a as u32);

        let (r_mask, r_shift, g_mask, g_shift, b_mask, b_shift, a_mask_shift) = match format {
            PixelFormat::BGR => (0xFF0000, 16, 0xFF00, 8, 0xFF, 0, None),
            PixelFormat::RGB => (0xFF, 0, 0xFF00, 8, 0xFF0000, 16, None),
            PixelFormat::BGRA => (0xFF0000, 16, 0xFF00, 8, 0xFF, 0, Some((0xFF000000, 24))),
            PixelFormat::RGBA => (0xFF, 0, 0xFF00, 8, 0xFF0000, 16, Some((0xFF000000, 24))),
            PixelFormat::BitMask {
                r: r_mask,
                g: g_mask,
                b: b_mask,
                a: a_mask,
            } => (
                r_mask,
                r_mask.trailing_zeros(),
                g_mask,
                g_mask.trailing_zeros(),
                b_mask,
                b_mask.trailing_zeros(),
                if let Some(v) = a_mask {
                    Some((v, v.trailing_zeros()))
                } else {
                    None
                },
            ),
        };

        if let Some((a_mask, a_shift)) = a_mask_shift {
            ((r << r_shift) & r_mask)
                | ((g << g_shift) & g_mask)
                | ((b << b_shift) & b_mask)
                | ((a << a_shift) & a_mask)
        } else {
            ((((r * a) / 255) << r_shift) & r_mask)
                | ((((g * a) / 255) << g_shift) & g_mask)
                | ((((b * a) / 255) << b_shift) & b_mask)
        }
    }
}
