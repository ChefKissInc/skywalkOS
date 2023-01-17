// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;

pub struct IncrementalIDGen {
    last_used: u64,
    freed: Vec<u64>,
}

impl IncrementalIDGen {
    #[inline(always)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_used: 0,
            freed: vec![],
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn next(&mut self) -> u64 {
        if let Some(ret) = self.freed.pop() {
            ret
        } else {
            self.last_used += 1;
            self.last_used
        }
    }

    #[inline(always)]
    pub fn free(&mut self, num: u64) {
        if num == self.last_used {
            self.last_used -= 1;
        } else {
            self.freed.push(num);
        }
    }
}
