// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

pub mod apic;
pub mod efer;
pub mod pat;
pub mod vm_cr;

pub trait ModelSpecificReg: Sized {
    const MSR_NUM: u32;

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    #[must_use]
    unsafe fn read() -> Self
    where
        Self: From<u64>,
    {
        let (low, high): (u32, u32);
        core::arch::asm!("rdmsr", in("ecx") Self::MSR_NUM, out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
        Self::from((u64::from(high) << 32) | u64::from(low))
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    unsafe fn write(self)
    where
        u64: From<Self>,
    {
        let value = u64::from(self);
        let (low, high): (u32, u32) = (value as u32, (value >> 32) as u32);
        core::arch::asm!("wrmsr", in("ecx") Self::MSR_NUM, in("eax") low, in("edx") high, options(nomem, nostack, preserves_flags));
    }
}
