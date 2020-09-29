use std::fs;
use std::path;
use std::{env, process};

const XILINX_SDK_ENV_VAR_NAME: &str = "XILINX_SDK";
const XILINX_ENV_VAR_NAME: &str = "XILINX";
const DEFAULT_XILINX_WIN_PATH: &str = "/c/Xilinx";
const DEFAULT_XILINX_LIN_PATH: &str = "/opt/Xilinx";
const SDK_DIR_NAME: &str = "SDK";
const DEFAULT_XILINX_SDK_VERSION: &str = "2019.1";

const PYNQ_XCOMPILER_PROVIDER: &str = "gnu";
const PYNQ_XCOMPILER_ARCH: &str = "aarch32";
#[cfg(not(windows))]
const PYNQ_XCOMPILER_OS: &str = "lin";
#[cfg(windows)]
const PYNQ_XCOMPILER_OS: &str = "nt";
const PYNQ_XCOMPILER_TOOL_NAME: &str = "gcc-arm-none-eabi";
const PYNQ_XCOMPILER_NAME: &str = "arm-none-eabi";
#[cfg(not(windows))]
const LIBC_H_RELATIVE_LOCATION: &str = "libc/usr/include";
#[cfg(windows)]
const LIBC_H_RELATIVE_LOCATION: &str = "libc/usr/include/linux";
const STDINT_H_RELATIVE_LOCATION: &str = "libc/usr/include";

/// Guess the Xilinx SDK install path like ".../Xilinx/SDK/version"
fn guess_xil_sdk_path() -> path::PathBuf {
    // If XILINX_SDK environment variable exists, use that
    let xil_sdk_env = env::var(XILINX_SDK_ENV_VAR_NAME);
    if let Ok(xil_sdk_env) = xil_sdk_env {
        return path::Path::new(&xil_sdk_env).to_path_buf();
    }

    // Like ".../Xilinx"
    let xil_dir =
        // If XILINX environment variable exists, use that
        if let Ok(xil_env) = env::var(XILINX_ENV_VAR_NAME) {
            xil_env
        }
        // Otherwise try to guess a path based on platform
        else if  cfg!(windows) {
            DEFAULT_XILINX_WIN_PATH.to_owned()
        } else if cfg!(not(windows)) {
            DEFAULT_XILINX_LIN_PATH.to_owned()
        } else {
            eprintln!("cannot detect Xilinx SDK location for this OS, please make sure Xilinx SDK is installed and set the XILINX_SDK environment variable to the directory path where Xilinx SDK is installed, like so: XILINX_SDK=.../Xilinx/SDK/version");
            process::exit(1)
        };
    let xil_dir = path::Path::new(&xil_dir);

    let no_tools_at_all = !xil_dir.exists();
    // Add to comprise ".../Xilinx/SDK"
    let sdk_parent_dir = xil_dir.join(SDK_DIR_NAME).to_owned();

    if !sdk_parent_dir.exists() {
        eprintln!("cannot detect Xilinx SDK at {:?}, please make sure Xilinx SDK is installed and set the XILINX_SDK environment variable to the directory path where Xilinx SDK is installed, like so: XILINX_SDK=.../Xilinx/SDK/version", sdk_parent_dir);
        if no_tools_at_all {
            eprintln!("cannot detect any Xilinx tools at {:?}", xil_dir);
        }
        process::exit(1)
    }

    // Then try to guess a version
    let sdk_dir = sdk_parent_dir.join(DEFAULT_XILINX_SDK_VERSION);
    if sdk_dir.exists() {
        sdk_dir
    } else {
        guess_xil_sdk_ver_path(&sdk_parent_dir)
    }
}

