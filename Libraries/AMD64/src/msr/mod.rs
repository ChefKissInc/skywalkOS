// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

pub mod apic;
pub mod efer;
pub mod pat;
pub mod vm_cr;

pub trait ModelSpecificReg: Sized + From<u64> {
    const MSR_NUM: u32;

    #[must_use]
    unsafe fn read() -> Self {
        let (low, high): (u32, u32);
        core::arch::asm!("rdmsr", in("ecx") Self::MSR_NUM, out("eax") low, out("edx") high, options(nostack, preserves_flags));
        Self::from((u64::from(high) << 32) | u64::from(low))
    }

    unsafe fn write(self)
    where
        u64: From<Self>,
    {
        let value = u64::from(self);
        let (low, high): (u32, u32) = (value as u32, (value >> 32) as u32);
        core::arch::asm!("wrmsr", in("ecx") Self::MSR_NUM, in("eax") low, in("edx") high, options(nostack, preserves_flags));
    }
}
