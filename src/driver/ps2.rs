//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::collections::VecDeque;
use core::{cell::SyncUnsafeCell, mem::MaybeUninit};

use amd64::{io::port::Port, sys::cpu::RegisterState};
use log::debug;
use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum PS2CtlCmd {
    ReadControllerCfg = 0x20,
    WriteControllerCfg = 0x60,
    ResetCPU = 0xFE,
}

#[bitfield(bits = 8)]
#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub struct Ps2Cfg {
    pub port1_intr: bool,
    pub port2_intr: bool,
    pub post_pass: bool,
    #[skip]
    __: bool,
    pub port1_clock: bool,
    pub port2_clock: bool,
    pub port1_translation: bool,
    #[skip]
    __: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Ps2Event {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    BackSpace,
    Pressed(char),
    Released(char),
    Other(u8),
}

pub struct PS2Ctl {
    data_port: Port<u8, u8>,
    sts_or_cmd_reg: Port<u8, u8>,
    pub queue: VecDeque<Ps2Event>,
}

pub static INSTANCE: SyncUnsafeCell<MaybeUninit<PS2Ctl>> = SyncUnsafeCell::new(MaybeUninit::uninit());

pub(crate) unsafe extern "sysv64" fn handler(_state: &mut RegisterState) {
    debug!("PS/2 interrupt handler called!");
    let this = (*INSTANCE.get()).assume_init_mut();
    let key = this.data_port.read();
    this.queue.push_back(match key {
        0xE => Ps2Event::BackSpace,
        0x2..=0xA => Ps2Event::Pressed("123456789".chars().nth(key as usize - 0x2).unwrap()),
        0x1C => Ps2Event::Pressed('\n'),
        0x10..=0x1C => Ps2Event::Pressed("qwertyuiop".chars().nth(key as usize - 0x10).unwrap()),
        0x1E..=0x26 => Ps2Event::Pressed("asdfghjkl".chars().nth(key as usize - 0x1E).unwrap()),
        0x29 => Ps2Event::Pressed('0'),
        0x2C..=0x32 => Ps2Event::Pressed("zxcvbnm".chars().nth(key as usize - 0x2C).unwrap()),
        0x39 => Ps2Event::Pressed(' '),
        v => Ps2Event::Other(v),
    });
    debug!("Done: {:?}", this.queue);
}

impl PS2Ctl {
    pub fn new() -> Self {
        Self {
            data_port: Port::new(0x60),
            sts_or_cmd_reg: Port::new(0x64),
            queue: VecDeque::new(),
        }
    }

    #[inline]
    fn output_full(&self) -> bool {
        unsafe { self.sts_or_cmd_reg.read() & 1 != 0 }
    }

    #[inline]
    fn input_full(&self) -> bool {
        unsafe { self.sts_or_cmd_reg.read() & (1 << 1) != 0 }
    }

    #[inline]
    fn send_cmd(&self, cmd: PS2CtlCmd, wait_for_ack: bool) {
        unsafe {
            self.sts_or_cmd_reg.write(cmd.into());
            if wait_for_ack {
                while self.data_port.read() != 0xFA {}
            }
        }
    }

    pub fn init(&mut self) {
        // Flush buffer before doing anything
        while self.output_full() {
            let _ = unsafe { self.data_port.read() };
        }
        debug!("Flushed buffer");
        // Disable interrupts for now
        debug!("Reading controller config");
        self.send_cmd(PS2CtlCmd::ReadControllerCfg, false);
        while !self.output_full() {}
        debug!("Enabling interrupts");
        let cfg = unsafe {
            Ps2Cfg::from(self.data_port.read())
                .with_port1_intr(true)
                .with_port2_intr(false)
        };
        crate::driver::acpi::ioapic::wire_legacy_irq(1, false);
        crate::sys::idt::set_handler(0x21, handler, true, true);
        debug!("Writing controller config");
        self.send_cmd(PS2CtlCmd::WriteControllerCfg, false);
        unsafe { self.data_port.write(cfg.into()) }
        while self.input_full() {}
    }

    pub fn reset_cpu(&self) {
        self.send_cmd(PS2CtlCmd::ResetCPU, false);
    }
}
