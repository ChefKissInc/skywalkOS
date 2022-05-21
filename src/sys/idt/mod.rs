//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use core::cell::SyncUnsafeCell;

use log::{debug, error};

mod isr;

seq_macro::seq!(N in 0..256 {
    static ENTRIES: SyncUnsafeCell<[amd64::sys::idt::Entry; 256]> = SyncUnsafeCell::new([
        #(
            amd64::sys::idt::Entry::new(
                0,
                amd64::sys::cpu::SegmentSelector::new(1, amd64::sys::cpu::PrivilegeLevel::Hypervisor),
                0,
                amd64::sys::idt::EntryType::InterruptGate, 0, true
            ),
        )*
    ]);
});

seq_macro::seq!(N in 0..256 {
    static HANDLERS: SyncUnsafeCell<[InterruptHandler; 256]> = SyncUnsafeCell::new([
        #(
            InterruptHandler {
                func: default_handler,
                is_irq: false,
                should_iret: false,
            },
        )*
    ]);
});

type HandlerFn = unsafe extern "sysv64" fn(&mut amd64::sys::cpu::RegisterState);

pub struct InterruptHandler {
    pub func: HandlerFn,
    pub is_irq: bool,
    pub should_iret: bool,
}

impl core::fmt::Debug for InterruptHandler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InterruptHandler")
            .field("func", &(self.func as usize))
            .field("is_irq", &self.is_irq)
            .field("should_iret", &self.should_iret)
            .finish()
    }
}

unsafe extern "sysv64" fn default_handler(regs: &mut amd64::sys::cpu::RegisterState) {
    let n = (regs.int_num & 0xFF) as u8;
    debug!("No handler for ISR #{}", n);
}

pub unsafe fn init() {
    seq_macro::seq!(N in 0..256 {
        let base = isr::isr~N as usize;
        let entry = &mut (*ENTRIES.get())[N];
        entry.offset_low = base as u16;
        entry.offset_middle = (base >> 16) as u16;
        entry.offset_high = (base >> 32) as u32;
    });

    let idtr = amd64::sys::idt::Idtr {
        limit: (core::mem::size_of_val(&ENTRIES) - 1) as u16,
        base: (*ENTRIES.get()).as_ptr(),
    };

    idtr.load()
}

pub fn set_handler(isr: u64, func: HandlerFn, is_irq: bool, should_iret: bool) {
    unsafe {
        let handler = &mut HANDLERS.get().as_mut().unwrap()[isr as usize];

        if handler.func as usize != default_handler as usize {
            error!(
                "Tried to register already existing ISR #{}. This is most likely a bug!",
                isr
            )
        }

        *handler = InterruptHandler {
            func,
            is_irq,
            should_iret,
        };
    }
}
