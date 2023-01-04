pub struct IncrementalNumGenerator {
    last_used: u64,
    last_free: Option<u64>,
}

impl IncrementalNumGenerator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_used: 0,
            last_free: None,
        }
    }

    #[must_use]
    pub fn next(&mut self) -> u64 {
        if let Some(last_free) = self.last_free {
            self.last_free = None;
            self.last_used = last_free;
            last_free
        } else {
            self.last_used += 1;
            self.last_used
        }
    }

    pub fn free(&mut self, num: u64) {
        if num == self.last_used {
            self.last_used -= 1;
        } else {
            self.last_free = self.last_free.or(Some(num));
        }
    }
}
