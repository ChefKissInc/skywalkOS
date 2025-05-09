// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use crate::acpi::tables::hpet::{regs::GeneralConfiguration, Hpet as HpetInner};

pub struct Hpet {
    inner: &'static HpetInner,
    clk: u64,
}

impl Hpet {
    #[inline]
    pub fn new(hpet: &'static HpetInner) -> Self {
        let clk = u64::from(hpet.capabilities().clk_period());
        hpet.set_config(GeneralConfiguration::new());
        hpet.set_counter_value(0);
        hpet.set_config(GeneralConfiguration::new().with_main_cnt_enable(true));
        Self { inner: hpet, clk }
    }
}

impl super::Timer for Hpet {
    fn sleep(&self, ms: u64) {
        let target = self.inner.counter_value() + (ms * 1_000_000_000_000) / self.clk;

        while self.inner.counter_value() < target {
            unsafe {
                core::arch::x86_64::_mm_pause();
            }
        }
    }
}
