// Copied and modified from https://github.com/AsyncModules/vsched/blob/e19b572714a6931972f1428e42d43cc34bcf47f2/vsched_apis/build.rs
use include_bytes_aligned::include_bytes_aligned;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use xmas_elf::symbol_table::Entry;

const VDSO_API_PATH: &str = "../vdso/src/api.rs";
static SO_CONTENT: &[u8] = include_bytes_aligned!(8, "../libvdsoexample.so");

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("api.rs");
    println!("cargo:rerun-if-changed={}", VDSO_API_PATH);
    build_vsched_api(out_path);
}

fn build_vsched_api(out_path: PathBuf) {
    let vsched_api_file_content = fs::read_to_string(VDSO_API_PATH).unwrap();
    println!("cargo:warning=so_len={}", SO_CONTENT.len());
    println!(
        "cargo:warning=so_start=0x{:x}",
        SO_CONTENT.as_ptr() as usize
    );
    let vdso_elf = xmas_elf::ElfFile::new(SO_CONTENT).expect("Error parsing app ELF file.");

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
    // vdso_vtable 数据结构定义
    let mut vdso_vtable_struct_str = "\nstruct VdsoVTable {\n".to_string();
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

    // 生成最终的 api.rs 文件
    let api_out_path = &out_path;
    std::fs::remove_file(api_out_path).unwrap_or(());
    let mut api_file_content = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(api_out_path)
        .unwrap();
    api_file_content.write_all(VDSO_SECTION.as_bytes()).unwrap();

    api_file_content
        .write_all(vdso_vtable_struct_str.as_bytes())
        .unwrap();

    api_file_content
        .write_all(static_vdso_vtable_str.as_bytes())
        .unwrap();

    api_file_content
        .write_all(fn_init_vdso_vtable_str.as_bytes())
        .unwrap();

    for api in apis.iter() {
        api_file_content.write_all(api.as_bytes()).unwrap();
    }
}

const INIT_VDSO_VTABLE_STR: &str = r#"
pub unsafe fn init_vdso_vtable(base: u64) {
"#;

const VDSO_SECTION: &str = r#"pub use structs::argument::*;
"#;
