// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::fmt::Write;

use amd64::paging::{PageTableFlags, PAGE_SIZE};
use skybuffer::{fb::FrameBuffer, pixel::Colour};

mod font;

pub struct Terminal {
    pub x: usize,
    pub y: usize,
    pub fb: FrameBuffer,
    pub width: usize,
    pub height: usize,
}

unsafe impl Sync for Terminal {}

impl Terminal {
    #[inline]
    pub const fn new(fb: FrameBuffer) -> Self {
        let width = fb.width / font::FONT_WIDTH;
        let height = fb.height / font::FONT_HEIGHT;
        Self {
            x: 0,
            y: 0,
            fb,
            width,
            height,
        }
    }

    #[inline]
    pub fn map_fb(&self) {
        unsafe {
            let state = &mut *super::state::SYS_STATE.get();
            let base = self.fb.base as u64;
            state.pml4.as_ref().unwrap().lock().map(
                base,
                base - amd64::paging::PHYS_VIRT_OFFSET,
                (self.fb.height * self.fb.stride).div_ceil(PAGE_SIZE as usize) as _,
                PageTableFlags::new_present()
                    .with_writable(true)
                    .with_pat_entry(2),
            );
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.fb
            .clear(Colour::new(0, 0, 0, 0).as_u32(self.fb.format));
        self.x = 0;
        self.y = 0;
    }

    #[inline]
    pub fn draw_char(&mut self, c: u8, colour: Colour) {
        let Some(v) = c
            .checked_sub(0x20)
            .and_then(|v| font::FONT_BITMAP.get(v as usize))
        else {
            return;
        };
        let (x, y) = (self.x * font::FONT_WIDTH, self.y * font::FONT_HEIGHT);
        let colour = colour.as_u32(self.fb.format);
        for (i, x_bit) in v
            .chunks_exact(2)
            .map(|a| u16::from_ne_bytes([a[0], a[1]]))
            .enumerate()
        {
            for bit in (0..font::FONT_WIDTH).filter(|bit| x_bit & (1 << bit) != 0) {
                self.fb
                    .plot_pixel(x + font::FONT_WIDTH - bit, y + i, colour)
                    .unwrap();
            }
        }
    }

    #[inline]
    const fn handle_scrollback(&mut self) {
        if self.y >= self.height {
            let row_len = self.fb.stride * font::FONT_HEIGHT;
            let len = (self.fb.stride * self.fb.height) - row_len;
            unsafe {
                self.fb.base.add(row_len).copy_to(self.fb.base, len);
                self.fb.base.add(len).write_bytes(0, row_len);
            }
            self.y -= 1;
            self.x = 0;
        }
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            if c == b'\n' {
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
