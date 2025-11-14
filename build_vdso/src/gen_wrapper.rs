use std::{fs, path::Path};

use crate::BuildConfig;

pub(crate) fn gen_wrapper(config: &BuildConfig) {
    let lib_path = Path::new(&config.out_dir).join("vdso_wrapper");
    let src_path = lib_path.join("src");
    fs::create_dir_all(&src_path).unwrap();
    let cargo_toml = cargo_toml_content(config);
    let lib_rs = lib_rs_content(config);

    fs::write(&lib_path.join("Cargo.toml"), cargo_toml).unwrap();
    fs::write(&src_path.join("lib.rs"), lib_rs).unwrap();
}

fn cargo_toml_content(config: &BuildConfig) -> String {
    let absolute_src_dir = fs::canonicalize(Path::new(&config.src_dir)).unwrap();
    format!(
        r#"[package]
name = "vdso_wrapper"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"

[dependencies]
{} = {{ path = "{}" }}
"#,
        config.package_name,
        absolute_src_dir.display()
    )
}

fn lib_rs_content(config: &BuildConfig) -> String {
    format!(
        r#"#![no_std]

pub use {}::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {{
    loop {{}}
}}

"#,
        config.package_name,
    )
}
