use std::env;
use std::process::Command;

pub fn configure_osc_build() {
    let ld = format!("{}/ld", env!("CARGO_MANIFEST_DIR"));

    // Set the main linker script
    println!("cargo:rustc-link-arg=-L{ld}");
    println!("cargo:rustc-link-arg=-T{ld}/userosc.ld");

    // Expose absolute symbols
    println!("cargo:rustc-link-arg={ld}/osc_api.syms");
}
