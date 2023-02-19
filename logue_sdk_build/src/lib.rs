pub fn configure_osc_build() {
    // Set the main linker script
    print!("cargo:rustc-link-arg=-L{}", env!("CARGO_MANIFEST_DIR"));
    print!(
        "cargo:rustc-link-arg=-T{}/ld/userosc.ld",
        env!("CARGO_MANIFEST_DIR")
    );
}
