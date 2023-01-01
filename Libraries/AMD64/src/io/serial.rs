// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptEnable {
    pub data_available: bool,
    pub transmitter_empty: bool,
    pub break_or_error: bool,
    pub status_change: bool,
    #[skip]
    __: B4,
}

#[derive(Debug, Clone, Copy, BitfieldSpecifier)]
#[bits = 2]
pub enum DataBits {
    FiveBits = 0b00,
    SixBits = 0b01,
    SevenBits = 0b10,
    EightBits = 0b11,
}

#[derive(Debug, Clone, Copy, BitfieldSpecifier)]
#[bits = 1]
pub enum StopBits {
    OneBit = 0b0,
    OnePointFiveDividedBy2 = 0b1,
}

#[derive(Debug, Clone, Copy, BitfieldSpecifier)]
#[bits = 3]
pub enum Parity {
    None = 0b000,
    Odd = 0b001,
    Even = 0b011,
    Mark = 0b101,
    Space = 0b111,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub struct LineControl {
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    #[skip]
    __: B1,
    pub dlab: bool,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub struct LineStatus {
    pub data_ready: bool,
    pub overrun_error: bool,
    pub parity_error: bool,
    pub framing_error: bool,
    pub break_indicator: bool,
    pub transmitter_empty: bool,
    pub transmitter_idle: bool,
    pub impending_error: bool,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub struct ModemControl {
    #[skip]
    __: B2,
    pub autoflow: bool,
    pub loopback: bool,
    pub aux_out_1: bool,
    pub aux_out_2: bool,
    pub req_send: bool,
    pub terminal_ready: bool,
}

pub struct SerialPort {
    port: super::port::Port<u8, u8>,
}

#[repr(u16)]
#[derive(IntoPrimitive)]
pub enum SerialPortReg {
    DataOrDivisor = 0,
    EnableIntrOrDivisorHigh,
    IntrIDOrFifo,
    LineControl,
    ModemControl,
    LineStatus,
}

impl SerialPort {
    #[must_use]
    pub const fn new(port_num: u16) -> Self {
        Self {
            port: super::port::Port::new(port_num),
        }
    }

    #[inline]
    fn line_status(&self) -> LineStatus {
        unsafe { self.port.read_off(SerialPortReg::LineStatus) }
    }

    #[inline]
    pub fn set_intr_enable(&self, val: u8) {
        unsafe {
            self.port
                .write_off(val, SerialPortReg::EnableIntrOrDivisorHigh);
        }
    }

    #[inline]
    fn can_send_data(&self) -> bool {
        self.line_status().transmitter_empty()
    }

    #[inline]
    fn set_line_ctl(&self, val: LineControl) {
        unsafe { self.port.write_off(val, SerialPortReg::LineControl) }
    }

    #[inline]
    fn set_modem_ctl(&self, val: ModemControl) {
        unsafe { self.port.write_off(val, SerialPortReg::ModemControl) }
    }

    #[inline]
    pub fn init(&self) {
        unsafe {
            self.set_intr_enable(0);
            self.set_line_ctl(LineControl::new().with_dlab(true));
            self.port.write_off(1, SerialPortReg::DataOrDivisor);
            self.set_intr_enable(0);
            self.set_line_ctl(
                LineControl::new()
                    .with_parity(Parity::None)
                    .with_data_bits(DataBits::EightBits),
            );
            // Disable FIFO
            self.port.write_off(0, SerialPortReg::IntrIDOrFifo);
            // Enable data terminal
            self.set_modem_ctl(
                ModemControl::new()
                    .with_terminal_ready(true)
                    .with_aux_out_2(true),
            );
        }
    }

    pub fn transmit(&self, value: u8) {
        while !self.can_send_data() {}

        unsafe { self.port.write(value) }
    }

    fn can_receive_data(&self) -> bool {
        self.line_status().data_ready()
    }

    #[must_use]
    pub fn receive(&self) -> u8 {
        while !self.can_receive_data() {}

        unsafe { self.port.read() }
    }
}
