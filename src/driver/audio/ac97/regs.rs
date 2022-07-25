//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;

#[bitfield(bits = 16)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u16)]
pub struct MasterOutputVolume {
    pub right: B6,
    #[skip]
    __: B2,
    pub left: B6,
    #[skip]
    __: B1,
    pub mute: bool,
}

#[bitfield(bits = 16)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u16)]
pub struct PcmOutputVolume {
    pub right: B5,
    #[skip]
    __: B3,
    pub left: B5,
    #[skip]
    __: B2,
    pub mute: bool,
}

#[bitfield(bits = 8)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub struct RegBoxTransfer {
    pub transfer_data: bool,
    pub reset: bool,
    pub last_ent_fire_intr: bool,
    pub ioc_intr: bool,
    pub fifo_err_intr: bool,
    #[skip]
    __: B3,
}

#[bitfield(bits = 16)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u16)]
pub struct RegBoxStatus {
    pub transfer_data: bool,
    pub end_of_transfer: bool,
    pub last_ent_fire_intr: bool,
    pub ioc_intr: bool,
    pub fifo_err_intr: bool,
    #[skip]
    __: B11,
}

#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy)]
#[bits = 2]
pub enum PcmChannels {
    #[default]
    Two = 0,
    Four,
    Six,
}

#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy)]
#[bits = 2]
pub enum PcmOutMode {
    #[default]
    SixteenSamples = 0,
    TwentySamples,
}

#[bitfield(bits = 32)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u32)]
pub struct GlobalControl {
    pub interrupts: bool,
    pub cold_reset: bool,
    pub warm_reset: bool,
    pub shut_down: bool,
    #[skip]
    __: u16,
    pub channels: PcmChannels,
    pub pcm_out_mode: PcmOutMode,
    #[skip]
    __: u8,
}

#[bitfield(bits = 32)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u32)]
pub struct GlobalStatus {
    #[skip]
    __: B20,
    #[skip(setters)]
    pub channel_caps: PcmChannels,
    pub sample_caps: PcmOutMode,
    #[skip]
    __: u8,
}

#[bitfield(bits = 16)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u16)]
pub struct BufferDescCtl {
    #[skip]
    __: B14,
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
