// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

pub fn main() {
    println!("cargo:rustc-link-arg-bins=-TKernel/src/linker.ld");
    println!("cargo:rerun-if-changed=Kernel/src/linker.ld");
}
