// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[repr(C, packed)]
pub struct RSDT {
    header: super::SDTHeader,
}

impl core::ops::Deref for RSDT {
    type Target = super::SDTHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

impl core::fmt::Debug for RSDT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Rsdt")
            .field("header", &self.header)
            .finish_non_exhaustive()
    }
}
