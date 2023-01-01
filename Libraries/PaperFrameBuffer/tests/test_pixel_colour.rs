// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[test]
pub fn color_rgba() {
    assert_eq!(
        paper_fb::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x54).to_u32(paper_fb::pixel::Bitmask {
            r: 0xFF000000,
            g: 0x00FF0000,
            b: 0x0000FF00,
            a: 0x000000FF
        }),
        0x54CDABFF
    )
}

#[test]
pub fn color_bgra() {
    assert_eq!(
        paper_fb::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x54).to_u32(paper_fb::pixel::Bitmask {
            r: 0x0000FF00,
            g: 0x00FF0000,
            b: 0xFF000000,
            a: 0x000000FF
        }),
        0x54FFABCD
    )
}
