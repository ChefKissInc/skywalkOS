use core::cell::UnsafeCell;

use log::info;

mod isr;

static ENTRIES: spin::Once<[amd64::sys::idt::Entry; 256]> = spin::Once::new();

seq_macro::seq!(N in 0..256 {
    static HANDLERS: SafeCell<[InterruptHandler; 256]> = SafeCell(UnsafeCell::new([
        #(
            InterruptHandler {
                func: default_handler,
                is_irq: false,
                should_iret: false,
            },
        )*
    ]));
});

pub struct SafeCell<T: ?Sized>(pub UnsafeCell<T>);

impl<T: ?Sized> core::ops::Deref for SafeCell<T> {
    type Target = UnsafeCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T: ?Sized> Sync for SafeCell<T> {}

pub struct InterruptHandler {
    pub func: unsafe extern "sysv64" fn(&mut amd64::sys::cpu::RegisterState),
    pub is_irq: bool,
    pub should_iret: bool,
}

unsafe extern "sysv64" fn default_handler(regs: &mut amd64::sys::cpu::RegisterState) {
    let n = (regs.int_num & 0xFF) as u8;
    info!("No handler for ISR {}", n);
}

pub unsafe fn init() {
    seq_macro::seq!(N in 0..256 {
        ENTRIES.call_once(||
            [
                #(
                    amd64::sys::idt::Entry::new(
                        isr::isr #N as usize as u64,
                        amd64::sys::cpu::SegmentSelector::new(1, amd64::sys::cpu::PrivilegeLevel::Hypervisor),
                        0,
                        amd64::sys::idt::EntryType::InterruptGate, 0, true
                    ),
                )*
            ]
        );
    });

    let idtr = amd64::sys::idt::Idtr {
        limit: (core::mem::size_of_val(&ENTRIES) - 1) as u16,
        base: ENTRIES.get().unwrap().as_ptr(),
    };

    idtr.load()
}
