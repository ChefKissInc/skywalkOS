// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::cell::SyncUnsafeCell;

use modular_bitfield::prelude::*;

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

#[derive(Default, BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 2]
#[repr(u16)]
pub enum PrivilegeLevel {
    #[default]
    Supervisor = 0,
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

#[derive(Default, BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 5]
#[repr(u8)]
pub enum DescriptorType {
    #[default]
    None = 0b0,
    CodeSegment = 0b11010,
    CodeSegmentAccessed = 0b11011,
    DataSegment = 0b10010,
    DataSegmentAccessed = 0b10011,
    TaskSegment = 0b01001,
}

#[bitfield(bits = 16)]
#[derive(Debug, Clone, Copy)]
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
    pub const fn null() -> Self {
        Self::new(
            0,
            DescriptorType::None,
            PrivilegeLevel::Supervisor,
            true,
            false,
        )
    }

    pub const fn new(
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
            attrs: SegmentAttributes::from_bytes([
                ty as u8 | ((dpl as u8) << 5) | ((present as u8) << 7),
                (long as u8) << 5,
            ]),
            base_high: 0,
        }
    }

    pub const fn new_from_ty(ty: DescriptorType, dpl: PrivilegeLevel) -> Self {
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
    pub const fn null() -> Self {
        Self {
            length: 104,
            base_low: 0,
            base_middle: 0,
            attrs: SegmentAttributes::from_bytes([DescriptorType::TaskSegment as u8, 1 << 5]),
            base_high: 0,
            base_upper: 0,
            __: 0,
        }
    }
}

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
            "lea {2}, [1f + rip]",
            "push {2}",
            "retfq",
            "1:",
            "mov ds, {3}",
            "mov es, {3}",
            "mov ss, {3}",
            in(reg) self,
            in(reg) u64::from(SegmentSelector::new(1, PrivilegeLevel::Supervisor).0),
            lateout(reg) _,
            in(reg) u64::from(SegmentSelector::new(2, PrivilegeLevel::Supervisor).0),
        );
    }
}
