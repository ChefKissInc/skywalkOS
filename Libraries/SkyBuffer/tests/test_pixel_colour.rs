// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[test]
pub fn color_rgba() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x54)
            .as_u32(skybuffer::pixel::PixelBitMask::RGBA),
        0x54CDABFF
    )
}

#[test]
pub fn color_bgra() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x54)
            .as_u32(skybuffer::pixel::PixelBitMask::BGRA),
        0x54FFABCD
    )
}
