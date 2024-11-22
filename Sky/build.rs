// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub fn main() {
    println!("cargo:rustc-link-arg-bins=-Tsrc/linker.ld");
    println!("cargo:rerun-if-changed=src/linker.ld");
}
