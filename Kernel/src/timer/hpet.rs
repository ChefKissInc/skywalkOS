// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use acpi::tables::hpet::{regs::GeneralConfiguration, HPET};

pub struct HighPrecisionEventTimer {
    inner: &'static HPET,
    clk: u64,
}

impl HighPrecisionEventTimer {
    #[inline]
    #[must_use]
    pub fn new(hpet: &'static HPET) -> Self {
        let clk = u64::from(hpet.capabilities().clk_period());
        hpet.set_config(GeneralConfiguration::new());
        hpet.set_counter_value(0);
        hpet.set_config(GeneralConfiguration::new().with_main_cnt_enable(true));
        Self { inner: hpet, clk }
    }
}

impl super::Timer for HighPrecisionEventTimer {
    fn sleep(&self, ms: u64) {
        let target = self.inner.counter_value() + (ms * 1_000_000_000_000) / self.clk;

        while self.inner.counter_value() < target {
            unsafe {
                core::arch::asm!("pause", options(nostack, preserves_flags));
            }
        }
    }
}
