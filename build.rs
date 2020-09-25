use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        // Set-up cross-compilation
        .clang_arg("-target")
        .clang_arg("armv7a-none-eabi")
        // Include Xilinx cross-compiler libc headers
        .clang_arg("-I/mnt/e/provisional/Software/Xilinx_WSL1/SDK/2019.1/gnu/aarch32/lin/gcc-arm-none-eabi/arm-none-eabi/libc/usr/include")
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // BSP includes
        .clang_arg("-I./include")
        // Use core instead of std to retain no_std compatibility
        .use_core()
        .ctypes_prefix("cty")
        // Blacklist the types that have the same name in C and Rust -> no bindings needed.
        .blacklist_type("u8|u16|u32|u64")
        // Do not generate tests, because I can't be bothered to set up #[test] in the build environment of the cross-compiler
        .layout_tests(false)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/xil.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Copy the libxil_sf.a library into the target directory to make it available
    // to dependencies TODO: allow overriding the library via an environment
    // variable
    const LIB_NAME: &str = "libxil_sf.a";
    let from: PathBuf = [&env::var("CARGO_MANIFEST_DIR").unwrap(), LIB_NAME]
        .iter()
        .collect();
    let to: PathBuf = [&env::var("OUT_DIR").unwrap(), LIB_NAME].iter().collect();
    fs::copy(from, to).unwrap();

    // Allow dependent libraries to discover the copied static library
    println!("cargo:root={}", env::var("OUT_DIR").unwrap());

    // Find the static library (libxil_sf.a) in the out directory
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );
    // Link the libxil_sf.a in the out directory
    // FIXME: I believe this does not currently link it right
    println!("cargo:rustc-link-lib=static=xil_sf");
}
