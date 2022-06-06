//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;

mod pio;

#[allow(dead_code)]
pub enum PCIIOAccessSize {
    Byte,
    Word,
    DWord,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PciAddress {
    pub segment: u16,
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
}

#[bitfield(bits = 16)]
#[derive(Debug)]
#[repr(u16)]
pub struct PciCmd {
    pub pio: bool,
    pub mmio: bool,
    pub bus_master: bool,
    pub special_cycle: bool,
    pub mem_write_and_invl: bool,
    pub vga_palette_snoop: bool,
    pub parity_error_resp: bool,
    pub wait_cycle_ctl: bool,
    pub serr: bool,
    pub fast_back_to_back: bool,
    pub disable_intrs: bool,
    #[skip]
    __: B5,
}

#[allow(dead_code)]
#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum PCICfgOffset {
    VendorId = 0x0,
    DeviceId = 0x2,
    Command = 0x4,
    Status = 0x6,
    RevisionId = 0x8,
    ProgIf = 0x9,
    ClassCode = 0xA,
    Subclass = 0xB,
    CacheLineSize = 0xC,
    LatencyTimer = 0xD,
    HeaderType = 0xE,
    Bist = 0xF,
    BaseAddr0 = 0x10,
    BaseAddr1 = 0x14,
    BaseAddr2 = 0x18,
    BaseAddr3 = 0x1C,
    BaseAddr4 = 0x20,
    BaseAddr5 = 0x24,
    CardBusCisPtr = 0x28,
    SubSystemVendorId = 0x2C,
    SubSystemId = 0x2E,
    ExpansionRomBase = 0x30,
    CapabilitiesPtr = 0x34,
    InterruptLine = 0x3C,
    InterruptPin = 0x3D,
    MinimumGrant = 0x3E,
    MaximumLatency = 0x3F,
}

pub trait PCIControllerIO: Sized + Sync + Clone + Copy {
    unsafe fn cfg_read(&self, addr: PciAddress, off: u8, access_size: PCIIOAccessSize) -> u32;
    unsafe fn cfg_write(&self, addr: PciAddress, off: u8, value: u32, access_size: PCIIOAccessSize);
}

#[derive(Clone, Copy)]
pub struct PCIDevice<T: PCIControllerIO>
where
    T: PCIControllerIO,
{
    addr: PciAddress,
    io: T,
}

impl<T: PCIControllerIO> PCIDevice<T> {
    pub fn new(addr: PciAddress, io: T) -> Self {
        Self { addr, io }
    }

    pub unsafe fn cfg_read<A>(&self, off: A, access_size: PCIIOAccessSize) -> u32
    where
        A: Into<u8>,
    {
        self.io.cfg_read(self.addr, off.into(), access_size)
    }

    pub unsafe fn cfg_write<A>(&self, off: A, value: u32, access_size: PCIIOAccessSize)
    where
        A: Into<u8>,
    {
        self.io.cfg_write(self.addr, off.into(), value, access_size)
    }
}

pub struct Pci<T: PCIControllerIO> {
    pub io: T,
}

impl Pci<pio::PciPortIo> {
    pub fn new() -> Self {
        Pci {
            io: pio::PciPortIo::new(),
        }
    }
}

impl<T: PCIControllerIO> Pci<T> {
    pub fn find(&self, predicate: fn(PCIDevice<T>) -> bool) -> Option<PCIDevice<T>> {
        for bus in 0..=255 {
            for slot in 0..32 {
                for func in 0..8 {
                    let device = PCIDevice::new(
                        PciAddress {
                            bus,
                            slot,
                            func,
                            ..Default::default()
                        },
                        self.io,
                    );
                    if predicate(device) {
                        return Some(device);
                    }
                }
            }
        }
        None
    }
}
