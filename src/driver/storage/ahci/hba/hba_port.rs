//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use modular_bitfield::prelude::*;

#[derive(Debug, BitfieldSpecifier)]
#[bits = 4]
pub enum StatusDeviceDetection {
    NoDevice = 0x0,
    NoPhyCommunicationEstablished = 0x1,
    DeviceExists = 0x3,
    PhyOffline = 0x4,
}

#[derive(Debug, BitfieldSpecifier)]
#[bits = 4]
pub enum CurrentInterfaceSpeed {
    NoDeviceOrNoCommunicationEstablished = 0x0,
    Gen1 = 0x1,
    Gen2 = 0x2,
    Gen3 = 0x3,
}

#[derive(Debug, BitfieldSpecifier)]
#[bits = 4]
pub enum InterfacePowerManagement {
    NoDeviceOrNoCommunicationEstablished = 0x0,
    ActiveState = 0x1,
    PartialState = 0x2,
    SlumberState = 0x6,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier, Clone, Copy)]
#[repr(u32)]
pub struct HBAPortStatus {
    #[skip(setters)]
    pub device_detection: StatusDeviceDetection,
    #[skip(setters)]
    pub interface_speed: CurrentInterfaceSpeed,
    #[skip(setters)]
    pub interface_pm: InterfacePowerManagement,
    #[skip]
    __: B20,
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum HBAPortSignature {
    Null = 0x0,
    SerialATA = 0x0101,
    SerialATAPI = 0xEB140101,
    EnclosureManagementBridge = 0xC33C0101,
    PortMultiplier = 0x96690101,
    Default = 0xFFFFFFFF,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct HBAPort {
    clb: u32,
    clbu: u32,
    fb: u32,
    fbu: u32,
    intr_status: u32,
    intr_enable: u32,
    cmd_and_status: u32,
    __: u32,
    task_file_data: u32,
    signature: HBAPortSignature,
    sata_status: u32,
    sata_control: u32,
    sata_error: u32,
    sata_active: u32,
    command_issue: u32,
    sata_notif: u32,
    fis_switch_ctl: u32,
    device_sleep: u32,
    ___: [u8; 40],
    _vendor: [u8; 16],
}

impl HBAPort {
    pub fn cmd_list_base(&self) -> u64 {
        (self.clb as u64) | ((self.clbu as u64) << 32)
    }

    pub fn fis_base(&self) -> u64 {
        (self.fb as u64) | ((self.fbu as u64) << 32)
    }

    pub fn signature(&self) -> HBAPortSignature {
        self.signature
    }

    pub fn sata_status(&self) -> HBAPortStatus {
        self.sata_status.into()
    }
}

impl core::fmt::Debug for HBAPort {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HBAPort")
            .field("cmd_list_base", &self.cmd_list_base())
            .field("fis_base", &self.fis_base())
            .field("signature", &self.signature())
            .field("sata_status", &self.sata_status())
            .finish()
    }
}
