// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![no_std]
#![deny(warnings, clippy::nursery, unused_extern_crates)]
#![allow(clippy::missing_safety_doc)]

use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ext")]
use skykit::msg::Message;

#[macro_use]
extern crate bitfield_struct;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct PCIAddress {
    pub segment: u16,
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
}

impl PCIAddress {
    #[must_use]
    #[inline]
    pub const fn new(segment: u16, bus: u8, slot: u8, func: u8) -> Self {
        Self {
            segment,
            bus,
            slot,
            func,
        }
    }
}

#[bitfield(u16)]
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
    #[bits(5)]
    __: u8,
}

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum PCICfgOffset {
    VendorID = 0x00,
    DeviceID = 0x02,
    Command = 0x04,
    Status = 0x06,
    RevisionId = 0x08,
    ProgIf = 0x09,
    ClassCode = 0x0A,
    Subclass = 0x0B,
    CacheLineSize = 0x0C,
    LatencyTimer = 0x0D,
    HeaderType = 0x0E,
    Bist = 0x0F,
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

#[derive(Debug, Serialize, Deserialize)]
pub enum PCIRequest {
    Read8(PCIAddress, u8),
    Read16(PCIAddress, u8),
    Read32(PCIAddress, u8),
    Write8(PCIAddress, u8, u8),
    Write16(PCIAddress, u8, u16),
    Write32(PCIAddress, u8, u32),
}

#[cfg(feature = "ext")]
impl PCIRequest {
    pub unsafe fn send(self, pid: u64) {
        Message::new(pid, postcard::to_allocvec(&self).unwrap().leak()).send();
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct PCIDevice {
    pid: u64,
    addr: PCIAddress,
}

impl PCIDevice {
    #[must_use]
    #[inline]
    pub const fn new(pid: u64, addr: PCIAddress) -> Self {
        Self { pid, addr }
    }
}

#[cfg(feature = "ext")]
impl PCIDevice {
    #[must_use]
    pub unsafe fn is_multifunction(&self) -> bool {
        (self.cfg_read8::<_, u8>(PCICfgOffset::HeaderType) & 0x80) != 0
    }

    #[must_use]
    pub unsafe fn cfg_read8<A: Into<u8>, R: From<u8>>(&self, off: A) -> R {
        PCIRequest::Read8(self.addr, off.into()).send(self.pid);
        Message::recv().data[0].into()
    }

    #[must_use]
    pub unsafe fn cfg_read16<A: Into<u8>, R: From<u16>>(&self, off: A) -> R {
        PCIRequest::Read16(self.addr, off.into()).send(self.pid);
        u16::from_le_bytes(Message::recv().data.try_into().unwrap()).into()
    }

    #[must_use]
    pub unsafe fn cfg_read32<A: Into<u8>, R: From<u32>>(&self, off: A) -> R {
        PCIRequest::Read32(self.addr, off.into()).send(self.pid);
        u32::from_le_bytes(Message::recv().data.try_into().unwrap()).into()
    }

    pub unsafe fn cfg_write8<A: Into<u8>, R: Into<u8>>(&self, off: A, value: R) {
        PCIRequest::Write8(self.addr, off.into(), value.into()).send(self.pid);
    }

    pub unsafe fn cfg_write16<A: Into<u8>, R: Into<u16>>(&self, off: A, value: R) {
        PCIRequest::Write16(self.addr, off.into(), value.into()).send(self.pid);
    }

    pub unsafe fn cfg_write32<A: Into<u8>, R: Into<u32>>(&self, off: A, value: R) {
        PCIRequest::Write32(self.addr, off.into(), value.into()).send(self.pid);
    }
}