fn guess_xil_sdk_ver_path(xil_sdk_parent_dir: &path::Path) -> path::PathBuf {
    let mut entries = fs::read_dir(xil_sdk_parent_dir)
        .expect(&format!("cannot read contents of {:?}", xil_sdk_parent_dir))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name())
        // Filter out anything that doesn't start with a number
        .filter(|name| {
            name.to_str().map_or(false, |name| {
                name.chars().nth(0).map_or(false, |c| c.is_digit(10))
            })
        })
        .collect::<Vec<_>>();

    // Sorts by filename, smallest number first
    entries.sort();

    // Take the last element, which is the latest version
    match entries.last() {
        Some(name) => xil_sdk_parent_dir.join(name),
        None => {
            eprintln!(
                "Xilinx SDK directory {:?} contains no installed version",
                xil_sdk_parent_dir
            );
            process::exit(1)
        }
    }
}

/// Returns the Xilinx SDK path like ".../Xilinx/SDK/<ver>"
fn locate_xil_sdk_path() -> path::PathBuf {
    let xil_dir = guess_xil_sdk_path();
    let xil_dir = path::Path::new(&xil_dir);

    if !xil_dir.exists() {
        let export_cmd = "export XILINX_SDK=/path/to/Xilinx/SDK/version";
        eprintln!(
            "Xilinx SDK does not exist at path {:?}. Please make sure Xilinx SDK is installed, and set the correct path using `{}`",
            xil_dir, export_cmd
        );
        process::exit(1);
    }

    if !xil_dir.is_dir() {
        eprintln!("{:?} is not a directory", xil_dir);
        process::exit(1)
    }

    xil_dir.to_path_buf()
}

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    // Locate the Xilinx toolchain directory (like ".../Xilinx/SDK/2019.1"), or
    // prompt the user for it
    let xil_sdk_dir = locate_xil_sdk_path();

    // Like so: "$(XILINX_PATH)/gnu/aarch32/lin/gcc-arm-none-eabi/arm-none-eabi/
    // libc/usr/include"
    let xcompiler_path: path::PathBuf = [
        xil_sdk_dir
            .into_os_string()
            .to_str()
            .expect("path needs to be UTF-8"),
        PYNQ_XCOMPILER_PROVIDER,
        PYNQ_XCOMPILER_ARCH,
        PYNQ_XCOMPILER_OS,
        PYNQ_XCOMPILER_TOOL_NAME,
        PYNQ_XCOMPILER_NAME,
    ]
    .iter()
    .collect();

    if !xcompiler_path.exists() {
        eprintln!(
            "the path for cross-compiler does not exist at {:?}",
            xcompiler_path
        );
        process::exit(1)
    }

    let libc_h_path: path::PathBuf = xcompiler_path.join(LIBC_H_RELATIVE_LOCATION);

    let stdint_h_path: path::PathBuf = xcompiler_path.join(STDINT_H_RELATIVE_LOCATION);

    let bindings = bindgen::Builder::default()
        // Set-up cross-compilation
        .clang_arg("-target")
        .clang_arg("armv7a-none-eabi")
        // Include Xilinx cross-compiler libc headers
        .clang_arg(&format!(
            "-I{}",
            libc_h_path.to_str().expect("path needs to be UTF-8")
        ))
        .clang_arg(&format!(
            "-I{}",
            stdint_h_path.to_str().expect("path needs to be UTF-8")
        ))
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // BSP includes
        .clang_arg("-I./include")
        // Use core instead of std to retain no_std compatibility
        .use_core()
        .ctypes_prefix("cty")
        // Do not generate tests, because I can't be bothered to set up #[test] in the build
        // environment of the cross-compiler
        .layout_tests(false)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/xil.rs file.
    let out_path = path::PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Copy the libxil_sf.a library into the target directory to make it available
    // to dependencies TODO: allow overriding the library via an environment
    // variable
    const LIB_NAME: &str = "libxil_sf.a";
    let from: path::PathBuf = [&env::var("CARGO_MANIFEST_DIR").unwrap(), LIB_NAME]
        .iter()
        .collect();
    let to: path::PathBuf = [&env::var("OUT_DIR").unwrap(), LIB_NAME].iter().collect();
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
