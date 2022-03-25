//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use amd64::io::port::Port;

pub struct PciPortIo;

impl PciPortIo {
    #[inline]
    fn send_addr(bus: u8, slot: u8, func: u8, off: u8) {
        unsafe {
            Port::<u32>::new(0xCF8).write(
                ((bus as u32) << 16)
                    | ((slot as u32) << 11)
                    | ((func as u32) << 8)
                    | ((off as u32) & !3u32)
                    | 0x80000000,
            );
        }
    }
}

impl super::PciIo for PciPortIo {
    fn cfg_read(
        &self,
        addr: super::PciAddress,
        off: u8,
        access_size: super::PciIoAccessSize,
    ) -> u32 {
        assert_eq!(addr.segment, 0, "Using segments on PCI non-express");
        Self::send_addr(addr.bus, addr.slot, addr.func, off);
        unsafe {
            match access_size {
                super::PciIoAccessSize::Byte => {
                    Port::<u8>::new(0xCFC + (off as u16 & 3)).read().into()
                }
                super::PciIoAccessSize::Word => {
                    Port::<u16>::new(0xCFC + (off as u16 & 3)).read().into()
                }
                super::PciIoAccessSize::DWord => Port::<u32>::new(0xCFC + (off as u16 & 3)).read(),
            }
        }
    }

    fn cfg_write(
        &self,
        addr: super::PciAddress,
        off: u8,
        value: u32,
        access_size: super::PciIoAccessSize,
    ) {
        assert_eq!(addr.segment, 0, "Using segments on PCI non-express");
        Self::send_addr(addr.bus, addr.slot, addr.func, off);
        unsafe {
            match access_size {
                super::PciIoAccessSize::Byte => {
                    Port::<u8>::new(0xCFC + (off as u16 & 3)).write(value.try_into().unwrap())
                }
                super::PciIoAccessSize::Word => {
                    Port::<u16>::new(0xCFC + (off as u16 & 3)).write(value.try_into().unwrap())
                }
                super::PciIoAccessSize::DWord => {
                    Port::<u32>::new(0xCFC + (off as u16 & 3)).write(value)
                }
            }
        }
    }
}
