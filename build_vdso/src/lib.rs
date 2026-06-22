//! 本模块用于构建vDSO库。
//!
//! 在vDSO外部代码的build.rs中调用[`build_vdso`]函数，传入[`BuildConfig`]配置结构体，即可完成vDSO库的构建。
//!
//! vDSO库的构建产物包括so文件和API库，vDSO外部代码在正确加载so文件与vVAR数据区后，将so文件的加载地址传入API库中，
//! 即可通过API库调用vDSO中的函数。

#![deny(missing_docs)]

use std::{
    env, fs,
    io::{stderr, stdout, Write},
    path::Path,
    process::Command,
};

pub mod build_config;
pub use build_config::*;

mod gen_api;
use gen_api::gen_api;

mod gen_wrapper;
use gen_wrapper::gen_wrapper;

/// 构建vdso的代码。在vdso外部代码的build.rs中调用该函数。
///
/// # 参数
///
/// - `config`: vdso构建配置结构体，详见[`BuildConfig`]。
pub fn build_vdso(config: &BuildConfig) {
    // // 用于打印环境变量的测试代码。
    // // 如果build_vdso执行失败，可能是主编译单元设置了某些环境变量，并被继承到了vdso的编译中。
    // let env = Command::new("env").output().unwrap();
    // println!("env:");
    // stdout().write_all(&env.stdout).unwrap();
    // panic!("aaa");

    // 创建输出目录
    fs::create_dir_all(&config.out_dir).unwrap();

    // 生成链接脚本
    let out_path = Path::new(&config.out_dir).join("vdso_linker.lds");
    let linker_script = gen_linker_script(&config.arch);
    fs::write(&out_path, &linker_script).unwrap();

    // 生成wrapper静态库
    gen_wrapper(config);

    build_so(config);

    gen_api(config);
}

// 选择编译目标三元组
fn build_target(arch: &str) -> &'static str {
    match arch {
        "x86_64" => "x86_64-unknown-none",
        "aarch64" => "aarch64-unknown-none",
        "riscv64" => "riscv64gc-unknown-none-elf",
        _ => panic!("Unsupported arch"),
    }
}

// 选择链接器程序
fn linker_program(arch: &str) -> &'static str {
    match arch {
        "x86_64" => "x86_64-linux-musl-ld",
        "aarch64" => "aarch64-linux-musl-ld",
        "riscv64" => "riscv64-linux-musl-ld",
        _ => panic!("Unsupported arch"),
    }
}

/// 生成链接脚本的代码
fn gen_linker_script(arch: &str) -> String {
    // Copied and modified from https://github.com/AsyncModules/vsched/blob/e19b572714a6931972f1428e42d43cc34bcf47f2/vsched/build.rs
    let arch_lds = match arch {
        "riscv64" => "riscv",
        "aarch64" => "aarch64",
        "x86_64" => "i386:x86-64",
        _ => panic!("Unsupported arch"),
    };
    let linker_template = include_str!("link.ld");
    // let linker_template = include_str!("link_no_segment.ld");
    let linker = linker_template.replace("{output_arch}", arch_lds);
    linker
}

