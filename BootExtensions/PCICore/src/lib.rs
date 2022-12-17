#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct PCIAddress {
    pub segment: u16,
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
}

impl PCIAddress {
    #[must_use]
    pub const fn new(segment: u16, bus: u8, slot: u8, func: u8) -> Self {
        Self {
            segment,
            bus,
            slot,
            func,
        }
    }
}

#[bitfield(bits = 16)]
#[derive(Debug)]
#[repr(u16)]
pub struct PCICommand {
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

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum PCICfgOffset {
    VendorID = 0x0,
    DeviceID = 0x2,
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

// struct PCIDevice(pci_core::PCIAddress);

// impl PCIDevice {
//     pub unsafe fn is_multifunction(&self) -> bool {
//         self.cfg_read8::<_, u8>(PCICfgOffset::HeaderType) & 0x80 != 0
//     }

//     pub unsafe fn cfg_read8<A: Into<u8>, R: From<u8>>(&self, off: A) -> R {
//         self.controller.cfg_read8(self.addr, off.into()).into()
//     }

//     pub unsafe fn cfg_read16<A: Into<u8>, R: From<u16>>(&self, off: A) -> R {
//         self.controller.cfg_read16(self.addr, off.into()).into()
//     }

//     pub unsafe fn cfg_read32<A: Into<u8>, R: From<u32>>(&self, off: A) -> R {
//         self.controller.cfg_read32(self.addr, off.into()).into()
//     }

//     pub unsafe fn cfg_write8<A: Into<u8>, R: Into<u8>>(&self, off: A, value: R) {
//         self.controller
//             .cfg_write8(self.addr, off.into(), value.into());
//     }

//     pub unsafe fn cfg_write16<A: Into<u8>, R: Into<u16>>(&self, off: A, value: R) {
//         self.controller
//             .cfg_write16(self.addr, off.into(), value.into());
//     }

//     pub unsafe fn cfg_write32<A: Into<u8>, R: Into<u32>>(&self, off: A, value: R) {
//         self.controller
//             .cfg_write32(self.addr, off.into(), value.into());
//     }
// }
