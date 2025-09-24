use std::{fmt::format, fs, path::Path};

use xmas_elf::symbol_table::Entry;

use crate::BuildConfig;

/// 在输出路径中创建一个Rust项目“api”，用于：
/// - 向调用者提供so文件和vvar数据结构的定义，用于调用者初始化vdso。
/// - 向调用者提供调用vdso的接口定义。
pub(crate) fn gen_api(config: &BuildConfig) {
    let lib_path = Path::new(&config.out_dir).join(&config.api_lib_name);
    let src_path = lib_path.join("src");
    fs::create_dir_all(&src_path).unwrap();
    let cargo_toml = cargo_toml_content(config);
    let lib_rs = lib_rs_content(config);
    let api_rs = api_rs_content(config);

    fs::write(&lib_path.join("Cargo.toml"), cargo_toml).unwrap();
    fs::write(&src_path.join("lib.rs"), lib_rs).unwrap();
    fs::write(&src_path.join("api.rs"), api_rs).unwrap();
}

fn cargo_toml_content(config: &BuildConfig) -> String {
    let absolute_src_dir = fs::canonicalize(Path::new(&config.src_dir)).unwrap();
    format!(
        r#"[package]
name = "{}"
edition = "2021"

[dependencies]
{} = {{ path = "{}" }}
log = {{ version = "0.4", optional = true }}

[features]
log = ["dep:log"]
default = []
"#,
        config.api_lib_name,
        config.package_name,
        absolute_src_dir.display()
    )
}

fn lib_rs_content(_config: &BuildConfig) -> String {
    String::from(
        r#"#![no_std]
pub mod api;
pub use api::*;
"#,
    )
}

fn api_rs_content(config: &BuildConfig) -> String {
    // 修改自https://github.com/AsyncModules/vsched/blob/e728dadd75aeb8da5cec1642320a6bd24af5b5bb/vsched_apis/build.rs的build_vsched_api函数
    let api_rs_path = Path::new(&config.src_dir)
        .join("src")
        .join("api")
        .with_extension("rs");
    // println!("api.rs path: {}", api_rs_path.display());
    let vsched_api_file_content = fs::read_to_string(&api_rs_path).unwrap();
    let elf_path = Path::new(&config.out_dir).join(format!("{}.so", config.so_name));
    let so_content = fs::read(&elf_path).unwrap();
    let vdso_elf = xmas_elf::ElfFile::new(&so_content).expect("Error parsing app ELF file.");

    let re = regex::Regex::new(
        r#"#\[unsafe\(no_mangle\)\]\npub extern \"C\" fn ([a-zA-Z0-9_]?.*)(\([a-zA-Z0-9_:]?.*\)[->]?.*) \{"#,
    )
    .unwrap();
    // 获取共享调度器的 api
    let mut fns = vec![];
    for (_, [name, args]) in re
        .captures_iter(&vsched_api_file_content)
        .map(|c| c.extract())
    {
        // println!("{}: {}", name, args);
        fns.push((name, args));
    }
    // pub use vdso库中的内容
    let pub_use_vdso_str = format!(
        "extern crate {};\npub use self::{}::*;\n\n",
        config.package_name, config.package_name
    );
    // vdso_vtable 数据结构定义
    let mut vdso_vtable_struct_str = "struct VdsoVTable {\n".to_string();
    for (name, args) in fns.iter() {
        vdso_vtable_struct_str.push_str(&format!("    pub {}: Option<fn{}>,\n", name, args));
    }
    vdso_vtable_struct_str.push_str("}\n");

    // 定义静态的 VDSO_VTABLE
    let mut static_vdso_vtable_str =
        "\nstatic mut VDSO_VTABLE: VdsoVTable = VdsoVTable {\n".to_string();
    for (name, _) in fns.iter() {
        static_vdso_vtable_str.push_str(&format!("    {}: None,\n", name));
    }
    static_vdso_vtable_str.push_str("};\n");

    // 运行时初始化 vsched_table 的函数
    let dyn_sym_table = vdso_elf.find_section_by_name(".dynsym").unwrap();
    let dyn_sym_table = match dyn_sym_table.get_data(&vdso_elf) {
        Ok(xmas_elf::sections::SectionData::DynSymbolTable64(dyn_sym_table)) => dyn_sym_table,
        _ => panic!("Invalid data in .dynsym section"),
    };
    let mut fn_init_vdso_vtable_str = INIT_VDSO_VTABLE_STR.to_string();
    for (name, args) in fns.iter() {
        let mut sym_value: usize = 0;
        for dynsym in dyn_sym_table {
            let sym_name = dynsym.get_name(&vdso_elf).unwrap();
            if sym_name == *name {
                sym_value = dynsym.value() as usize;
                break;
            }
        }
        assert!(sym_value != 0, "Function {} not found in .dynsym", name);

        fn_init_vdso_vtable_str.push_str(&format!(
            r#"    // {}:
    let fn_ptr = base + 0x{:x};
    #[cfg(feature = "log")]
    log::debug!("{}: 0x{{:x}}", fn_ptr);
    let f: fn{} = unsafe {{ core::mem::transmute(fn_ptr) }};
    unsafe {{ VDSO_VTABLE.{}  = Some(f); }}

"#,
            name, sym_value, name, args, name
        ));
    }
    fn_init_vdso_vtable_str.push_str(
        r#"}
    "#,
    );

    // 构建给内核和用户运行时使用的接口
    let mut apis = vec![];
    for (name, args) in fns.iter() {
        let re = regex::Regex::new(r#"\(([a-zA-Z0-9_:]?.*)\)"#).unwrap();
        let mut fn_args = String::new();
        for (_, [ident_ty]) in re.captures_iter(args).map(|c| c.extract()) {
            // println!("{}: {}", name, args);
            let ident_str: Vec<&str> = ident_ty
                .split(",")
                .map(|s| {
                    let idx = s.find(":");
                    if let Some(idx) = idx {
                        let ident = s[..idx].trim();
                        ident
                    } else {
                        ""
                    }
                })
                .collect();
            for ident in ident_str.iter() {
                if ident.len() > 0 {
                    fn_args.push_str(&format!("{}, ", ident));
                }
            }
            fn_args = fn_args.trim_end_matches(", ").to_string();
            // println!("{:?}", fn_args);
        }

        apis.push(format!(
            r#"
pub fn {}{} {{
    if let Some(f) = unsafe {{ VDSO_VTABLE.{} }} {{
        #[cfg(feature = "log")]
        log::debug!("Calling {} at 0x{{:x}}.", f as *const () as usize);
        f({})
    }} else {{
        panic!("{} is not initialized")
    }}
}}
"#,
            name, args, name, name, fn_args, name
        ));
    }
    // println!("apis: {:?}", apis);

    let mut api_content = String::new();
    api_content.push_str(&pub_use_vdso_str);
    api_content.push_str(&vdso_vtable_struct_str);
    api_content.push_str(&static_vdso_vtable_str);
    api_content.push_str(&fn_init_vdso_vtable_str);

    for api in apis.iter() {
        api_content.push_str(api);
    }

    api_content
}

const INIT_VDSO_VTABLE_STR: &str = r#"
pub unsafe fn init_vdso_vtable(base: u64) {
"#;
