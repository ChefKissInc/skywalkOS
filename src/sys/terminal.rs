// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::fmt::Write;

use amd64::paging::pml4::PML4;
use paper_fb::{framebuffer::Framebuffer, pixel::Colour};

pub struct Terminal {
    x: usize,
    y: usize,
    fb: Framebuffer,
    pub width: usize,
    pub height: usize,
}

unsafe impl Send for Terminal {}
unsafe impl Sync for Terminal {}

impl Terminal {
    pub const fn new(fb: Framebuffer) -> Self {
        Self {
            x: 0,
            y: 0,
            fb,
            width: fb.width / 8,
            height: fb.height / 8,
        }
    }

    pub fn map_fb(&self) {
        unsafe {
            (*super::state::SYS_STATE.get())
                .pml4
                .assume_init_mut()
                .map_huge_pages(
                    self.fb.base as usize,
                    self.fb.base as usize - amd64::paging::PHYS_VIRT_OFFSET,
                    (self.fb.height * self.fb.pitch + 0x20_0000 - 1) / 0x20_0000,
                    amd64::paging::PageTableEntry::new()
                        .with_writable(true)
                        .with_present(true)
                        .with_pcd(true),
                );
        }
    }

    pub fn clear(&mut self) {
        self.fb.clear(0).unwrap();
        self.x = 0;
        self.y = 0;
    }

    pub fn draw_char(&mut self, c: char, colour: Colour) {
        let x = self.x * 8;
        let mut y = self.y * 8;
        for &x_bit in &font8x8::legacy::BASIC_LEGACY[c as usize] {
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

    pub fn backspace(&mut self) {
        if self.x > 0 {
            self.x -= 1;
        } else {
            self.y -= 1;
            self.x = self.width - 1;
        }

        for y in 0..8 {
            for x in 0..8 {
                self.fb
                    .plot_pixel((self.x * 8) + x, (self.y * 8) + y, 0)
                    .unwrap();
            }
        }
    }

    pub fn handle_scrollback(&mut self) {
        if self.y >= self.height {
            let off = self.fb.pitch * 8;
            let off_clr = (self.fb.height - 8) * self.fb.pitch;
            unsafe {
                self.fb.base.add(off).copy_to(self.fb.base, off_clr);
                self.fb.base.add(off_clr).write_bytes(0, off)
            }
            self.y -= 1;
        }
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            match c {
                '\n' => {
                    self.y += 1;
                    self.x = 0;
                    self.handle_scrollback();
                }
                _ => {
                    self.draw_char(c, Colour::new(0xFF, 0xFF, 0xFF, 0xFF));
                    self.x += 1;
                    if self.x >= self.width {
                        self.y += 1;
                        self.x = 0;
                        self.handle_scrollback();
                    }
                }
            }
        }
        Ok(())
    }
}
