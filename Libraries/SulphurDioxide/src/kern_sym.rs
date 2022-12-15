// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct KernSymbol {
    pub start: u64,
    pub end: u64,
    pub name: &'static str,
}
