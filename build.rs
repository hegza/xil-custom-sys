use std::{fs, path, ffi, env, process};

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
const LIBC_H_RELATIVE_LOCATION: &str = "libc/usr/include/linux";
const STDINT_H_RELATIVE_LOCATION: &str = "libc/usr/include";

const LIBRARY_NAME: &str = "xil_sf";

static C_FLAGS: &[&str] = &[
   "-mcpu=cortex-a9",
   "-mfpu=vfpv3",
   "-mfloat-abi=soft",
   "-nostartfiles",
];

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

/// Get paths to compile
fn get_src_paths() -> Vec<path::PathBuf> {
    // How on earth you make a globally accessible Path in rust? Is it even possible?
    // I'll make a function that returns a constant pathbuf then
    let root_path = path::PathBuf::from("libsrc");
    let sub_paths = root_path.read_dir().expect("Unable to read libsrc directory").filter_map(|entry| {
        let entry = entry.expect("Unable to read file from directory");
        let path = entry.path();

        // Ignore files at root-level
        if path.is_file() {
            return None;
        }

        // All paths include this intermediary src/
        let path = path.join("src/");

        Some(path.clone())
    }).collect::<Vec<path::PathBuf>>();

    sub_paths
}

fn src_files(path: &path::PathBuf) -> Vec<path::PathBuf> {
    let ignore_files = vec![];

    let c_ext = Some(ffi::OsStr::new("c"));
    let asm_ext = Some(ffi::OsStr::new("S"));

    if path.is_file() {  // Single files can be compiled too, though idk why someone wants that
        let ext = path.extension();
        match ext {
            e if e==c_ext => {return vec![path.clone()];},
            _ => panic!("Invalid file extension on source file."),
        }
    }
    else if path.is_dir() {
        path.read_dir()
            .expect(&format!("Unable to read directory: {}", path.to_str().unwrap()))
            .filter_map(|entry| {
                let entry = entry.expect("Unable to read a file from directory");

                let path = entry.path();

                // Ignore directories
                if path.is_dir() {
                    return None;
                }

                // Ignore Files
                if ignore_files.contains(&path.file_name()) {
                    return None;
                }

                // We only care about .c and .S
                let ext = path.extension();
                if ext == c_ext || ext == asm_ext {
                    Some(path.clone())
                } else {
                    None
                }
            }).collect::<Vec<path::PathBuf>>()
    }
    else {panic!("Uh oh")}
}

fn compile() {
    let mut c_files = Vec::new();
    let mut builder = cc::Build::new();

    // Add the root include directory
    builder.include("include/");

    for path in get_src_paths().iter() {
        let c = src_files(path);
        c_files.extend(c);
        builder.include(&path);
    }

    builder.archiver("arm-none-eabi-ar")
    .pic(false)
    .warnings(false);


    for flag in C_FLAGS {
        builder.flag(flag);
    }

    // Compile C Files
    builder.compiler("arm-none-eabi-gcc")
        .files(c_files)
        .opt_level_str("2")
        .flag("-c")
        .compile(LIBRARY_NAME);
}

fn generate_bindings() {
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

}

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes.
    println!("cargo:rerun-if-changed=wrapper.h");

    // Generate bindings.rs
    generate_bindings();

    // Compile libxil.a
    compile();
}
