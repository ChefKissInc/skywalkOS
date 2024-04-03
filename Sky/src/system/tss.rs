// Copyright (c) ChefKiss Inc 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

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
    #[inline]
    pub const fn new(kern_rsp: u64) -> Self {
        Self {
            __: 0,
            privilege_stack_table: [kern_rsp; 3],
            ___: 0,
            interrupt_stack_table: [kern_rsp; 7],
            ____: 0,
            _____: 0,
            io_bitmap_offset: 112,
            io_bitmap: [0; 8193],
        }
    }
}
