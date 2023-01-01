// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use arrayvec::ArrayString;
use modular_bitfield::prelude::*;

#[bitfield(bits = 32)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub struct FeaturesMisc {
    #[skip(setters)]
    pub brand_id: u8,
    #[skip(setters)]
    pub clflush: u8,
    #[skip(setters)]
    pub proc_count: u8,
    #[skip(setters)]
    pub apic_id: u8,
}

#[bitfield(bits = 64)]
#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub struct CPUFeatures {
    // ECX
    #[skip(setters)]
    pub sse3: bool,
    #[skip(setters)]
    pub pclmulqdq: bool,
    #[skip]
    __: bool,
    #[skip(setters)]
    pub monitor: bool,
    #[skip]
    __: B5,
    #[skip(setters)]
    pub ssse3: bool,
    #[skip]
    __: B2,
    #[skip(setters)]
    pub fma: bool,
    #[skip(setters)]
    pub cmpxchg16b: bool,
    #[skip]
    __: B5,
    #[skip(setters)]
    pub sse41: bool,
    #[skip(setters)]
    pub sse42: bool,
    #[skip]
    __: bool,
    #[skip(setters)]
    pub movbe: bool,
    #[skip(setters)]
    pub popcnt: bool,
    #[skip]
    __: bool,
    #[skip(setters)]
    pub aes: bool,
    #[skip(setters)]
    pub xsave: bool,
    #[skip(setters)]
    pub osxsave: bool,
    #[skip(setters)]
    pub avx: bool,
    #[skip(setters)]
    pub f16c: bool,
    #[skip(setters)]
    pub rdrand: bool,
    #[skip(setters)]
    pub is_guest: bool,
    #[skip(setters)]
    pub fpu: bool,
    #[skip(setters)]
    pub vme: bool,
    #[skip(setters)]
    pub de: bool,
    #[skip(setters)]
    pub pse: bool,
    #[skip(setters)]
    pub tsc: bool,
    #[skip(setters)]
    pub msr: bool,
    #[skip(setters)]
    pub pae: bool,
    #[skip(setters)]
    pub mce: bool,
    #[skip(setters)]
    pub cmpxchg8b: bool,
    #[skip(setters)]
    pub apic: bool,
    #[skip]
    __: bool,
    #[skip(setters)]
    pub sysenter_sysexit: bool,
    #[skip(setters)]
    pub mtrr: bool,
    #[skip(setters)]
    pub pge: bool,
    #[skip(setters)]
    pub mca: bool,
    #[skip(setters)]
    pub cmov: bool,
    #[skip(setters)]
    pub pat: bool,
    #[skip(setters)]
    pub pse36: bool,
    #[skip]
    __: bool,
    #[skip(setters)]
    pub clfsh: bool,
    #[skip]
    __: B3,
    #[skip(setters)]
    pub mmx: bool,
    #[skip(setters)]
    pub fxsr: bool,
    #[skip(setters)]
    pub sse: bool,
    #[skip(setters)]
    pub sse2: bool,
    #[skip]
    __: bool,
    #[skip(setters)]
    pub htt: bool,
    #[skip]
    __: B3,
}

#[derive(Debug, Clone, Copy)]
pub struct CPUIdentification {
    pub largest_func_id: u32,
    pub vendor_string: ArrayString<12>,
    pub features: CPUFeatures,
    pub misc: FeaturesMisc,
}

impl Default for CPUIdentification {
    fn default() -> Self {
        Self::new()
    }
}

impl CPUIdentification {
    /// # Panics
    ///
    /// If the CPUID String is not valid UTF-8
    #[must_use]
    pub fn new() -> Self {
        // Function 0
        let res = unsafe { core::arch::x86_64::__cpuid(0) };
        let mut s = [0u8; 12];
        s[..4].copy_from_slice(&res.ebx.to_le_bytes()[..]);
        s[4..8].copy_from_slice(&res.edx.to_le_bytes()[..]);
        s[8..12].copy_from_slice(&res.ecx.to_le_bytes()[..]);
        let largest_func_id = res.eax;
        let vendor_string = ArrayString::from_byte_string(&s).unwrap();

        // Function 1
        let res = unsafe { core::arch::x86_64::__cpuid(1) };
        let features = CPUFeatures::from(u64::from(res.ecx) | (u64::from(res.edx) << 32));
        let misc = FeaturesMisc::from(res.ebx);

        Self {
            largest_func_id,
            vendor_string,
            features,
            misc,
        }
    }
}
