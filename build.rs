extern crate bindgen;

use std::env;
use std::path::PathBuf;
use bindgen::EnumVariation;

// Use https://github.com/rust-cuda/cuda-sys/blob/cuda-bindgen/cuda-bindgen/src/main.rs
// OR https://github.com/inducer/pycuda/blob/master/pycuda/compiler.py#L349
fn find_cuda() -> PathBuf {
    let cuda_env = env::var("CUDA_LIBRARY_PATH").ok().unwrap_or(String::from(""));
    let mut paths: Vec<PathBuf> = env::split_paths(&cuda_env).collect();
    paths.push(PathBuf::from("/usr/local/cuda"));
    paths.push(PathBuf::from("/opt/cuda"));
    for path in paths {
        if path.join("include/nvrtc.h").is_file() {
            return path;
        }
    }
    panic!("Cannot find CUDA NVRTC libraries");
}

pub fn read_env() -> Vec<PathBuf> {
    if let Ok(path) = env::var("CUDA_LIBRARY_PATH") {
        // The location of the libcuda, libcudart, and libcublas can be hardcoded with the
        // CUDA_LIBRARY_PATH environment variable.
        let split_char = if cfg!(target_os = "windows") {
            ";"
        } else {
            ":"
        };
        path.split(split_char).map(|s| PathBuf::from(s)).collect()
    } else {
        vec![]
    }
}

fn find_cuda_windows() -> PathBuf {
    let paths = read_env();
    if !paths.is_empty() {
        return paths[0].clone();
    }

    if let Ok(path) = env::var("CUDA_PATH") {
        // If CUDA_LIBRARY_PATH is not found, then CUDA_PATH will be used when building for
        // Windows to locate the Cuda installation. Cuda installs the full Cuda SDK for 64-bit,
        // but only a limited set of libraries for 32-bit. Namely, it does not include cublas in
        // 32-bit, which cuda-sys requires.

        // 'path' points to the base of the CUDA Installation. The lib directory is a
        // sub-directory.
        let path = PathBuf::from(path);

        // To do this the right way, we check to see which target we're building for.
        let target = env::var("TARGET")
            .expect("cargo did not set the TARGET environment variable as required.");

        // Targets use '-' separators. e.g. x86_64-pc-windows-msvc
        let target_components: Vec<_> = target.as_str().split("-").collect();

        // We check that we're building for Windows. This code assumes that the layout in
        // CUDA_PATH matches Windows.
        if target_components[2] != "windows" {
            panic!(
                "The CUDA_PATH variable is only used by cuda-sys on Windows. Your target is {}.",
                target
            );
        }

        // Sanity check that the second component of 'target' is "pc"
        debug_assert_eq!(
            "pc", target_components[1],
            "Expected a Windows target to have the second component be 'pc'. Target: {}",
            target
        );

        if path.join("include/nvrtc.h").is_file() {
            return path;
        }

    }

    // No idea where to look for CUDA
    panic!("Cannot find CUDA NVRTC libraries");
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let cuda_path;
    if cfg!(target_os = "windows") {
            cuda_path = find_cuda_windows()
        } else {
            cuda_path = find_cuda();
        };

    // let cuda_path = find_cuda();

    bindgen::builder()
        .header("nvrtc.h")
        .clang_arg(format!("-I{}/include", cuda_path.display()))
        .allowlist_recursively(false)
        .allowlist_type("^_?nvrtc.*")
        .allowlist_var("^_?nvrtc.*")
        .allowlist_function("^_?nvrtc.*")
        .derive_copy(false)
        .default_enum_style(EnumVariation::Rust { non_exhaustive: false })
        .generate()
        .expect("Unable to generate NVRTC bindings")
        .write_to_file(out_path.join("nvrtc_bindings.rs"))
        .expect("Unable to write NVRTC bindings");

    // Check for Windows
    if cfg!(target_os = "windows") {
        println!(
            "cargo:rustc-link-search=native={}\\lib\\x64",
            cuda_path.display()
        );
    } else {
   
        println!(
            "cargo:rustc-link-search=native={}/lib64",
            cuda_path.display()
        );
    }


    println!("cargo:rustc-link-lib=dylib=nvrtc");
    println!("cargo:rerun-if-changed=build.rs");
}