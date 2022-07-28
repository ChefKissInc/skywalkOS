//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::string::String;
use core::fmt::Write;

use log::{error, info};
use sulfur_dioxide::module::Module;

use crate::{
    driver::{
        acpi::ACPIPlatform,
        audio::ac97::AC97,
        keyboard::ps2::Ps2Event,
        pci::{PCIAddress, PCICfgOffset, PCIController, PCIDevice, PCIIOAccessSize},
    },
    sys::terminal::Terminal,
};

pub fn terminal_loop(
    acpi: &ACPIPlatform,
    pci: &PCIController,
    terminal: &mut Terminal,
    mut ac97: Option<&mut AC97>,
) -> ! {
    let ps2ctl = unsafe { (*crate::driver::keyboard::ps2::INSTANCE.get()).assume_init_mut() };
    let state = unsafe { &mut (*crate::sys::state::SYS_STATE.get()) };
    'menu: loop {
        write!(terminal, "\nBoxOS# ").unwrap();
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
                                            r#"Cardboard debug terminal
 Commands  |          Description
clear      |  Clear terminal
greeting   |  Very epic example command
acpidump   |  Dump ACPI information
pcidump    |  Dump PCI devices
audiotest  |  Play test sound through AC97
resume     |  Resume playback
pause      |  Pause playback
restart    |  Restart machine by resetting CPU
help       |  Display this
memusage   |  View memory usage"#
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
                                        for segment in 0..pci.segment_count() {
                                            for bus in 0..=255 {
                                                for slot in 0..32 {
                                                    for func in 0..8 {
                                                        let addr = PCIAddress {
                                                            segment,
                                                            bus,
                                                            slot,
                                                            func,
                                                        };
                                                        let device =
                                                            PCIDevice::new(addr, pci.get_io(addr));

                                                        unsafe {
                                                            let vendor_id: u32 = device.cfg_read(
                                                                PCICfgOffset::VendorId,
                                                                PCIIOAccessSize::Word,
                                                            );
                                                            if vendor_id != 0xFFFF {
                                                                info!(
                                                                    "PCI Device at {}:{}:{} has \
                                                                     vendor ID {:#06X} and device \
                                                                     ID {:#06X}, class code \
                                                                     {:#06X}",
                                                                    bus,
                                                                    slot,
                                                                    func,
                                                                    vendor_id,
                                                                    device.cfg_read::<_, u32>(
                                                                        PCICfgOffset::DeviceId,
                                                                        PCIIOAccessSize::Word,
                                                                    ),
                                                                    device.cfg_read::<_, u32>(
                                                                        PCICfgOffset::ClassCode,
                                                                        PCIIOAccessSize::Word,
                                                                    ),
                                                                )
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    "audiotest" => {
                                        if let Some(ref mut ac97) = ac97 {
                                            if let Some(ref modules) = state.modules {
                                                if let Some(Module::Audio(module)) =
                                                    modules.iter().find(|v| {
                                                        if let Module::Audio(v) = v {
                                                            v.name == "testaudio"
                                                        } else {
                                                            false
                                                        }
                                                    })
                                                {
                                                    info!("Starting playback of test audio");
                                                    ac97.play_audio(module.data)
                                                } else {
                                                    error!(
                                                        "Failure to find 'testaudio' boot loader \
                                                         module"
                                                    );
                                                }
                                            }
                                        } else {
                                            error!("No sound device available!");
                                        }
                                    }
                                    "resume" => {
                                        if let Some(ac97) = &mut ac97 {
                                            info!("Resuming audio playback");
                                            ac97.start_playback();
                                        } else {
                                            error!("No sound device available!");
                                        }
                                    }
                                    "pause" => {
                                        if let Some(ac97) = &mut ac97 {
                                            info!("Pausing audio playback");
                                            ac97.stop_playback();
                                        } else {
                                            error!("No sound device available!");
                                        }
                                    }
                                    "restart" => ps2ctl.reset_cpu(),
                                    "memusage" => {
                                        let pmm = unsafe { state.pmm.assume_init_ref() };
                                        let used = {
                                            let pmm = pmm.lock();
                                            (pmm.total_pages - pmm.free_pages) * 4096 / 1024 / 1024
                                        };
                                        let total = pmm.lock().total_pages * 4096 / 1024 / 1024;
                                        info!("Used memory: {}MiB out of {}MiB", used, total);
                                    }
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
