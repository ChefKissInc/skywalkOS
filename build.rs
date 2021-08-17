fn main() {
    println!("cargo:rustc-link-arg-bins=-Tsrc/linker.ld")
}
