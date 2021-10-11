pub fn main() {
    println!("cargo:rustc-link-arg-bins=-Tsrc/linker.ld");
    println!("cargo:rerun-if-changed=src/linker.ld");
}
