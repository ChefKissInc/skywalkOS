// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Module {
    pub name: &'static str,
    pub data: &'static [u8],
}

impl core::fmt::Debug for Module {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Module({:?})", self.name))
    }
}
