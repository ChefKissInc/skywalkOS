/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

pub static ENTRIES: [amd64::sys::gdt::SegmentDescriptor; 5] = [
    amd64::sys::gdt::SegmentDescriptor::default(),
    amd64::sys::gdt::SegmentDescriptor::new_from_ty(amd64::sys::gdt::DescriptorType::CodeSegment),
    amd64::sys::gdt::SegmentDescriptor::new_from_ty(amd64::sys::gdt::DescriptorType::DataSegment),
    amd64::sys::gdt::SegmentDescriptor::new_from_ty(amd64::sys::gdt::DescriptorType::TaskSegment),
    amd64::sys::gdt::SegmentDescriptor::default(),
];

pub static GDTR: amd64::sys::gdt::Gdtr = amd64::sys::gdt::Gdtr {
    limit: (core::mem::size_of_val(&ENTRIES) - 1) as u16,
    addr: ENTRIES.as_ptr(),
};
