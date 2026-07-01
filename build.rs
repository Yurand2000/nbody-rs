use std::process::Command;

fn main() {
    if std::env::var_os("CARGO_FEATURE_CUDA").is_some() {
        compile_cuda_kernels();
    }
}

fn compile_cuda_kernels() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_path = format!("{}/libkernels.a", out_dir);

    let input_dir = std::path::Path::new("cuda-kernels");

    let obj_files =
        std::fs::read_dir(input_dir).expect("read_dir failed")
            .map(|entry| entry.expect("dir entry failed").path())
            .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("cu"))
            .map(|path| {
                let stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .expect("bad filename");

                let in_file = path.to_str().expect("bad path");
                let out_file = format!("{}/{}.o", out_dir, stem);

                let status = Command::new("nvcc")
                    .args([
                        "-c",
                        &in_file,
                        "-o",
                        &out_file,
                        "--compiler-options=-fPIC",
                    ])
                    .status()
                    .expect("failed to run nvcc");

                if !status.success() {
                    panic!("nvcc failed for {}", path.display());
                }

                out_file
            })
            .collect::<Vec<_>>();

    // Pack into a static library
    let status = Command::new("ar")
        .args(
            ["crus", &lib_path].into_iter()
                .chain(obj_files.iter().map(|file| file.as_str()))
        )
        .status()
        .expect("failed to run ar");

    if !status.success() {
        panic!("ar failed");
    }

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-search=native=/opt/cuda/lib64");
    println!("cargo:rustc-link-lib=static=kernels");
    println!("cargo:rustc-link-lib=dylib=cudart");
    println!("cargo:rustc-link-lib=dylib=stdc++");
}
