//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use amd64::cpu::gdt::{DescriptorType, Gdtr, SegmentDescriptor};

pub static ENTRIES: [SegmentDescriptor; 5] = [
    SegmentDescriptor::default(),
    SegmentDescriptor::new_from_ty(DescriptorType::CodeSegment),
    SegmentDescriptor::new_from_ty(DescriptorType::DataSegment),
    SegmentDescriptor::new_from_ty(DescriptorType::TaskSegment),
    SegmentDescriptor::default(),
];

pub static GDTR: Gdtr = Gdtr {
    limit: (core::mem::size_of_val(&ENTRIES) - 1) as u16,
    addr: ENTRIES.as_ptr(),
};
