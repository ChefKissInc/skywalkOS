// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

pub fn main() {
    println!("cargo:rustc-link-arg-bins=-TKernel/src/linker.ld");
    println!("cargo:rerun-if-changed=Kernel/src/linker.ld");
}
