//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::mem::size_of;

use modular_bitfield::prelude::*;

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier)]
#[repr(u32)]
pub struct BIOSHandoff {
    pub bios_owned: bool,
    pub os_owned: bool,
    pub smi_on_ooc: bool,
    pub os_ownership_change: bool,
    pub bios_busy: bool,
    #[skip]
    __: B27,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier)]
#[repr(u32)]
pub struct HBACapabilitiesExt {
    pub bios_handoff: bool,
    pub nvmhci: bool,
    pub auto_partial_to_slumber: bool,
    #[skip]
    __: B29,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier)]
#[repr(u32)]
pub struct EnclosureManagementControl {
    pub msg_received: bool,
    #[skip]
    __: B7,
    pub transmit_msg: bool,
    pub reset: bool,
    #[skip]
    __: B6,
    pub led_msg: bool,
    pub safte_msg: bool,
    pub ses2_msg: bool,
    pub sgpio_msg: bool,
    #[skip]
    __: B4,
    pub single_message_buffer: bool,
    pub transmit_only: bool,
    pub hw_activility_led: bool,
    pub port_multiplier: bool,
    #[skip]
    __: B4,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier)]
#[repr(u32)]
pub struct EnclosureManagementLocation {
    pub buffer_size: u16,
    pub offset_in_dwords: u16,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier)]
#[repr(u32)]
pub struct GlobalHBAControl {
    pub hba_reset: bool,
    pub intr_enable: bool,
    pub msi_revert_to_single_msg: bool,
    #[skip]
    __: B28,
    pub ahci_enable: bool,
}

#[derive(Debug, BitfieldSpecifier)]
#[bits = 4]
pub enum HBAInterfaceSpeed {
    Gen1 = 0b0001,
    Gen2 = 0b0010,
    Gen3 = 0b0011,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier)]
#[repr(u32)]
pub struct HBACapabilities {
    pub port_count: B5,
    pub external_sata: bool,
    pub enclosure_management: bool,
    pub cmd_completion_coalescing: bool,
    pub command_slot_count: B5,
    pub partial_state: bool,
    pub slumber_state: bool,
    pub multiple_drq_block: bool,
    pub fis_based_switching: bool,
    pub port_multiplier: bool,
    pub ahci_only: bool,
    #[skip]
    __: B1,
    pub interface_speed: HBAInterfaceSpeed,
    pub command_list_override: bool,
    pub activity_led: bool,
    pub aggressive_link_pm: bool,
    pub staggered_spinup: bool,
    pub mechanical_presence_switch: bool,
    pub snotification: bool,
    pub native_cmd_queuing: bool,
    pub addr_64bit: bool,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct HBAMemory {
    capability: u32,
    ghc: u32,
    interrupt_status: u32,
    port_implemented: u32,
    version: u32,
    ccc_ctl: u32,
    ccc_ports: u32,
    em_loc: u32,
    em_ctl: u32,
    cap_ext: u32,
    bios_handoff: u32,
    __: [u8; 52],
    nvmhci: [u8; 64],
    _vendor: [u8; 96],
}

impl HBAMemory {
    pub fn capability(&self) -> HBACapabilities {
        self.capability.into()
    }

    pub fn cap_ext(&self) -> HBACapabilitiesExt {
        self.cap_ext.into()
    }

    pub fn global_host_control(&self) -> GlobalHBAControl {
        self.ghc.into()
    }

    pub fn set_global_host_control(&mut self, val: GlobalHBAControl) {
        self.ghc = val.into();
    }

    pub fn is_port_implemented(&self, port: u8) -> bool {
        self.port_implemented & (1u32 << u32::from(port)) != 0
    }

    pub fn get_port_ref(&mut self, port: u8) -> Option<&mut super::HBAPort> {
        if self.is_port_implemented(port) {
            let addr = self as *mut _ as usize + size_of::<Self>();
            Some(unsafe { &mut *((addr as *mut super::HBAPort).add(port.into())) })
        } else {
            None
        }
    }
}

impl core::fmt::Debug for HBAMemory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let port_implemented = self.port_implemented;

        f.debug_struct("HBAMemory")
            .field("capability", &self.capability())
            .field("global_host_control", &self.global_host_control())
            .field("port_implemented", &port_implemented)
            .finish()
    }
}
