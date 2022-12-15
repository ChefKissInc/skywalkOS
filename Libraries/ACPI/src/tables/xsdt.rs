// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[repr(C, packed)]
pub struct XSDT(super::SDTHeader);

impl core::ops::Deref for XSDT {
    type Target = super::SDTHeader;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Debug for XSDT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("XSDT")
            .field("header", &self.0)
            .finish_non_exhaustive()
    }
}
