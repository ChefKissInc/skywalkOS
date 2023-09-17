// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use num_enum::IntoPrimitive;

#[bitfield(u16)]
pub struct MasterOutputVolume {
    #[bits(6)]
    pub right: usize,
    #[bits(2)]
    __: u8,
    #[bits(6)]
    pub left: usize,
    #[bits(1)]
    __: u8,
    pub mute: bool,
}

#[bitfield(u16)]
pub struct PcmOutputVolume {
    #[bits(5)]
    pub right: usize,
    #[bits(3)]
    __: u8,
    #[bits(5)]
    pub left: usize,
    #[bits(2)]
    __: u8,
    pub mute: bool,
}

#[bitfield(u8)]
pub struct RegBoxTransfer {
    pub transfer_data: bool,
    pub reset: bool,
    pub last_ent_fire_intr: bool,
    pub ioc_intr: bool,
    pub fifo_err_intr: bool,
    #[bits(3)]
    __: u8,
}

#[bitfield(u16)]
pub struct RegBoxStatus {
    pub transfer_data: bool,
    pub end_of_transfer: bool,
    pub last_ent_fire_intr: bool,
    pub ioc_intr: bool,
    pub fifo_err_intr: bool,
    #[bits(11)]
    __: u16,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(u32)]
pub enum PcmChannels {
    #[default]
    Two = 0,
    Four,
    Six,
}

impl PcmChannels {
    const fn into_bits(self) -> u32 {
        self as _
    }

    const fn from_bits(value: u32) -> Self {
        match value {
            0 => Self::Two,
            1 => Self::Four,
            2 => Self::Six,
            _ => panic!("Invalid value for PcmChannels"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(u32)]
pub enum PcmOutMode {
    #[default]
    SixteenSamples = 0,
    TwentySamples,
}

impl PcmOutMode {
    const fn into_bits(self) -> u32 {
        self as _
    }

    const fn from_bits(value: u32) -> Self {
        match value {
            0 => Self::SixteenSamples,
            1 => Self::TwentySamples,
            _ => panic!("Invalid value for PcmOutMode"),
        }
    }
}

#[bitfield(u32)]
pub struct GlobalControl {
    pub interrupts: bool,
    pub cold_reset: bool,
    pub warm_reset: bool,
    pub shut_down: bool,
    __: u16,
    #[bits(2)]
    pub channels: PcmChannels,
    #[bits(2)]
    pub pcm_out_mode: PcmOutMode,
    __: u8,
}

#[bitfield(u32)]
pub struct GlobalStatus {
    #[bits(20)]
    __: u32,
    #[bits(2)]
    pub channel_caps: PcmChannels,
    #[bits(2)]
    pub sample_caps: PcmOutMode,
    __: u8,
}

#[bitfield(u16)]
pub struct BufferDescCtl {
    #[bits(14)]
    __: u16,
    pub last: bool,
    pub fire_interrupt: bool,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct BufferDescriptor {
    pub addr: u32,
    pub samples: u16,
    pub ctl: BufferDescCtl,
}

#[repr(u16)]
#[derive(IntoPrimitive)]
pub enum MixerReg {
    Reset = 0x0,
    MasterVolume = 0x2,
    PCMOutVolume = 0x18,
    SampleRate = 0x2C,
}

#[repr(u16)]
#[derive(IntoPrimitive)]
pub enum AudioBusReg {
    PCMOutBDLAddr = 0x10,
    PCMOutLastEnt = 0x15,
    PCMOutStatus = 0x16,
    PCMOutTransferControl = 0x1B,
    GlobalControl = 0x2C,
}
