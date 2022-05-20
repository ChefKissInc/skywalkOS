//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::string::String;
use core::fmt::Write;

use log::{error, info};

use crate::{
    driver::{
        ac97::Ac97,
        acpi::Acpi,
        pci::{Pci, PciAddress, PciConfigOffset, PciDevice, PciIoAccessSize},
        ps2::Ps2Event,
    },
    sys::terminal::Terminal,
};

pub fn terminal_loop(acpi: &Acpi, pci: &Pci, terminal: &mut Terminal, ac97: &mut Option<Ac97>) {
    let ps2ctl = unsafe {
        (&mut *crate::driver::ps2::INSTANCE.get())
            .get_mut()
            .unwrap()
    };
    'menu: loop {
        write!(terminal, "\nFirework# ").unwrap();
        let mut cmd = String::new();
        loop {
            if let Some(key) = ps2ctl.queue.pop_front() {
                match key {
                    Ps2Event::Pressed(c) => {
                        terminal.write_char(c).unwrap();
                        match c {
                            '\n' => {
                                match cmd.as_str() {
                                    "help" => {
                                        info!(
                                            r#"Fuse debug terminal
Available commands:
clear      <= Clear terminal
greeting   <= Very epic example command
acpidump   <= Dump ACPI information
pcidump    <= Dump PCI devices
audiotest  <= Play test sound through AC97
restart    <= Restart machine by resetting CPU
help       <= Display this"#
                                        );
                                    }
                                    "clear" => terminal.clear(),
                                    "greeting" => info!("Greetings, User."),
                                    "acpidump" => {
                                        info!("ACPI version {}", acpi.version);
                                        for table in &acpi.tables {
                                            info!("Table '{}': {:#X?}", table.0, table.1);
                                        }
                                    }
                                    "pcidump" => {
                                        for bus in 0..=255 {
                                            for slot in 0..32 {
                                                for func in 0..8 {
                                                    let device = PciDevice::new(
                                                        PciAddress {
                                                            bus,
                                                            slot,
                                                            func,
                                                            ..Default::default()
                                                        },
                                                        pci.io.as_ref(),
                                                    );
                                                    unsafe {
                                                        let vendor_id = device.cfg_read(
                                                            PciConfigOffset::VendorId,
                                                            PciIoAccessSize::Word,
                                                        );
                                                        if vendor_id != 0xFFFF {
                                                            info!(
                                                                "PCI Device at {}:{}:{} has \
                                                                 vendor ID {:#06X} and device ID \
                                                                 {:#06X}, class code {:#06X}",
                                                                bus,
                                                                slot,
                                                                func,
                                                                vendor_id,
                                                                device.cfg_read(
                                                                    PciConfigOffset::DeviceId,
                                                                    PciIoAccessSize::Word,
                                                                ),
                                                                device.cfg_read(
                                                                    PciConfigOffset::ClassCode,
                                                                    PciIoAccessSize::Word,
                                                                ),
                                                            )
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    "audiotest" => {
                                        if let Some(ac97) = ac97 {
                                            let modules =
                                                unsafe { &*crate::sys::state::SYS_STATE.get() }
                                                    .modules
                                                    .get()
                                                    .unwrap();
                                            if let Some(module) =
                                                modules.iter().find(|v| v.name == "testaudio")
                                            {
                                                info!("Starting playback of test audio");
                                                ac97.play_audio(module.data)
                                            } else {
                                                error!(
                                                    "Failure to find 'testaudio' boot loader \
                                                     module"
                                                );
                                            }
                                        } else {
                                            error!("No sound device available!");
                                        }
                                    }
                                    "restart" => ps2ctl.reset_cpu(),
                                    _ => writeln!(terminal, "Unknown command").unwrap(),
                                }

                                continue 'menu;
                            }
                            _ => cmd.push(c),
                        }
                    }
                    Ps2Event::BackSpace => {
                        if !cmd.is_empty() {
                            cmd.pop();
                            terminal.backspace()
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
