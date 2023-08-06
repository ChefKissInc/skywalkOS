// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::multiple_crate_versions)]

// #[macro_use]
// extern crate log;
#[macro_use]
extern crate alloc;

use alloc::string::String;
use core::fmt::Write;

use fireworkkit::{
    msg::Message,
    osdtentry::{OSDTEntry, FKEXT_PROC_KEY, OSDTENTRY_NAME_KEY},
    osvalue::OSValue,
    syscall::SystemCall,
    userspace::{logger::KWriter, port::Port},
};
use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};

#[derive(IntoPrimitive)]
#[repr(u8)]
enum PS2CtlCmd {
    ReadControllerCfg = 0x20,
    WriteControllerCfg = 0x60,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
enum Ps2Event {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    BackSpace,
    Pressed(char),
    Released(char),
    Other(u8),
}

struct PS2Ctl {
    data_port: Port<u8, u8>,
    sts_or_cmd_reg: Port<u8, u8>,
}

impl PS2Ctl {
    #[inline]
    pub const fn new() -> Self {
        Self {
            data_port: Port::new(0x60),
            sts_or_cmd_reg: Port::new(0x64),
            // queue: VecDeque::new(),
        }
    }

    #[inline]
    fn output_full(&self) -> bool {
        unsafe { (self.sts_or_cmd_reg.read() & 1) != 0 }
    }

    #[inline]
    fn input_full(&self) -> bool {
        unsafe { (self.sts_or_cmd_reg.read() & (1 << 1)) != 0 }
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

    pub fn init(&self) {
        while self.output_full() {
            let _ = unsafe { self.data_port.read() };
        }

        self.send_cmd(PS2CtlCmd::ReadControllerCfg, false);
        while !self.output_full() {}

        let cfg = unsafe {
            Ps2Cfg::from(self.data_port.read())
                .with_port1_intr(true)
                .with_port2_intr(false)
                .with_port1_translation(true)
        };
        unsafe { SystemCall::register_irq_handler(1) }
        self.send_cmd(PS2CtlCmd::WriteControllerCfg, false);
        unsafe { self.data_port.write(cfg.into()) }
        while self.input_full() {}
    }
}

fn print_ent(ent: OSDTEntry, ident: usize) {
    let spacing = " ".repeat(ident);

    let id: u64 = ent.into();
    let props = ent.properties();
    writeln!(
        KWriter,
        "{spacing}+ {} <{}>",
        if let Some(OSValue::String(v)) = props.get(OSDTENTRY_NAME_KEY) {
            v.as_str()
        } else {
            "Unnamed"
        },
        id
    )
    .unwrap();

    for (k, v) in props.into_iter().filter(|(k, _)| k != OSDTENTRY_NAME_KEY) {
        writeln!(KWriter, "{spacing}|- {k}: {v:X?}").unwrap();
    }

    for child in ent.children() {
        print_ent(child, ident + 2);
    }
}

#[no_mangle]
extern "C" fn _start(instance: OSDTEntry) -> ! {
    fireworkkit::userspace::logger::init();

    let this = PS2Ctl::new();
    this.init();
    let mut s = String::new();
    write!(KWriter, "# ").unwrap();
    loop {
        let msg = unsafe { Message::recv() };
        if msg.pid != 0 {
            continue;
        }

        while this.output_full() {
            let event = match unsafe { this.data_port.read() } {
                0xE => Ps2Event::BackSpace,
                v @ 0x2..=0xB => {
                    Ps2Event::Pressed("1234567890".chars().nth(v as usize - 0x2).unwrap())
                }
                0x1C => Ps2Event::Pressed('\n'),
                v @ 0x10..=0x1C => {
                    Ps2Event::Pressed("qwertyuiop".chars().nth(v as usize - 0x10).unwrap())
                }
                v @ 0x1E..=0x26 => {
                    Ps2Event::Pressed("asdfghjkl".chars().nth(v as usize - 0x1E).unwrap())
                }
                0x29 => Ps2Event::Pressed('0'),
                v @ 0x2C..=0x32 => {
                    Ps2Event::Pressed("zxcvbnm".chars().nth(v as usize - 0x2C).unwrap())
                }
                0x39 => Ps2Event::Pressed(' '),
                v => Ps2Event::Other(v),
            };

            let Ps2Event::Pressed(ch) = event else {
                continue;
            };
            write!(KWriter, "{ch}").unwrap();

            if ch != '\n' {
                s.push(ch);
                continue;
            }

            match s.as_str() {
                "osdt" => print_ent(OSDTEntry::default(), 0),
                "msgparent" => {
                    let pid: u64 = instance
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .get_property(FKEXT_PROC_KEY)
                        .unwrap()
                        .try_into()
                        .unwrap();

                    unsafe {
                        Message::new(pid, vec![1, 2, 3, 4].leak()).send();
                    }
                }
                v if v.split_whitespace().next() == Some("msg") => 'a: {
                    let mut v = v.split_whitespace().skip(1);
                    let Some(pid) = v.next().and_then(|v| v.parse().ok()) else {
                        writeln!(KWriter, "Expected PID").unwrap();
                        break 'a;
                    };
                    let Some(data) = v.next().and_then(|v| v.parse::<u64>().ok()) else {
                        writeln!(KWriter, "Expected data").unwrap();
                        break 'a;
                    };
                    unsafe {
                        Message::new(pid, data.to_be_bytes().to_vec().leak()).send();
                    }
                }
                _ => writeln!(KWriter, "{s}").unwrap(),
            }
            write!(KWriter, "# ").unwrap();
            s.clear();
        }
    }
}
