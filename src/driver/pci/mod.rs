#![allow(dead_code)]

use alloc::boxed::Box;

mod pio;

pub enum PciIoAccessSize {
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

#[repr(u8)]
pub enum PciConfigOffset {
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
    // Bist = 0xF,
    BaseAddr0 = 0x10,
    BaseAddr1 = 0x14,
    BaseAddr2 = 0x18,
    BaseAddr3 = 0x1C,
    BaseAddr4 = 0x20,
    BaseAddr5 = 0x24,
    // CardBusCisPtr = 0x28,
    // SubSystemVendorId = 0x2C,
    // SubSystemId = 0x2E,
    // ExpansionRomBase = 0x30,
    // CapabilitiesPtr = 0x34,
    // InterruptLine = 0x3C,
    // InterruptPin = 0x3D,
    // MinimumGrant = 0x3E,
    // MaximumLatency = 0x3F
}

pub trait PciIo {
    fn cfg_read(&self, addr: PciAddress, off: u8, access_size: PciIoAccessSize) -> u32;
    fn cfg_write(&self, addr: PciAddress, off: u8, value: u32, access_size: PciIoAccessSize);
}

#[repr(u8)]
pub enum PciHeaderType {
    General = 0x0,
    PciToPciBridge,
    CardBusBridge,
}

pub struct PciDevice<'a> {
    addr: PciAddress,
    io: &'a dyn PciIo,
}

impl<'a> PciDevice<'a> {
    pub fn new(addr: PciAddress, io: &'a dyn PciIo) -> Self {
        Self { addr, io }
    }

    pub fn cfg_read(&self, off: u8, access_size: PciIoAccessSize) -> u32 {
        self.io.cfg_read(self.addr, off, access_size)
    }

    pub fn cfg_write(&self, off: u8, value: u32, access_size: PciIoAccessSize) {
        self.io.cfg_write(self.addr, off, value, access_size)
    }
}

pub struct Pci {
    pub io: Box<dyn PciIo>,
}

impl Pci {
    pub fn new() -> Pci {
        Pci {
            io: Box::new(pio::PciPortIo),
        }
    }
}