/// 先编译为静态库，再单独链接成 so。
fn build_so(config: &BuildConfig) {
    // 获取输出目录和生成链接脚本路径
    let out_dir = Path::new(&config.out_dir);
    let absolute_script_dir = fs::canonicalize(out_dir.join("vdso_linker.lds"))
        .unwrap()
        .display()
        .to_string();
    // 生成版本脚本
    let version_script_path = out_dir.join("vdso_version.map");
    fs::write(&version_script_path, version_script_content(config)).unwrap();

    // 获取编译目标和链接器程序
    let build_target = build_target(&config.arch);
    let linker = linker_program(&config.arch);
    // 获取是否为release模式
    let build_mode = match config.mode.as_str() {
        "debug" => "",
        "release" => "--release",
        _ => panic!("Unsupported mode"),
    };
    // 获取编译输出的冗长程度
    let build_verbose = match config.verbose {
        0 => "",
        1 => "-v",
        2 => "-vv",
        _ => panic!("Unsupported verbose level"),
    };
    // 获取.a输出目录
    fs::create_dir_all(out_dir.join("target")).unwrap();
    let absolute_build_target_dir = fs::canonicalize(out_dir.join("target"))
        .unwrap()
        .display()
        .to_string();
    // 三元组参数
    let toolchain_arg = format!("+{}", &config.toolchain);

    let mut cargo_args = vec![
        &toolchain_arg,
        "build",
        "-Z",
        "unstable-options",
        "-Z",
        "build-std=core,compiler_builtins,alloc",
        "-Z",
        "build-std-features=compiler-builtins-mem",
        "--target",
        build_target,
        "--target-dir",
        &absolute_build_target_dir,
    ];
    // // features
    // let features_arg = config.features.join(",");
    // if !config.features.is_empty() {
    //     cargo_args.push("--features");
    //     cargo_args.push(&features_arg);
    // }
    if build_mode != "" {
        cargo_args.push(build_mode);
    }
    if build_verbose != "" {
        cargo_args.push(build_verbose);
    }
    let mut cargo = Command::new("cargo");

    // 添加环境变量，过滤掉以CARGO或RUST开头的环境变量
    cargo.env_clear();
    for (key, value) in env::vars() {
        if !(key.starts_with("CARGO") || key.starts_with("RUST")) {
            cargo.env(key, value);
        }
    }

    // wrappper输出目录
    let wrapper_dir = out_dir.join("vdso_wrapper");
    // 构建编译命令
    cargo
        .current_dir(&wrapper_dir)
        .env("ARCH", &config.arch)
        .env("RUSTFLAGS", "-C force-frame-pointers=yes")
        .args(cargo_args);
    println!("----------------cargo command----------------");
    println!("{:?}", &cargo);
    // output()会触发执行命令并等待完成
    let cargo_output = cargo.output().expect("Failed to execute cargo build");
    println!("-----------------cargo stdout----------------");
    stdout().write_all(&cargo_output.stdout).unwrap();
    println!("-----------------cargo stderr----------------");
    stderr().write_all(&cargo_output.stderr).unwrap();
    if !cargo_output.status.success() {
        panic!("cargo build failed");
    }

    // 获取.a路径
    let src_file = Path::new(&absolute_build_target_dir)
        .join(build_target)
        .join(&config.mode)
        .join("libvdso_wrapper")
        .with_extension("a")
        .display()
        .to_string();
    // 目标so路径
    let dst_file = Path::new(&config.out_dir)
        .join(&config.so_name)
        .with_extension("so")
        .display()
        .to_string();
    let mut linker_cmd = Command::new(linker);
    // 链接命令参数
    linker_cmd.args([
        "-shared",
        "-soname",
        &config.so_name,
        "-T",
        &absolute_script_dir,
        "--version-script",
        version_script_path
            .to_str()
            .expect("version script 不是有效 UTF-8"),
        "--gc-sections",
        "--whole-archive",
        &src_file,
        "--no-whole-archive",
        "-o",
        &dst_file,
    ]);
    println!("---------------linker command---------------");
    println!("{:?}", &linker_cmd);
    let linker_output = linker_cmd.output().expect("Failed to execute vDSO linker");
    println!("----------------linker stdout----------------");
    stdout().write_all(&linker_output.stdout).unwrap();
    println!("----------------linker stderr----------------");
    stderr().write_all(&linker_output.stderr).unwrap();
    if !linker_output.status.success() {
        panic!("linker failed");
    }
}

fn version_script_content(config: &BuildConfig) -> String {
    let mut symbols = exported_symbols(config);
    symbols.sort();
    symbols.dedup();

    let mut content = String::from("vdso {\n    global:\n");
    for symbol in symbols {
        content.push_str("        ");
        content.push_str(&symbol);
        content.push_str(";\n");
    }
    content.push_str("    local:\n        *;\n};\n");
    content
}

fn exported_symbols(config: &BuildConfig) -> Vec<String> {
    let mut symbols: Vec<String> = Vec::new();
    symbols.push("panic_loop".into());

    let api_rs_path = Path::new(&config.src_dir).join("src").join("api.rs");
    if let Ok(api_source) = fs::read_to_string(&api_rs_path) {
        let re = regex::Regex::new(
            r#"(?s)#\[unsafe\(no_mangle\)\]\s*pub\s+extern\s+\"C\"\s+fn\s+([A-Za-z0-9_]+)\s*\("#,
        )
        .unwrap();

        let mut api_symbols: Vec<String> = re
            .captures_iter(&api_source)
            .map(|capture| capture[1].to_string())
            .collect();
        symbols.append(&mut api_symbols);

        let re = regex::Regex::new(r#"extern \"C\" \{([^\{\}]+)\}"#).unwrap();
        for (_, [extern_fns]) in re.captures_iter(&api_source).map(|c| c.extract()) {
            let fns_re = regex::Regex::new(r#"fn ([a-zA-Z0-9_]+)\(\) -> !;"#).unwrap();
            for (_, [name]) in fns_re.captures_iter(&extern_fns).map(|c| c.extract()) {
                // println!("name: {}", name);
                symbols.push(name.into());
            }
        }
    }

    let interface_rs_path = Path::new(&config.src_dir).join("src").join("interface.rs");
    if let Ok(interface_source) = fs::read_to_string(&interface_rs_path) {
        let re = regex::Regex::new(r#"pub trait ([a-zA-Z0-9_]+) \{([^\{\}]+)\}"#).unwrap();

        let mut interface_symbols: Vec<String> = re
            .captures_iter(&interface_source)
            .map(|capture| format!("init_vtable_{}", capture.extract::<2>().1[0]))
            .collect();
        symbols.append(&mut interface_symbols);
    }

    symbols
}
