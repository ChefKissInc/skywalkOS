// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#[test]
pub fn color_rgba() {
    assert_eq!(
        paper_fb::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x54)
            .as_u32(paper_fb::pixel::PixelBitMask::RGBA),
        0x54CDABFF
    )
}

#[test]
pub fn color_bgra() {
    assert_eq!(
        paper_fb::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x54)
            .as_u32(paper_fb::pixel::PixelBitMask::BGRA),
        0x54FFABCD
    )
}
