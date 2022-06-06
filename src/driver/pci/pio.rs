//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use amd64::io::port::Port;

#[derive(Clone, Copy)]
pub struct PciPortIo;

impl PciPortIo {
    pub const fn new() -> Self {
        Self
    }

    #[inline]
    unsafe fn send_addr(addr: super::PciAddress, off: u8) {
        assert_eq!(addr.segment, 0, "Using segments on PCI non-express");

        Port::<u32, u32>::new(0xCF8).write(
            ((addr.bus as u32) << 16)
                | ((addr.slot as u32) << 11)
                | ((addr.func as u32) << 8)
                | ((off as u32) & !3u32)
                | 0x80000000,
        );
    }
}

impl super::PCIControllerIO for PciPortIo {
    unsafe fn cfg_read(
        &self,
        addr: super::PciAddress,
        off: u8,
        access_size: super::PCIIOAccessSize,
    ) -> u32 {
        Self::send_addr(addr, off);

        match access_size {
            super::PCIIOAccessSize::Byte => {
                Port::<u8, u8>::new(0xCFC + (off as u16 & 3)).read().into()
            }
            super::PCIIOAccessSize::Word => Port::<u16, u16>::new(0xCFC + (off as u16 & 3))
                .read()
                .into(),
            super::PCIIOAccessSize::DWord => Port::<u32, u32>::new(0xCFC + (off as u16 & 3)).read(),
        }
    }

    unsafe fn cfg_write(
        &self,
        addr: super::PciAddress,
        off: u8,
        value: u32,
        access_size: super::PCIIOAccessSize,
    ) {
        Self::send_addr(addr, off);

        match access_size {
            super::PCIIOAccessSize::Byte => {
                Port::<u8, u8>::new(0xCFC + (off as u16 & 3)).write(value.try_into().unwrap())
            }
            super::PCIIOAccessSize::Word => {
                Port::<u16, u16>::new(0xCFC + (off as u16 & 3)).write(value.try_into().unwrap())
            }
            super::PCIIOAccessSize::DWord => {
                Port::<u32, u32>::new(0xCFC + (off as u16 & 3)).write(value)
            }
        }
    }
}
