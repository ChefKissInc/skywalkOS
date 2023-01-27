// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Bitmask {
    pub r: u32,
    pub g: u32,
    pub b: u32,
    pub a: u32,
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

    /// Turns this Colour into raw pixel data.
    /// This operation is expensive, as it turns the bit-mask into a bit offset

    #[must_use]
    pub const fn to_u32(&self, bitmask: Bitmask) -> u32 {
        let red_pixel = bitmask.r.leading_zeros();
        let green_pixel = bitmask.g.leading_zeros();
        let blue_pixel = bitmask.b.leading_zeros();
        let alpha_pixel = bitmask.a.leading_zeros();

        ((self.r as u32) << red_pixel)
            | ((self.g as u32) << green_pixel)
            | ((self.b as u32) << blue_pixel)
            | ((self.a as u32) << alpha_pixel)
    }
}
