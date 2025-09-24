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

    // 生成wrapper cdylib
    gen_wrapper(config);

    build_so(config);

    gen_api(config);
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
    let linker = format!(
        r#"
    OUTPUT_ARCH({})

    SECTIONS {{
        . = SIZEOF_HEADERS;

        /* 先放置动态链接相关的只读段 */
        .hash		: {{ *(.hash) }}
    	.gnu.hash	: {{ *(.gnu.hash) }}
    	.dynsym		: {{ *(.dynsym) }}
    	.dynstr		: {{ *(.dynstr) }}
    	.gnu.version	: {{ *(.gnu.version) }}
    	.gnu.version_d	: {{ *(.gnu.version_d) }}
    	.gnu.version_r	: {{ *(.gnu.version_r) }}

        /* 动态段单独分配 */
        .dynamic    : {{ *(.dynamic) }}

        . = ALIGN(16);
        /* 代码段（.text）需要放在只读数据段之前 */
        .text       : {{
            *(.text.start)
            *(.text .text.*)
        }}

        . = ALIGN(4K);
        /* 只读数据段（.rodata等） */
        .rodata     : {{
            *(.rodata .rodata.* .gnu.linkonce.r.*)
            *(.note.*)
        }}

        . = ALIGN(4K);
        .plt : {{ *(.plt .plt.*) }}

        . = ALIGN(4K);
        /* 数据段（.data、.bss等）单独分配 */
        .data       : {{
            *(.data .data.* .gnu.linkonce.d.*)
            *(.got.plt) *(.got)
        }}

        . = ALIGN(4K);
        .bss        : {{
            *(.bss .bss.* .gnu.linkonce.b.*)
            *(COMMON)
        }}

        .eh_frame_hdr	: {{ *(.eh_frame_hdr) }}
    	.eh_frame	: {{ KEEP (*(.eh_frame)) }}
    }}
    "#,
        arch_lds
    );
    linker
}

/// 编译vdso库为so文件，并拷贝到输出目录
fn build_so(config: &BuildConfig) {
    let absolute_script_dir = fs::canonicalize(Path::new(&config.out_dir).join("vdso_linker.lds"))
        .unwrap()
        .display()
        .to_string();
    let rustflags = format!(
        "-C link-arg=-fpie -C link-arg=-soname={} -C link-arg=-T{}",
        &config.so_name, absolute_script_dir
    ); // 由于cargo在另外的目录执行，因此需要传入绝对路径
    println!("RUSTFLAGS: {}", rustflags);
    let build_target = match config.arch.as_str() {
        "x86_64" => "x86_64-unknown-linux-musl",
        "aarch64" => "aarch64-unknown-linux-musl",
        "riscv64" => "riscv64gc-unknown-linux-musl",
        _ => panic!("Unsupported arch"),
    };
    let build_mode = match config.mode.as_str() {
        "debug" => "",
        "release" => "--release",
        _ => panic!("Unsupported mode"),
    };
    let build_verbose = match config.verbose {
        0 => "",
        1 => "-v",
        2 => "-vv",
        _ => panic!("Unsupported verbose level"),
    };
    fs::create_dir_all(Path::new(&config.out_dir).join("target")).unwrap();
    let absolute_build_target_dir = fs::canonicalize(Path::new(&config.out_dir).join("target"))
        .unwrap()
        .display()
        .to_string();
    let mut cargo_args = vec![
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
    if build_mode != "" {
        cargo_args.push(build_mode);
    }
    if build_verbose != "" {
        cargo_args.push(build_verbose);
    }
    let mut cargo = Command::new("cargo");
    // 如果启用了crt-static特性，则在vdso的编译中去掉该特性，否则会报错
    if let Ok(value) = env::var("CARGO_CFG_TARGET_FEATURE") {
        if value.contains("crt-static") {
            let mut vdso_value = value.replace(",crt-static", "");
            if vdso_value == value {
                vdso_value = value.replace("crt-static,", "")
            }
            if vdso_value == value {
                // 说明该变量只指定了crt-static一项
                cargo.env_remove("CARGO_CFG_TARGET_FEATURE");
            } else {
                cargo.env("CARGO_CFG_TARGET_FEATURE", vdso_value);
            }
        }
    }
    if let Ok(value) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        if value.contains("+crt-static") {
            let vdso_value = value.replace("+crt-static", "-crt-static");
            cargo.env("CARGO_ENCODED_RUSTFLAGS", vdso_value);
        }
    }
    let wrapper_dir = Path::new(&config.out_dir).join("vdso_wrapper");
    cargo
        .current_dir(&wrapper_dir)
        .env("ARCH", &config.arch)
        .env("RUSTFLAGS", rustflags)
        .args(cargo_args);
    println!("----------------cargo command----------------");
    println!("{:?}", &cargo);
    let cargo_output = cargo.output().expect("Failed to execute cargo build");
    println!("-----------------cargo stdout----------------");
    stdout().write_all(&cargo_output.stdout).unwrap();
    println!("-----------------cargo stderr----------------");
    stderr().write_all(&cargo_output.stderr).unwrap();
    if !cargo_output.status.success() {
        panic!("cargo build failed");
    }

    let mut objcopy = Command::new("rust-objcopy");
    // let src_filename = String::from("lib") + &config.package_name;
    let src_file = Path::new(&absolute_build_target_dir)
        .join(build_target)
        .join(&config.mode)
        .join("libvdso_wrapper")
        .with_extension("so")
        .display()
        .to_string();
    let dst_file = Path::new(&config.out_dir)
        .join(&config.so_name)
        .with_extension("so")
        .display()
        .to_string();
    objcopy.args(["-X", &src_file, &dst_file]);
    println!("---------------objcopy command---------------");
    println!("{:?}", &objcopy);
    let objcopy_output = objcopy.output().expect("Failed to execute rust-objcopy");
    println!("----------------objcopy stdout---------------");
    stdout().write_all(&objcopy_output.stdout).unwrap();
    println!("----------------objcopy stderr---------------");
    stderr().write_all(&objcopy_output.stderr).unwrap();
    if !objcopy_output.status.success() {
        panic!("objcopy failed");
    }
}
