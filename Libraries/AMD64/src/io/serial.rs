// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use num_enum::IntoPrimitive;

#[bitfield(u8)]
pub struct InterruptEnable {
    pub data_available: bool,
    pub transmitter_empty: bool,
    pub break_or_error: bool,
    pub status_change: bool,
    #[bits(4)]
    __: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// 2 bits
pub enum DataBits {
    FiveBits = 0b00,
    SixBits = 0b01,
    SevenBits = 0b10,
    EightBits = 0b11,
}

impl DataBits {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0b00 => Self::FiveBits,
            0b01 => Self::SixBits,
            0b10 => Self::SevenBits,
            0b11 => Self::EightBits,
            _ => panic!("Invalid DataBits"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// 1 bit
pub enum StopBits {
    OneBit = 0b0,
    OnePointFiveDividedBy2 = 0b1,
}

impl StopBits {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0b0 => Self::OneBit,
            0b1 => Self::OnePointFiveDividedBy2,
            _ => panic!("Invalid StopBits"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// 3 bits
pub enum Parity {
    None = 0b000,
    Odd = 0b001,
    Even = 0b011,
    Mark = 0b101,
    Space = 0b111,
}

impl Parity {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0b000 => Self::None,
            0b001 => Self::Odd,
            0b011 => Self::Even,
            0b101 => Self::Mark,
            0b111 => Self::Space,
            _ => panic!("Invalid Parity"),
        }
    }
}

#[bitfield(u8)]
pub struct LineControl {
    #[bits(2)]
    pub data_bits: DataBits,
    #[bits(1)]
    pub stop_bits: StopBits,
    #[bits(3)]
    pub parity: Parity,
    #[skip]
    __: bool,
    pub dlab: bool,
}

#[bitfield(u8)]
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

#[bitfield(u8)]
pub struct ModemControl {
    #[bits(2)]
    __: u8,
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
    IntrIDOrFIFO,
    LineControl,
    ModemControl,
    LineStatus,
}

impl SerialPort {
    #[inline]
    pub const fn new(port_num: u16) -> Self {
        Self {
            port: super::port::Port::new(port_num),
        }
    }

    fn line_status(&self) -> LineStatus {
        unsafe { self.port.read_off(SerialPortReg::LineStatus) }
    }

    pub fn set_intr_enable(&self, val: u8) {
        unsafe {
            self.port
                .write_off(val, SerialPortReg::EnableIntrOrDivisorHigh);
        }
    }

    fn can_send_data(&self) -> bool {
        self.line_status().transmitter_empty()
    }

    fn set_line_ctl(&self, val: LineControl) {
        unsafe { self.port.write_off(val, SerialPortReg::LineControl) }
    }

    fn set_modem_ctl(&self, val: ModemControl) {
        unsafe { self.port.write_off(val, SerialPortReg::ModemControl) }
    }

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
            self.port.write_off(0, SerialPortReg::IntrIDOrFIFO);
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

    pub fn receive(&self) -> u8 {
        while !self.can_receive_data() {}

        unsafe { self.port.read() }
    }
}
