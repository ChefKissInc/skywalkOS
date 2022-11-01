// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![deny(
    warnings,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    unused_extern_crates,
    rust_2021_compatibility
)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::too_many_lines,
    clippy::module_name_repetitions,
    clippy::similar_names
)]

pub mod cpuid;
pub mod io;
pub mod paging;
pub mod registers;
pub mod spec;
