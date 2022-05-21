//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use amd64::{
    registers::msr::{apic::ApicBase, Msr},
    sys::apic::{LocalApic, SpuriousIntrVector},
};

pub fn get_set_lapic_addr() -> u64 {
    unsafe {
        let addr = (*crate::sys::state::SYS_STATE.get())
            .madt
            .assume_init_mut()
            .lapic_addr;
        ApicBase::read().with_apic_base(addr).write();
        addr
    }
}

pub trait ApicHelper {
    fn enable(&self);
}

impl ApicHelper for LocalApic {
    fn enable(&self) {
        self.write_spurious_intr_vec(
            SpuriousIntrVector::new()
                .with_vector(0xFF)
                .with_apic_soft_enable(true),
        )
    }
}
