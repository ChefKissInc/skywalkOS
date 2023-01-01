// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryData {
    pub base: u64,
    pub length: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum MemoryEntry {
    Usable(MemoryData),
    BadMemory(MemoryData),
    ACPIReclaimable(MemoryData),
    BootLoaderReclaimable(MemoryData),
    KernelOrModule(MemoryData),
    FrameBuffer(MemoryData),
}
