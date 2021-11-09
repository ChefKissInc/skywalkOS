pub static ENTRIES: [amd64::registers::gdt::SegmentDescriptor; 5] = [
    amd64::registers::gdt::SegmentDescriptor::default(),
    amd64::registers::gdt::SegmentDescriptor::new_from_ty(
        amd64::registers::gdt::DescriptorType::CodeSegment,
    ),
    amd64::registers::gdt::SegmentDescriptor::new_from_ty(
        amd64::registers::gdt::DescriptorType::DataSegment,
    ),
    amd64::registers::gdt::SegmentDescriptor::new_from_ty(
        amd64::registers::gdt::DescriptorType::TaskSegment,
    ),
    amd64::registers::gdt::SegmentDescriptor::default(),
];

pub static GDTR: amd64::registers::gdt::Gdtr = amd64::registers::gdt::Gdtr {
    limit: (core::mem::size_of_val(&ENTRIES) - 1) as u16,
    addr: ENTRIES.as_ptr(),
};
