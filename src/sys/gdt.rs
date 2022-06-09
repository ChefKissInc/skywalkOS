//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use core::arch::asm;

use modular_bitfield::prelude::*;

pub static ENTRIES: [SegmentDescriptor; 5] = [
    SegmentDescriptor::default(),
    SegmentDescriptor::new_from_ty(DescriptorType::CodeSegment),
    SegmentDescriptor::new_from_ty(DescriptorType::DataSegment),
    SegmentDescriptor::new_from_ty(DescriptorType::TaskSegment),
    SegmentDescriptor::default(),
];

pub static GDTR: GDTReg = GDTReg {
    limit: (core::mem::size_of_val(&ENTRIES) - 1) as u16,
    addr: ENTRIES.as_ptr(),
};

#[derive(Default, BitfieldSpecifier)]
#[bits = 2]
#[repr(u16)]
pub enum PrivilegeLevel {
    #[default]
    Hypervisor = 0,
    User = 3,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    pub const fn new(index: u16, dpl: PrivilegeLevel) -> Self {
        Self((index << 3) | (dpl as u16))
    }
}

#[derive(Default, BitfieldSpecifier)]
#[bits = 5]
#[repr(u8)]
pub enum DescriptorType {
    #[default]
    None = 0b0,
    CodeSegment = 0b11010,
    DataSegment = 0b10010,
    TaskSegment = 0b01001,
}

#[bitfield(bits = 16)]
#[repr(u16)]
pub struct SegmentAttributes {
    pub ty: DescriptorType,
    pub dpl: PrivilegeLevel,
    pub present: bool,
    pub limit_high: B4,
    pub avl: B1,
    pub long: bool,
    pub default_op_size: bool,
    pub granularity: bool,
}

#[repr(C, packed)]
pub struct SegmentDescriptor {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_middle: u8,
    pub attrs: SegmentAttributes,
    pub base_high: u8,
}

impl SegmentDescriptor {
    pub const fn default() -> Self {
        Self::new(0, DescriptorType::None, true, false)
    }

    pub const fn new(limit_low: u16, ty: DescriptorType, present: bool, long: bool) -> Self {
        Self {
            limit_low,
            base_low: 0,
            base_middle: 0,
            attrs: SegmentAttributes::from_bytes([
                ty as u8 | ((present as u8) << 7),
                (long as u8) << 5,
            ]),
            base_high: 0,
        }
    }

    pub const fn new_from_ty(ty: DescriptorType) -> Self {
        match ty {
            DescriptorType::CodeSegment => Self::new(0, ty, true, true),
            DescriptorType::TaskSegment => Self::new(104, ty, false, false),
            _ => Self::new(0, ty, true, false),
        }
    }
}

#[repr(C, packed)]
pub struct GDTReg {
    pub limit: u16,
    pub addr: *const SegmentDescriptor,
}

unsafe impl Sync for GDTReg {}

impl GDTReg {
    pub unsafe fn load(&self, cs: SegmentSelector, ds: SegmentSelector) {
        asm!(
            "lgdt [{}]",
            "push {}",
            "lea {2}, [1f + rip]",
            "push {2}",
            "retfq",
            "1:",
            "mov ds, {3}",
            "mov es, {3}",
            "mov fs, {3}",
            "mov gs, {3}",
            "mov ss, {3}",
            in(reg) self,
            in(reg) cs.0 as u64,
            lateout(reg) _,
            in(reg) ds.0 as u64,
        );
    }
}
