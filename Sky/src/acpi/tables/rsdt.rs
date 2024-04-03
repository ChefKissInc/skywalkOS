// Copyright (c) ChefKiss Inc 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[derive(Debug)]
#[repr(transparent)]
pub struct RootSystemDescTable(super::SystemDescTableHeader);

impl core::ops::Deref for RootSystemDescTable {
    type Target = super::SystemDescTableHeader;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
