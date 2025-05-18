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
        if self.r == 0 && self.g == 0 && self.b == 0 {
            return 0;
        }
        let (r, g, b, a) = (self.r as u32, self.g as u32, self.b as u32, self.a as u32);

        let (r_shift, g_shift, b_shift, a_shift) = match format {
            PixelFormat::BGR => (16, 8, 0, None),
            PixelFormat::RGB => (0, 8, 16, None),
            PixelFormat::BGRA => (16, 8, 0, Some(24)),
            PixelFormat::RGBA => (0, 8, 16, Some(24)),
            PixelFormat::BitMask {
                r: r_mask,
                g: g_mask,
                b: b_mask,
                a: a_mask,
            } => {
                let (r_shift, g_shift, b_shift, a_mask_shift) = (
                    r_mask.trailing_zeros(),
                    g_mask.trailing_zeros(),
                    b_mask.trailing_zeros(),
                    if let Some(v) = a_mask {
                        Some((v, v.trailing_zeros()))
                    } else {
                        None
                    },
                );
                return if let Some((a_mask, a_shift)) = a_mask_shift {
                    ((r << r_shift) & r_mask)
                        | ((g << g_shift) & g_mask)
                        | ((b << b_shift) & b_mask)
                        | ((a << a_shift) & a_mask)
                } else {
                    ((((r * a) / 255) << r_shift) & r_mask)
                        | ((((g * a) / 255) << g_shift) & g_mask)
                        | ((((b * a) / 255) << b_shift) & b_mask)
                };
            }
        };

        if let Some(a_shift) = a_shift {
            (r << r_shift) | (g << g_shift) | (b << b_shift) | (a << a_shift)
        } else {
            (((r * a) / 255) << r_shift)
                | (((g * a) / 255) << g_shift)
                | (((b * a) / 255) << b_shift)
        }
    }
}
