// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

use paper_fb::pixel::PixelBitMask;

pub const CURRENT_REVISION: u64 = 0x1B;

pub type EntryPoint = extern "sysv64" fn(&'static BootInfo) -> !;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct KernSymbol {
    pub start: u64,
    pub end: u64,
    pub name: &'static str,
}

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
    FrameBuffer(MemoryData),
}

#[repr(C)]
#[derive(Debug)]
pub struct ScreenRes {
    pub width: usize,
    pub height: usize,
}

impl ScreenRes {
    #[inline]
    #[must_use]
    pub const fn new(res: (usize, usize)) -> Self {
        Self {
            width: res.0,
            height: res.1,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FrameBufferInfo {
    pub resolution: ScreenRes,
    pub pixel_bitmask: PixelBitMask,
    pub pitch: usize,
    pub base: *mut u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct BootInfo {
    pub revision: u64,
    pub kern_symbols: &'static [KernSymbol],
    pub verbose: bool,
    pub serial_enabled: bool,
    pub memory_map: &'static [MemoryEntry],
    pub frame_buffer: Option<&'static FrameBufferInfo>,
    pub acpi_rsdp: *const u8,
    pub fkcache: &'static [u8],
}

impl BootInfo {
    #[inline]
    #[must_use]
    pub fn new(
        kern_symbols: &'static [KernSymbol],
        verbose: bool,
        serial_enabled: bool,
        frame_buffer: Option<&'static FrameBufferInfo>,
        acpi_rsdp: *const u8,
        fkcache: &'static [u8],
    ) -> Self {
        Self {
            revision: CURRENT_REVISION,
            kern_symbols,
            verbose,
            serial_enabled,
            memory_map: Default::default(),
            frame_buffer,
            acpi_rsdp,
            fkcache,
        }
    }
}
