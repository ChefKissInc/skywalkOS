// Copyright (c) ChefKiss Inc 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use arrayvec::ArrayString;

#[bitfield(u32)]
pub struct FeaturesMisc {
    pub brand_id: u8,
    pub clflush: u8,
    pub proc_count: u8,
    pub apic_id: u8,
}

#[bitfield(u64)]
pub struct CPUFeatures {
    // ECX
    pub sse3: bool,
    pub pclmulqdq: bool,
    __: bool,
    pub monitor: bool,
    #[bits(5)]
    __: u8,
    pub ssse3: bool,
    #[bits(2)]
    __: u8,
    pub fma: bool,
    pub cmpxchg16b: bool,
    #[bits(5)]
    __: u8,
    pub sse41: bool,
    pub sse42: bool,
    __: bool,
    pub movbe: bool,
    pub popcnt: bool,
    __: bool,
    pub aes: bool,
    pub xsave: bool,
    pub osxsave: bool,
    pub avx: bool,
    pub f16c: bool,
    pub rdrand: bool,
    pub is_guest: bool,
    pub fpu: bool,
    pub vme: bool,
    pub de: bool,
    pub pse: bool,
    pub tsc: bool,
    pub msr: bool,
    pub pae: bool,
    pub mce: bool,
    pub cmpxchg8b: bool,
    pub apic: bool,
    __: bool,
    pub sysenter_sysexit: bool,
    pub mtrr: bool,
    pub pge: bool,
    pub mca: bool,
    pub cmov: bool,
    pub pat: bool,
    pub pse36: bool,
    __: bool,
    pub clfsh: bool,
    #[bits(3)]
    __: u8,
    pub mmx: bool,
    pub fxsr: bool,
    pub sse: bool,
    pub sse2: bool,
    __: bool,
    pub htt: bool,
    #[bits(3)]
    __: u8,
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
    #[inline]
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
