extern crate bindgen;

use std::env;
use std::path::PathBuf;

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

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cuda_path = find_cuda();

    bindgen::builder()
        .header("nvrtc.h")
        .clang_arg(format!("-I{}/include", cuda_path.display()))
        .whitelist_recursively(false)
        .whitelist_type("^_?nvrtc.*")
        .whitelist_var("^_?nvrtc.*")
        .whitelist_function("^_?nvrtc.*")
        .derive_copy(false)
        .default_enum_style(bindgen::EnumVariation::Rust)
        .generate()
        .expect("Unable to generate NVRTC bindings")
        .write_to_file(out_path.join("nvrtc_bindings.rs"))
        .expect("Unable to write NVRTC bindings");

    println!(
        "cargo:rustc-link-search=native={}/lib64",
        cuda_path.display()
    );
    println!("cargo:rustc-link-lib=dylib=nvrtc");
    println!("cargo:rerun-if-changed=build.rs");
}