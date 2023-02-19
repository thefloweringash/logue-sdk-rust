use std::env;
use std::process::Command;

pub fn configure_osc_build() {
    let ld = format!("{}/ld", env!("CARGO_MANIFEST_DIR"));

    // Set the main linker script
    println!("cargo:rustc-link-arg=-L{ld}");
    println!("cargo:rustc-link-arg=-T{ld}/userosc.ld");

    // Expose absolute symbols
    println!("cargo:rerun-if-changed={ld}/osc_api.c");
    println!("cargo:rerun-if-changed={ld}/osc_api.ld");

    let host_cc = env::var("HOST_CC").expect("Environment variable: HOST_CC to build osc_api");
    let out_dir = env::var("OUT_DIR").expect("Environment variable: OUT_DIR");
    // let target_arch =
    //     env::var("CARGO_CFG_TARGET_ARCH").expect("Environment variable: CARGO_CFG_TARGET_ARCH");
    let target_arch = "armv7e-m"; // TODO: why can't cargo tell me the actual arch?

    // println!(
    //     "env: {}",
    //     String::from_utf8(Command::new("env").output().unwrap().stdout).unwrap()
    // );

    let syms_api_object = format!("{out_dir}/osc_api.o");

    let output = Command::new(host_cc)
        .arg(format!("-march={target_arch}"))
        .arg("-nostartfiles")
        .arg(format!("-Wl,-script={ld}/osc_api.ld"))
        .arg("-o")
        .arg(&syms_api_object)
        .arg(format!("{ld}/osc_api.c"))
        .output()
        .expect("Failed to build api object");

    if !output.status.success() {
        print!("Object compilation failed: {:?}\n", output.status);
        print!("stderr:\n {}", String::from_utf8(output.stderr).unwrap());
        print!("stdout:\n {}", String::from_utf8(output.stdout).unwrap());
        panic!("Could not product object api");
    }

    println!("cargo:rustc-link-arg=--just-symbols={syms_api_object}");
}
