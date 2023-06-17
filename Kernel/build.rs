// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

pub fn main() {
    println!("cargo:rustc-link-arg-bins=-Tsrc/linker.ld");
    println!("cargo:rerun-if-changed=src/linker.ld");
}
