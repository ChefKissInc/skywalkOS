// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#[repr(C, packed)]
pub struct RootSystemDescTable(super::SDTHeader);

impl core::ops::Deref for RootSystemDescTable {
    type Target = super::SDTHeader;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Debug for RootSystemDescTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RSDT")
            .field("header", &self.0)
            .finish_non_exhaustive()
    }
}
