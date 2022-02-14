/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use amd64::io::port::Port;
use log::info;

#[repr(u8)]
pub enum PS2CtlCmd {
    ReadControllerCfg = 0x20,
    WriteControllerCfg = 0x60,
    // ResetCPU = 0xFE,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum KeyEvent {
    Pressed(char),
    Released(char),
}

pub struct PS2Ctl {
    data_port: Port<u8>,
    sts_or_cmd_reg: Port<u8>,
}

impl PS2Ctl {
    pub const fn new() -> Self {
        Self {
            data_port: Port::new(0x60),
            sts_or_cmd_reg: Port::new(0x64),
        }
    }

    fn wait_for_ack(&self) {
        unsafe { while self.data_port.read() != 0xFA {} }
    }

    fn output_full(&self) -> bool {
        unsafe { self.sts_or_cmd_reg.read() & 1 != 0 }
    }

    fn input_full(&self) -> bool {
        unsafe { self.sts_or_cmd_reg.read() & (1 << 1) != 0 }
    }

    fn send_cmd(&self, cmd: PS2CtlCmd, wait_for_ack: bool) {
        unsafe { self.sts_or_cmd_reg.write(cmd as u8) };
        if wait_for_ack {
            self.wait_for_ack();
        }
    }

    pub fn init(&mut self) {
        // Flush buffer before doing anything
        while self.output_full() {
            let _ = unsafe { self.data_port.read() };
        }
        info!("ps2: flushed buffer");
        // Disable interrupts for now
        info!("ps2: reading controller config");
        self.send_cmd(PS2CtlCmd::ReadControllerCfg, false);
        while !self.output_full() {}
        info!("ps2: disabling interrupts and translation");
        let cfg = unsafe { self.data_port.read() & !(1u8 | (1u8 << 1)) };
        info!("ps2: writing controller config");
        self.send_cmd(PS2CtlCmd::WriteControllerCfg, false);
        unsafe { self.data_port.write(cfg) }
        while self.input_full() {}
    }

    // pub fn reset_cpu(&self) {
    //     self.send_cmd(PS2ControllerCmd::ResetCPU);
    // }

    pub fn wait_for_key(&self) -> Result<KeyEvent, ()> {
        while !self.output_full() {}

        let key = unsafe { self.data_port.read() };
        match key {
            0xE => Ok(KeyEvent::Pressed('\r')),
            0x2..=0xA => {
                Ok(KeyEvent::Pressed(
                    "123456789".chars().nth(key as usize - 0x2).unwrap(),
                ))
            }
            0x1C => Ok(KeyEvent::Pressed('\n')),
            0x10..=0x1C => {
                Ok(KeyEvent::Pressed(
                    "qwertyuiop".chars().nth(key as usize - 0x10).unwrap(),
                ))
            }
            0x1E..=0x26 => {
                Ok(KeyEvent::Pressed(
                    "asdfghjkl".chars().nth(key as usize - 0x1E).unwrap(),
                ))
            }
            0x29 => Ok(KeyEvent::Pressed('0')),
            0x2C..=0x32 => {
                Ok(KeyEvent::Pressed(
                    "zxcvbnm".chars().nth(key as usize - 0x2C).unwrap(),
                ))
            }
            0x39 => Ok(KeyEvent::Pressed(' ')),
            _ => Err(()),
        }
    }
}
