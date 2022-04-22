use alloc::boxed::Box;

use modular_bitfield::prelude::*;

mod pio;

#[allow(dead_code)]
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

#[repr(u8)]
pub enum PciConfigOffset {
    VendorId = 0x0,
    DeviceId = 0x2,
    Command = 0x4,
    // Status = 0x6,
    // RevisionId = 0x8,
    // ProgIf = 0x9,
    ClassCode = 0xA,
    // Subclass = 0xB,
    // CacheLineSize = 0xC,
    // LatencyTimer = 0xD,
    // HeaderType = 0xE,
    // Bist = 0xF,
    BaseAddr0 = 0x10,
    BaseAddr1 = 0x14,
    // BaseAddr2 = 0x18,
    // BaseAddr3 = 0x1C,
    // BaseAddr4 = 0x20,
    // BaseAddr5 = 0x24,
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

#[derive(Clone, Copy)]
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

    pub fn find(&self, predicate: fn(PciDevice) -> bool) -> Option<PciDevice> {
        for bus in 0..=255 {
            for slot in 0..32 {
                for func in 0..8 {
                    let device = PciDevice::new(
                        PciAddress {
                            bus,
                            slot,
                            func,
                            ..Default::default()
                        },
                        self.io.as_ref(),
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
