//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use amd64::{
    intrs::apic::{LocalAPIC, SpuriousIntrVector},
    registers::msr::{apic::APICBase, ModelSpecificReg},
};

pub fn get_set_lapic_addr() -> u64 {
    unsafe {
        let addr = (*crate::sys::state::SYS_STATE.get())
            .madt
            .assume_init_mut()
            .lapic_addr;
        APICBase::read().with_apic_base(addr).write();
        addr
    }
}

pub trait APICHelper {
    fn enable(&self);
}

impl APICHelper for LocalAPIC {
    fn enable(&self) {
        self.write_spurious_intr_vec(
            SpuriousIntrVector::new()
                .with_vector(0xFF)
                .with_apic_soft_enable(true),
        )
    }
}
