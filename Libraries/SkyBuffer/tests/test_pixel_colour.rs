// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[test]
pub fn color_component_valid() {
    assert!(skybuffer::pixel::PixelFormat::is_valid_component(0xFF));
    assert!(skybuffer::pixel::PixelFormat::is_valid_component(0xFF00));
    assert!(skybuffer::pixel::PixelFormat::is_valid_component(0xFF0000));
    assert!(skybuffer::pixel::PixelFormat::is_valid_component(
        0xFF000000
    ));

    assert!(skybuffer::pixel::PixelFormat::is_valid_component(0x0F));
    assert!(skybuffer::pixel::PixelFormat::is_valid_component(0x0F00));
    assert!(skybuffer::pixel::PixelFormat::is_valid_component(0x0F0000));
    assert!(skybuffer::pixel::PixelFormat::is_valid_component(
        0x0F000000
    ));
}

#[test]
pub fn color_component_invalid() {
    assert!(!skybuffer::pixel::PixelFormat::is_valid_component(0xFFF));
    assert!(!skybuffer::pixel::PixelFormat::is_valid_component(0xFFF0));
    assert!(!skybuffer::pixel::PixelFormat::is_valid_component(0xFF0F00));
    assert!(!skybuffer::pixel::PixelFormat::is_valid_component(
        0x0F000F00
    ));
}

#[test]
pub fn color_rgb() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0xFF)
            .as_u32(skybuffer::pixel::PixelFormat::RGB),
        0x00CDABFF
    )
}

#[test]
pub fn color_bgr() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0xFF)
            .as_u32(skybuffer::pixel::PixelFormat::BGR),
        0x00FFABCD
    )
}

#[test]
pub fn color_rgb_alpha() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0xAA)
            .as_u32(skybuffer::pixel::PixelFormat::RGB),
        0x008872AA
    )
}

#[test]
pub fn color_bgr_alpha() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0xAA)
            .as_u32(skybuffer::pixel::PixelFormat::BGR),
        0x00AA7288
    )
}

#[test]
pub fn color_rgb_zero_alpha() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x00)
            .as_u32(skybuffer::pixel::PixelFormat::RGB),
        0x00000000
    )
}

#[test]
pub fn color_bgr_zero_alpha() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x00)
            .as_u32(skybuffer::pixel::PixelFormat::BGR),
        0x00000000
    )
}

#[test]
pub fn color_rgba() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x54)
            .as_u32(skybuffer::pixel::PixelFormat::RGBA),
        0x54CDABFF
    )
}

#[test]
pub fn color_bgra() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xAB, 0xCD, 0x54)
            .as_u32(skybuffer::pixel::PixelFormat::BGRA),
        0x54FFABCD
    )
}

#[test]
pub fn color_bitmask() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xB, 0xCD, 0x4).as_u32(
            skybuffer::pixel::PixelFormat::from_bitmasks(
                0x0000FF,
                0x000F00,
                0xFF0000,
                Some(0x00F000),
            ),
        ),
        0xCD4BFF
    )
}

#[test]
pub fn color_bitmask_alpha() {
    assert_eq!(
        skybuffer::pixel::Colour::new(0xFF, 0xB, 0xCD, 0x4).as_u32(
            skybuffer::pixel::PixelFormat::from_bitmasks(0x0000FF, 0x000F00, 0xFF0000, None),
        ),
        0x030004
    )
}
