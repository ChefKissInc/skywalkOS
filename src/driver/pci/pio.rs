// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use amd64::io::port::Port;

#[derive(Clone)]
pub struct PCIPortIO;

impl PCIPortIO {
    pub const fn new() -> Self {
        Self
    }

    #[inline]
    unsafe fn send_addr(addr: super::PCIAddress, off: u8) {
        assert_eq!(addr.segment, 0, "Using segments on PCI non-express");

        Port::<u32, u32>::new(0xCF8).write(
            (u32::from(addr.bus) << 16)
                | (u32::from(addr.slot) << 11)
                | (u32::from(addr.func) << 8)
                | (u32::from(off) & !3u32)
                | 0x8000_0000,
        );
    }
}

impl super::PCIControllerIO for PCIPortIO {
    unsafe fn cfg_read(
        &self,
        addr: super::PCIAddress,
        off: u8,
        access_size: super::PCIIOAccessSize,
    ) -> u32 {
        Self::send_addr(addr, off);

        match access_size {
            super::PCIIOAccessSize::Byte => Port::<u8, u8>::new(0xCFC + (u16::from(off) & 3))
                .read()
                .into(),
            super::PCIIOAccessSize::Word => Port::<u16, u16>::new(0xCFC + (u16::from(off) & 3))
                .read()
                .into(),
            super::PCIIOAccessSize::DWord => {
                Port::<u32, u32>::new(0xCFC + (u16::from(off) & 3)).read()
            }
        }
    }

    unsafe fn cfg_write(
        &self,
        addr: super::PCIAddress,
        off: u8,
        value: u32,
        access_size: super::PCIIOAccessSize,
    ) {
        Self::send_addr(addr, off);

        match access_size {
            super::PCIIOAccessSize::Byte => {
                Port::<u8, u8>::new(0xCFC + (u16::from(off) & 3)).write(value.try_into().unwrap());
            }
            super::PCIIOAccessSize::Word => {
                Port::<u16, u16>::new(0xCFC + (u16::from(off) & 3))
                    .write(value.try_into().unwrap());
            }
            super::PCIIOAccessSize::DWord => {
                Port::<u32, u32>::new(0xCFC + (u16::from(off) & 3)).write(value);
            }
        }
    }
}
