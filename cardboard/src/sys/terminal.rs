// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::fmt::Write;

use amd64::paging::pml4::PML4;
use paper_fb::{framebuffer::Framebuffer, pixel::Colour};

pub struct Terminal {
    pub x: usize,
    pub y: usize,
    pub fb: Framebuffer,
    pub width: usize,
    pub height: usize,
}

unsafe impl Send for Terminal {}
unsafe impl Sync for Terminal {}

impl Terminal {
    pub const fn new(fb: Framebuffer) -> Self {
        let width = fb.width / 8;
        let height = fb.height / 8;
        Self {
            x: 0,
            y: 0,
            fb,
            width,
            height,
        }
    }

    pub fn map_fb(&self) {
        unsafe {
            let state = super::state::SYS_STATE.get().as_mut().unwrap();
            let base = self.fb.base.as_ptr() as u64;
            state.pml4.get_mut().unwrap().map_huge_pages(
                base,
                base - amd64::paging::PHYS_VIRT_OFFSET,
                ((self.fb.height * self.fb.stride + 0x20_0000 - 1) / 0x20_0000) as _,
                amd64::paging::PageTableEntry::new()
                    .with_writable(true)
                    .with_present(true)
                    .with_pcd(true),
            );
        }
    }

    pub fn clear(&mut self) {
        self.fb.clear(0);
        self.x = 0;
        self.y = 0;
    }

    pub fn draw_char(&mut self, c: char, colour: Colour) {
        let x = self.x * 8;
        let mut y = self.y * 8;
        let Some(v) = font8x8::legacy::BASIC_LEGACY.get(c as usize) else {
            return;
        };
        for &x_bit in v {
            for bit in 0..8 {
                if x_bit & (1 << bit) != 0 {
                    self.fb
                        .plot_pixel(x + bit, y, colour.to_u32(self.fb.bitmask))
                        .unwrap();
                }
            }
            y += 1;
        }
    }

    pub fn handle_scrollback(&mut self) {
        if self.y >= self.height {
            self.fb
                .base
                .copy_within(self.fb.stride * 8..self.fb.stride * self.fb.height, 0);
            self.fb.base[self.fb.stride * (self.fb.height - 8)..].fill(0);
            self.y -= 1;
            self.x = 0;
        }
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.y += 1;
                self.x = 0;
                self.handle_scrollback();
            } else {
                self.draw_char(c, Colour::new(0xFF, 0xFF, 0xFF, 0xFF));
                self.x += 1;
                if self.x >= self.width {
                    self.y += 1;
                    self.x = 0;
                    self.handle_scrollback();
                }
            }
        }
        Ok(())
    }
}
