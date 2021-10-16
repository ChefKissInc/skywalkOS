/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

pub fn main() {
    println!("cargo:rustc-link-arg-bins=-Tsrc/linker.ld");
    println!("cargo:rerun-if-changed=src/linker.ld");
}
