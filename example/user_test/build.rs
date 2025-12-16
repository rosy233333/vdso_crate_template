use build_vdso::*;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../vdso_example");
    println!("cargo:rerun-if-changed=../build_vdso");

    let mut config = BuildConfig::new("../vdso_example", "vdso_example");
    config.so_name = String::from("libvdsoexample");
    config.api_lib_name = String::from("libvdsoexample");
    config.out_dir = String::from("../../output");
    config.toolchain = String::from("nightly-2025-09-12");
    config.verbose = 2;
    build_vdso(&config);
}
