// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u64)]
pub enum AccessSize {
    Byte,
    Word,
    DWord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u64)]
pub enum SystemCall {
    KPrint,
    ReceiveMessage,
    SendMessage,
    Quit,
    Yield,
    PortIn,
    PortOut,
    RegisterIRQHandler,
    Allocate,
    Free,
    AckMessage,
    NewOSDTEntry,
    GetOSDTEntryInfo,
    SetOSDTEntryProp,
}

#[cfg(feature = "userspace")]
impl SystemCall {
    pub unsafe fn quit() -> ! {
        core::arch::asm!("int 249", in("rdi") Self::Quit as u64, options(nostack, preserves_flags, noreturn));
    }

    pub unsafe fn r#yield() {
        core::arch::asm!("int 249", in("rdi") Self::Yield as u64, options(nostack, preserves_flags));
    }

    pub unsafe fn register_irq_handler(irq: u8) {
        core::arch::asm!(
            "int 249",
            in("rdi") Self::RegisterIRQHandler as u64,
            in("sil") irq,
            options(nostack, preserves_flags),
        );
    }
}
