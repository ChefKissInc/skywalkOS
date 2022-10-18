// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[repr(C, packed)]
pub struct TaskSegmentSelector {
    __: u32,
    pub rsp: [u64; 4],
    ___: u64,
    pub ist: [u64; 7],
    ____: u64,
    _____: u16,
    pub io_bitmap_offset: u16,
    pub io_bitmap: [u8; 8193],
}

impl TaskSegmentSelector {
    pub const fn new(kern_rsp: u64) -> Self {
        Self {
            __: 0,
            rsp: [kern_rsp; 4],
            ___: 0,
            ist: [kern_rsp; 7],
            ____: 0,
            _____: 0,
            io_bitmap_offset: 112,
            io_bitmap: [0; 8193],
        }
    }
}
