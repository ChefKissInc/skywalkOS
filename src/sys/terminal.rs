//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use core::fmt::Write;

use amd64::paging::pml4::Pml4;
use vesa::framebuffer::Framebuffer;

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
            (&mut *super::state::SYS_STATE.pml4.get())
                .get_mut()
                .unwrap()
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

    pub fn draw_char(&mut self, c: char) {
        let x = self.x * 8;
        let mut y = self.y * 8;
        for &x_bit in &font8x8::legacy::BASIC_LEGACY[c as usize] {
            for bit in 0..8 {
                if x_bit & (1 << bit) != 0 {
                    self.fb.draw_pixel(x + bit, y, !0u32).unwrap();
                }
            }
            y += 1;
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
                    self.draw_char(c);
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
