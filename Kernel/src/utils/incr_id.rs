// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::vec::Vec;

pub struct IncrementalIDGen {
    last_used: u64,
    freed: Vec<u64>,
}

impl IncrementalIDGen {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_used: 0,
            freed: vec![],
        }
    }

    #[must_use]
    pub fn next(&mut self) -> u64 {
        if let Some(ret) = self.freed.pop() {
            ret
        } else {
            self.last_used += 1;
            self.last_used
        }
    }

    pub fn free(&mut self, num: u64) {
        if num == self.last_used {
            self.last_used -= 1;
        } else {
            self.freed.push(num);
        }
    }
}
