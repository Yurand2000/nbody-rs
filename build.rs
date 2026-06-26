use std::process::Command;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_path = format!("{}/libkernels.a", out_dir);

    // Adjust paths/flags as needed for your environment.
    let status = Command::new("nvcc")
        .args([
            "-c",
            "cuda-kernels/example-kernel.cu",
            "-o",
            &format!("{}/example-kernel.o", out_dir),
            "--compiler-options=-fPIC",
        ])
        .status()
        .expect("failed to run nvcc");
    assert!(status.success());

    // Pack into a static library
    let status = Command::new("ar")
        .args(["crus", &lib_path, &format!("{}/example-kernel.o", out_dir)])
        .status()
        .expect("failed to run ar");
    assert!(status.success());

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-search=native=/opt/cuda/lib64");
    println!("cargo:rustc-link-lib=static=kernels");
    println!("cargo:rustc-link-lib=dylib=cudart");
    println!("cargo:rustc-link-lib=dylib=stdc++");
}
