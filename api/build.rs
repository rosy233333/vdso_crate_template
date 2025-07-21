// Copied and modified from https://github.com/AsyncModules/vsched/blob/e728dadd75aeb8da5cec1642320a6bd24af5b5bb/vsched/src/lib.rs
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const VSCHED_API_PATH: &str = "../vsched/src/api.rs";

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("api.rs");
    println!("cargo:rerun-if-changed={}", VSCHED_API_PATH);
    build_vsched_api(out_path);
}

fn build_vsched_api(out_path: PathBuf) {
    let vsched_api_file_content = fs::read_to_string(VSCHED_API_PATH).unwrap();
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
    // vsched_vtable 数据结构定义
    let mut vsched_vtable_struct_str = "\nstruct VschedVTable {\n".to_string();
    for (name, args) in fns.iter() {
        vsched_vtable_struct_str.push_str(&format!("    pub {}: Option<fn{}>,\n", name, args));
    }
    vsched_vtable_struct_str.push_str("}\n");
    // println!("vsched_vtable_str: {}", vsched_vtable_struct_str);

    // 定义静态的 VSCHED_VTABLE
    let mut static_vsched_vtable_str =
        "\nstatic mut VSCHED_VTABLE: VschedVTable = VschedVTable {\n".to_string();
    for (name, _) in fns.iter() {
        static_vsched_vtable_str.push_str(&format!("    {}: None,\n", name));
    }
    static_vsched_vtable_str.push_str("};\n");

    // 运行时初始化 vsched_table 的函数
    let mut fn_init_vsched_vtable_str = INIT_VSCHED_VTABLE_STR.to_string();
    for (name, args) in fns.iter() {
        fn_init_vsched_vtable_str.push_str(&format!(
            r#"            if name == "{}" {{
                let fn_ptr = base + dynsym.value();
                log::debug!("{{}}: {{:x}}", name, fn_ptr);
                let f: fn{} = unsafe {{ core::mem::transmute(fn_ptr) }};
                unsafe {{ VSCHED_VTABLE.{}  = Some(f); }}
            }}
"#,
            name, args, name
        ));
    }
    fn_init_vsched_vtable_str.push_str(
        r#"        }
    }
}
    "#,
    );
    // println!("fn_init_vsched_vtable_str: {}", fn_init_vsched_vtable_str);

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
    if let Some(f) = unsafe {{ VSCHED_VTABLE.{} }} {{
        f({})
    }} else {{
        panic!("{} is not initialized")
    }}
}}
"#,
            name, args, name, fn_args, name
        ));
    }
    // println!("apis: {:?}", apis);

    // 生成最终的 api.rs 文件
    let api_out_path = &out_path;
    std::fs::remove_file(api_out_path).unwrap_or(());
    let mut api_file_content = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(api_out_path)
        .unwrap();
    api_file_content
        .write_all(VSCHED_SECTION.as_bytes())
        .unwrap();

    api_file_content
        .write_all(vsched_vtable_struct_str.as_bytes())
        .unwrap();

    api_file_content
        .write_all(static_vsched_vtable_str.as_bytes())
        .unwrap();

    api_file_content
        .write_all(fn_init_vsched_vtable_str.as_bytes())
        .unwrap();

    for api in apis.iter() {
        api_file_content.write_all(api.as_bytes()).unwrap();
    }
}

const INIT_VSCHED_VTABLE_STR: &str = r#"
pub unsafe fn init_vsched_vtable(base: u64, vsched_elf: &ElfFile) {
    if let Some(dyn_sym_table) = vsched_elf.find_section_by_name(".dynsym") {
        let dyn_sym_table = match dyn_sym_table.get_data(&vsched_elf) {
            Ok(xmas_elf::sections::SectionData::DynSymbolTable64(dyn_sym_table)) => dyn_sym_table,
            _ => panic!("Invalid data in .dynsym section"),
        };
        for dynsym in dyn_sym_table {
            let name = dynsym.get_name(&vsched_elf).unwrap();
"#;

const VSCHED_SECTION: &str = r#"/// 这里的与 Vsched 相关的实现可以在 build 脚本中来自动化构建，而不是手动构建出来

use base_task::*;
use xmas_elf::symbol_table::Entry;
use xmas_elf::ElfFile;
"#;
