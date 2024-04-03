// Copyright (c) ChefKiss Inc 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::vec::Vec;

pub struct IncrementalIDGen {
    last_used: u64,
    freed: Vec<u64>,
}

impl Default for IncrementalIDGen {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalIDGen {
    #[inline]
    pub const fn new() -> Self {
        Self {
            last_used: 0,
            freed: vec![],
        }
    }

    pub fn next(&mut self) -> u64 {
        let Some(ret) = self.freed.pop() else {
            self.last_used += 1;
            return self.last_used;
        };
        ret
    }

    pub fn free(&mut self, num: u64) {
        if num == self.last_used {
            self.last_used -= 1;
        } else {
            self.freed.push(num);
        }
    }
}
