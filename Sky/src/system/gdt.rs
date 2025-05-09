// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::cell::SyncUnsafeCell;

#[derive(Default, Debug, Clone, Copy)]
#[repr(u16)]
/// 2 bits
pub enum PrivilegeLevel {
    #[default]
    Supervisor = 0,
    User = 3,
}

impl PrivilegeLevel {
    pub const fn into_bits(self) -> u16 {
        self as _
    }

    pub const fn from_bits(value: u16) -> Self {
        match value {
            0 => Self::Supervisor,
            3 => Self::User,
            _ => panic!("Unknown PrivilegeLevel"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    #[inline]
    pub const fn new(index: u16, dpl: PrivilegeLevel) -> Self {
        Self((index << 3) | (dpl as u16))
    }
}

impl From<SegmentSelector> for u64 {
    #[inline]
    fn from(value: SegmentSelector) -> Self {
        value.0 as _
    }
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
/// 5 bits
pub enum DescriptorType {
    #[default]
    None = 0b0,
    CodeSegment = 0b11010,
    CodeSegmentAccessed = 0b11011,
    DataSegment = 0b10010,
    DataSegmentAccessed = 0b10011,
    TaskSegment = 0b01001,
}

impl DescriptorType {
    const fn into_bits(self) -> u16 {
        self as _
    }

    const fn from_bits(value: u16) -> Self {
        match value {
            0b00000 => Self::None,
            0b11010 => Self::CodeSegment,
            0b11011 => Self::CodeSegmentAccessed,
            0b10010 => Self::DataSegment,
            0b10011 => Self::DataSegmentAccessed,
            0b01001 => Self::TaskSegment,
            _ => panic!("Unknown DescriptorType"),
        }
    }
}

#[bitfield(u16)]
pub struct SegmentAttributes {
    #[bits(5)]
    pub ty: DescriptorType,
    #[bits(2)]
    pub dpl: PrivilegeLevel,
    pub present: bool,
    #[bits(4)]
    pub limit_high: u8,
    pub avl: bool,
    pub long: bool,
    pub default_op_size: bool,
    pub granularity: bool,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct SegmentDescriptor {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_middle: u8,
    pub attrs: SegmentAttributes,
    pub base_high: u8,
}

impl SegmentDescriptor {
    #[inline]
    const fn null() -> Self {
        Self::new(
            0,
            DescriptorType::None,
            PrivilegeLevel::Supervisor,
            true,
            false,
        )
    }

    #[inline]
    const fn new(
        limit_low: u16,
        ty: DescriptorType,
        dpl: PrivilegeLevel,
        present: bool,
        long: bool,
    ) -> Self {
        Self {
            limit_low,
            base_low: 0,
            base_middle: 0,
            attrs: SegmentAttributes::new()
                .with_ty(ty)
                .with_dpl(dpl)
                .with_present(present)
                .with_long(long),
            base_high: 0,
        }
    }

    #[inline]
    const fn new_from_ty(ty: DescriptorType, dpl: PrivilegeLevel) -> Self {
        match ty {
            DescriptorType::CodeSegment => Self::new(0, ty, dpl, true, true),
            _ => Self::new(0, ty, dpl, true, false),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TaskSegmentDescriptor {
    pub length: u16,
    pub base_low: u16,
    pub base_middle: u8,
    pub attrs: SegmentAttributes,
    pub base_high: u8,
    pub base_upper: u32,
    __: u32,
}

impl TaskSegmentDescriptor {
    #[inline]
    pub const fn null() -> Self {
        Self {
            length: 104,
            base_low: 0,
            base_middle: 0,
            attrs: SegmentAttributes::new()
                .with_ty(DescriptorType::TaskSegment)
                .with_long(true),
            base_high: 0,
            base_upper: 0,
            __: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GDTData {
    _null: SegmentDescriptor,
    _code_segment: SegmentDescriptor,
    _data_segment: SegmentDescriptor,
    _user_code_segment: SegmentDescriptor,
    _user_data_segment: SegmentDescriptor,
    pub task_segment: TaskSegmentDescriptor,
}

impl GDTData {
    #[inline]
    pub const fn new() -> Self {
        Self {
            _null: SegmentDescriptor::null(),
            _code_segment: SegmentDescriptor::new_from_ty(
                DescriptorType::CodeSegment,
                PrivilegeLevel::Supervisor,
            ),
            _data_segment: SegmentDescriptor::new_from_ty(
                DescriptorType::DataSegment,
                PrivilegeLevel::Supervisor,
            ),
            _user_code_segment: SegmentDescriptor::new_from_ty(
                DescriptorType::CodeSegment,
                PrivilegeLevel::User,
            ),
            _user_data_segment: SegmentDescriptor::new_from_ty(
                DescriptorType::DataSegment,
                PrivilegeLevel::User,
            ),
            task_segment: TaskSegmentDescriptor::null(),
        }
    }
}

pub static GDT: SyncUnsafeCell<GDTData> = SyncUnsafeCell::new(GDTData::new());

pub static GDTR: GDTReg = GDTReg {
    limit: (core::mem::size_of_val(&GDT) - 1) as u16,
    addr: GDT.get(),
};

#[repr(C, packed)]
pub struct GDTReg {
    pub limit: u16,
    pub addr: *const GDTData,
}

unsafe impl Sync for GDTReg {}

impl GDTReg {
    pub unsafe fn load(&self) {
        debug!("Initialising.");
        core::arch::asm!(
            "lgdt [{}]",
            "push {}",
            "lea {2}, [2f + rip]",
            "push {2}",
            "retfq",
            "2:",
            "mov ds, {3}",
            "mov es, {3}",
            "mov ss, {3}",
            in(reg) self,
            in(reg) u64::from(SegmentSelector::new(1, PrivilegeLevel::Supervisor)),
            lateout(reg) _,
            in(reg) u64::from(SegmentSelector::new(2, PrivilegeLevel::Supervisor)),
            options(preserves_flags)
        );
    }
}
