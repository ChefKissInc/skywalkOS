// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct TaskSegmentSelector {
    __: u32,
    pub privilege_stack_table: [u64; 3],
    ___: u64,
    pub interrupt_stack_table: [u64; 7],
    ____: u64,
    _____: u16,
    pub io_bitmap_offset: u16,
    pub io_bitmap: [u8; 8193],
}

impl TaskSegmentSelector {
    #[inline(always)]
    #[must_use]
    pub const fn new(kern_rsp: u64) -> Self {
        Self {
            __: 0,
            privilege_stack_table: [kern_rsp, 0, 0],
            ___: 0,
            interrupt_stack_table: [kern_rsp, 0, 0, 0, 0, 0, 0],
            ____: 0,
            _____: 0,
            io_bitmap_offset: 112,
            io_bitmap: [0; 8193],
        }
    }
}
