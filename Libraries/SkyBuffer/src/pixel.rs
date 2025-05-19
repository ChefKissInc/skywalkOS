// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PixelFormat {
    RGB,
    BGR,
    RGBA,
    BGRA,
    Custom {
        r: u32,
        g: u32,
        b: u32,
        a: Option<u32>,
    },
}

impl PixelFormat {
    #[inline]
    pub const fn is_valid_component(v: u32) -> bool {
        32 - (v.leading_zeros() + v.trailing_zeros()) <= 8
    }

    #[inline]
    pub const fn from_bitmasks(r: u32, g: u32, b: u32, a: Option<u32>) -> Self {
        Self::Custom {
            r: r.trailing_zeros(),
            g: g.trailing_zeros(),
            b: b.trailing_zeros(),
            a: if let Some(a) = a {
                Some(a.trailing_zeros())
            } else {
                None
            },
        }
    }
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
        let Self { r, g, b, a } = self;
        let (r, g, b, a) = (r as u32, g as u32, b as u32, a as u32);

        let (r_shift, g_shift, b_shift, a_shift) = match format {
            PixelFormat::BGR => (16, 8, 0, None),
            PixelFormat::RGB => (0, 8, 16, None),
            PixelFormat::BGRA => (16, 8, 0, Some(24)),
            PixelFormat::RGBA => (0, 8, 16, Some(24)),
            PixelFormat::Custom { r, g, b, a } => (r, g, b, a),
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
