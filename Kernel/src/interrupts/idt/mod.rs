// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::cell::SyncUnsafeCell;

use crate::system::{
    gdt::{PrivilegeLevel, SegmentSelector},
    RegisterState,
};

mod isr;

seq_macro::seq!(N in 0..256 {
    static ENTRIES: SyncUnsafeCell<[Entry; 256]> = SyncUnsafeCell::new([
        #(
            Entry::new(
                0,
                SegmentSelector::new(1, PrivilegeLevel::Supervisor),
                0,
                EntryType::InterruptGate,
                PrivilegeLevel::Supervisor,
                true,
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

pub static IDTR: IDTReg = IDTReg {
    limit: (core::mem::size_of_val(&ENTRIES) - 1) as u16,
    base: unsafe { (*ENTRIES.get()).as_ptr() },
};

type HandlerFn = unsafe extern "sysv64" fn(&mut RegisterState);

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

unsafe extern "sysv64" fn default_handler(regs: &mut RegisterState) {
    let n = regs.int_num as u8;
    debug!("No handler for ISR #{n}");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// 4 bits
pub enum EntryType {
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}

impl EntryType {
    const fn into_bits(self) -> u16 {
        self as _
    }

    const fn from_bits(value: u16) -> Self {
        match value {
            0b1110 => Self::InterruptGate,
            0b1111 => Self::TrapGate,
            _ => panic!("Unknown IDT EntryType"),
        }
    }
}

#[bitfield(u16)]
pub struct EntryFlags {
    #[bits(3)]
    pub ist: u8,
    #[bits(5)]
    __: B5,
    #[bits(4, default=EntryType::InterruptGate)]
    pub ty: EntryType,
    __: bool,
    #[bits(2)]
    pub dpl: PrivilegeLevel,
    pub present: bool,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
    pub offset_low: u16,
    pub selector: SegmentSelector,
    pub flags: EntryFlags,
    pub offset_middle: u16,
    pub offset_high: u32,
    __: u32,
}

impl Entry {
    #[inline]
    pub const fn new(
        base: u64,
        selector: SegmentSelector,
        ist: u8,
        ty: EntryType,
        dpl: PrivilegeLevel,
        present: bool,
    ) -> Self {
        Self {
            offset_low: base as u16,
            selector,
            flags: EntryFlags::new()
                .with_ist(ist)
                .with_ty(ty)
                .with_dpl(dpl)
                .with_present(present),
            offset_middle: (base >> 16) as u16,
            offset_high: (base >> 32) as u32,
            __: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IDTReg {
    pub limit: u16,
    pub base: *const Entry,
}

impl IDTReg {
    pub unsafe fn load(&self) {
        debug!("Initialising.");
        seq_macro::seq!(N in 0..256 {
            let base = isr::isr~N as usize as u64;
            let entry = &mut (*ENTRIES.get())[N];
            entry.offset_low = base as u16;
            entry.offset_middle = (base >> 16) as u16;
            entry.offset_high = (base >> 32) as u32;
        });

        core::arch::asm!("lidt [{}]", in(reg) self, options(readonly, preserves_flags));
    }
}

unsafe impl Sync for IDTReg {}

pub fn set_handler(
    isr: u8,
    ist: u8,
    dpl: PrivilegeLevel,
    func: HandlerFn,
    is_irq: bool,
    should_iret: bool,
) {
    let handler = unsafe { &mut (*HANDLERS.get())[isr as usize] };
    assert_eq!(
        handler.func as usize, default_handler as usize,
        "Tried to register already existing ISR #{isr}",
    );

    let ent = unsafe { &mut (*ENTRIES.get())[isr as usize] };
    ent.flags = ent.flags.with_dpl(dpl).with_ist(ist);

    *handler = InterruptHandler {
        func,
        is_irq,
        should_iret,
    };
}
