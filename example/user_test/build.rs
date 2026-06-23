use build_vdso::*;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../vdso_example");
    println!("cargo:rerun-if-changed=../build_vdso");

    let arch = option_env!("ARCH");

    let mut config = BuildConfig::new("../vdso_example", "vdso_example");
    if let Some(arch) = arch {
        config.arch = String::from(arch);
    } else {
        config.arch = String::from("riscv64");
    }
    config.so_name = String::from("libvdsoexample");
    config.api_lib_name = String::from("libvdsoexample");
    config.out_dir = String::from("../../output");
    // config.toolchain = String::from("nightly-2025-09-30");
    config.verbose = 2;
    config.log = true;
    build_vdso(&config);
}
