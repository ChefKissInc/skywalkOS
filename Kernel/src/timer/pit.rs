// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use amd64::io::port::Port;
use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier, Clone, Copy)]
#[bits = 3]
#[repr(u8)]
pub enum Mode {
    IntrOnTerminalCount = 0b000,
    OneShot = 0b001,
    RateGenerator = 0b010,
    SquareWaveGenerator = 0b011,
    SwTriggeredStrobe = 0b100,
    HwTriggeredStrobe = 0b101,
}

#[derive(BitfieldSpecifier, Clone, Copy)]
#[bits = 2]
#[repr(u8)]
pub enum AccessMode {
    LatchCount = 0b00,
    LoByteOnly = 0b01,
    HiByteOnly = 0b10,
    LoByteOrHiByte = 0b11,
}

#[derive(BitfieldSpecifier, Clone, Copy)]
#[bits = 2]
#[repr(u8)]
pub enum Channel {
    Zero = 0b00,
    ReadBackCommand = 0b11,
}

#[bitfield(bits = 8)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub struct ModeCommand {
    pub bcd: bool,
    pub mode: Mode,
    pub access_mode: AccessMode,
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
    #[inline(always)]
    #[must_use]
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
