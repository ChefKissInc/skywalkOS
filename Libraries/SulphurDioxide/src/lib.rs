// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

pub const CURRENT_REVISION: u64 = 0x18;

pub type EntryPoint = extern "sysv64" fn(&'static BootInfo) -> !;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct KernSymbol {
    pub start: u64,
    pub end: u64,
    pub name: &'static str,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct BootSettings {
    pub verbose: bool,
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
    KernelOrModule(MemoryData),
    FrameBuffer(MemoryData),
}

#[repr(C)]
#[derive(Debug)]
pub enum PixelFormat {
    RedGreenBlue,
    BlueGreenRed,
    Bitmask,
}

#[repr(C)]
#[derive(Debug)]
pub struct PixelBitmask {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub alpha: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct ScreenRes {
    pub width: usize,
    pub height: usize,
}

impl ScreenRes {
    #[inline(always)]
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
    pub pixel_format: PixelFormat,
    pub pixel_bitmask: PixelBitmask,
    pub pitch: usize,
    pub base: *mut u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct BootInfo {
    pub revision: u64,
    pub kern_symbols: &'static [KernSymbol],
    pub settings: BootSettings,
    pub memory_map: &'static [MemoryEntry],
    pub frame_buffer: Option<&'static FrameBufferInfo>,
    pub acpi_rsdp: &'static acpi::tables::rsdp::RSDP,
    pub dc_cache: &'static [u8],
}

impl BootInfo {
    #[inline(always)]
    #[must_use]
    pub fn new(
        kern_symbols: &'static [KernSymbol],
        settings: BootSettings,
        frame_buffer: Option<&'static FrameBufferInfo>,
        acpi_rsdp: &'static acpi::tables::rsdp::RSDP,
        dc_cache: &'static [u8],
    ) -> Self {
        Self {
            revision: CURRENT_REVISION,
            kern_symbols,
            settings,
            memory_map: Default::default(),
            frame_buffer,
            acpi_rsdp,
            dc_cache,
        }
    }
}
