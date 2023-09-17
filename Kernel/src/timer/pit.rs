// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use amd64::io::port::Port;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// 3 bits
pub enum Mode {
    IntrOnTerminalCount = 0b000,
    OneShot = 0b001,
    RateGenerator = 0b010,
    SquareWaveGenerator = 0b011,
    SwTriggeredStrobe = 0b100,
    HwTriggeredStrobe = 0b101,
}

impl Mode {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0b000 => Self::IntrOnTerminalCount,
            0b001 => Self::OneShot,
            0b010 => Self::RateGenerator,
            0b011 => Self::SquareWaveGenerator,
            0b100 => Self::SwTriggeredStrobe,
            0b101 => Self::HwTriggeredStrobe,
            _ => panic!("Invalid PIT Mode"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// 2 bits
pub enum AccessMode {
    LatchCount = 0b00,
    LoByteOnly = 0b01,
    HiByteOnly = 0b10,
    LoByteOrHiByte = 0b11,
}

impl AccessMode {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0b00 => Self::LatchCount,
            0b01 => Self::LoByteOnly,
            0b10 => Self::HiByteOnly,
            0b11 => Self::LoByteOrHiByte,
            _ => panic!("Invalid PIT AccessMode"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// 2 bits
pub enum Channel {
    Zero = 0b00,
    ReadBackCommand = 0b11,
}

impl Channel {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0b00 => Self::Zero,
            0b11 => Self::ReadBackCommand,
            _ => panic!("Invalid PIT Channel"),
        }
    }
}

#[bitfield(u8)]
pub struct ModeCommand {
    pub bcd: bool,
    #[bits(3)]
    pub mode: Mode,
    #[bits(2)]
    pub access_mode: AccessMode,
    #[bits(2)]
    pub channel: Channel,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct ProgrammableIntervalTimer {
    channel0: Port<u8, u8>,
    mode_cmd: Port<u8, ModeCommand>,
}

#[allow(dead_code)]
impl ProgrammableIntervalTimer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            channel0: Port::new(0x40),
            mode_cmd: Port::new(0x43),
        }
    }

    pub fn with_one_shot(self) -> Self {
        unsafe {
            self.mode_cmd.write(
                ModeCommand::new()
                    .with_bcd(false)
                    .with_mode(Mode::OneShot)
                    .with_access_mode(AccessMode::LoByteOrHiByte)
                    .with_channel(Channel::Zero),
            );
        }
        self
    }

    pub fn read_counter(self) -> u16 {
        unsafe {
            self.mode_cmd.write(ModeCommand::new());
            let lo = u16::from(self.channel0.read());
            let hi = u16::from(self.channel0.read());

            lo | (hi << 8)
        }
    }

    pub fn with_reload(self, val: u16) -> Self {
        let lo = val as u8;
        let hi = (val >> 8) as u8;

        unsafe {
            self.channel0.write(lo);
            self.channel0.write(hi);
        }

        self
    }
}
