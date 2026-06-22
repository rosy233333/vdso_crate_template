use std::{fs, path::Path};

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
    let loader_rs = loader_rs_content(config);

    fs::write(&lib_path.join("Cargo.toml"), cargo_toml).unwrap();
    fs::write(&src_path.join("lib.rs"), lib_rs).unwrap();
    fs::write(&src_path.join("api.rs"), api_rs).unwrap();
    fs::write(&src_path.join("loader.rs"), loader_rs).unwrap();
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
crate_interface = "0.2"
page_table_entry = "0.5.7"
include_bytes_aligned = "0.1.4"
xmas-elf = "0.9.0"
elf_parser = {{ git = "https://github.com/rosy233333/elf_parser.git" }}
lazyinit = "0.2"

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
pub mod loader;
pub use loader::*;

extern crate alloc;
"#,
    )
}

fn api_rs_content(config: &BuildConfig) -> String {
    // 修改自https://github.com/AsyncModules/vsched/blob/e728dadd75aeb8da5cec1642320a6bd24af5b5bb/vsched_apis/build.rs的build_vsched_api函数

    let elf_path = Path::new(&config.out_dir).join(format!("{}.so", config.so_name));
    let so_content = fs::read(&elf_path).unwrap();
    let vdso_elf = xmas_elf::ElfFile::new(&so_content).expect("Error parsing app ELF file.");

    // 获取vDSO的 api
    let api_rs_path = Path::new(&config.src_dir)
        .join("src")
        .join("api")
        .with_extension("rs");
    // println!("api.rs path: {}", api_rs_path.display());
    let mut fns = vec![];
    if let Ok(mut vsched_api_file_content) = fs::read_to_string(&api_rs_path) {
        vsched_api_file_content = vsched_api_file_content
            .split('\n')
            .filter(|s| !(*s).trim().starts_with("//"))
            .collect();
        vsched_api_file_content = vsched_api_file_content.split('\t').collect();
        vsched_api_file_content = vsched_api_file_content.split("    ").collect();
        // println!("vsched_api_file_content: {}", vsched_api_file_content);

        let re = regex::Regex::new(
        r#"#\[unsafe\(no_mangle\)\]pub extern \"C\" fn ([a-zA-Z0-9_]+)(\([a-zA-Z0-9_:]?[^\{]*\)[->]?[^\{]*) \{"#,
    )
    .unwrap();

        for (_, [name, args]) in re
            .captures_iter(&vsched_api_file_content)
            .map(|c| c.extract())
        {
            println!("name: {}\nargs: {}", name, args);
            fns.push((name.to_owned(), args.to_owned()));
        }

        let re = regex::Regex::new(r#"extern \"C\" \{([^\{\}]+)\}"#).unwrap();
        for (_, [extern_fns]) in re
            .captures_iter(&vsched_api_file_content)
            .map(|c| c.extract())
        {
            let fns_re = regex::Regex::new(r#"fn ([a-zA-Z0-9_]+)\(\) -> !;"#).unwrap();
            for (_, [name]) in fns_re.captures_iter(&extern_fns).map(|c| c.extract()) {
                println!("name: {}", name);
                fns.push((name.to_owned(), "() -> !".into()));
            }
        }
    }

    let mut traits = vec![];
    let interface_rs_path = Path::new(&config.src_dir)
        .join("src")
        .join("interface")
        .with_extension("rs");
    // println!("api.rs path: {}", api_rs_path.display());
    if let Ok(mut vsched_interface_file_content) = fs::read_to_string(&interface_rs_path) {
        vsched_interface_file_content = vsched_interface_file_content
            .split('\n')
            .filter(|s| !(*s).trim().starts_with("//"))
            .collect();
        vsched_interface_file_content = vsched_interface_file_content.split('\t').collect();
        vsched_interface_file_content = vsched_interface_file_content.split("    ").collect();
        // println!(
        //     "cargo:warning=vsched_interface_file_content: {}",
        //     vsched_interface_file_content
        // );

        let re =
            regex::Regex::new(r#"trait_interface\! \{pub trait ([a-zA-Z0-9_]+) \{([^\{\}]+)\}\}"#)
                .unwrap();
        // let re = regex::Regex::new(r#"pub trait ([a-zA-Z0-9_]+) \{([^\{\}]+)\}"#).unwrap();
        // let re = regex::Regex::new(r#"pub trait ([a-zA-Z0-9_]+)\{(.)?"#).unwrap();

        // 获取vDSO的 interface
        for (_, [name, fns]) in re
            .captures_iter(&vsched_interface_file_content)
            .map(|c| c.extract())
        {
            let mut fns_name = vec![];
            let fns_name_re = regex::Regex::new(r#"fn ([a-zA-Z0-9_]+)\("#).unwrap();
            fns_name_re
                .captures_iter(&fns)
                .map(|c| c.extract())
                .for_each(|(_, [fn_name])| {
                    fns_name.push(fn_name.to_owned());
                });
            traits.push((name.to_owned(), fns_name));
        }
        println!("cargo:warning=traits: {:?}", traits);
        // panic!("pause");
    }

    // pub use vdso库中的内容
    let pub_use_vdso_str = format!(
        "extern crate {};\nuse alloc::vec::Vec;\npub use page_table_entry::MappingFlags;\npub use self::{}::*;\n\n",
        config.package_name, config.package_name
    );
    // vdso_vtable 数据结构定义
    let mut vdso_vtable_struct_str = "pub struct VdsoVTable {\n".to_string();
    for (name, args) in fns.iter() {
        vdso_vtable_struct_str.push_str(&format!("    pub {}: Option<fn{}>,\n", name, args));
    }
    for (name, fns_name) in traits.iter() {
        let init_fn_name = format!("init_vtable_{}", name);
        let args = format!(
            "({})",
            fns_name
                .iter()
                .map(|_fn_name| "usize")
                .collect::<Vec<_>>()
                .join(", ")
        );
        vdso_vtable_struct_str
            .push_str(&format!("    pub {}: Option<fn{}>,\n", init_fn_name, args));
    }
    vdso_vtable_struct_str.push_str("}\n");

    // 定义静态的 VDSO_VTABLE
    let mut static_vdso_vtable_str =
        "\npub static mut VDSO_VTABLE: VdsoVTable = VdsoVTable {\n".to_string();
    for (name, _) in fns.iter() {
        static_vdso_vtable_str.push_str(&format!("    {}: None,\n", name));
    }
    for (name, _) in traits.iter() {
        let init_fn_name = format!("init_vtable_{}", name);
        static_vdso_vtable_str.push_str(&format!("    {}: None,\n", init_fn_name));
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

    for (name, fns_name) in traits.iter() {
        let init_fn_name = format!("init_vtable_{}", name);
        let mut sym_value: usize = 0;
        for dynsym in dyn_sym_table {
            let sym_name = dynsym.get_name(&vdso_elf).unwrap();
            if sym_name == init_fn_name.as_str() {
                sym_value = dynsym.value() as usize;
                break;
            }
        }
        assert!(
            sym_value != 0,
            "Function {} not found in .dynsym",
            init_fn_name
        );

        let args = format!(
            "({})",
            fns_name
                .iter()
                .map(|_fn_name| "usize")
                .collect::<Vec<_>>()
                .join(", ")
        );

        fn_init_vdso_vtable_str.push_str(&format!(
            r#"    // {}:
    let fn_ptr = base + 0x{:x};
    #[cfg(feature = "log")]
    log::debug!("{}: 0x{{:x}}", fn_ptr);
    let f: fn{} = unsafe {{ core::mem::transmute(fn_ptr) }};
    unsafe {{ VDSO_VTABLE.{}  = Some(f); }}

"#,
            init_fn_name, sym_value, init_fn_name, args, init_fn_name
        ));
    }

    fn_init_vdso_vtable_str.push_str(
        r#"}
    "#,
    );

    //     fn_init_vdso_vtable_str.push_str(
    //         r#"
    // /// 在加载vDSO的地址空间（通常是内核）中调用，同时加载vDSO和初始化VTABLE。
    // ///
    // /// 若在一个地址空间中加载再映射到另一个地址空间中，需使用`map_and_init`。
    // ///
    // /// 该函数的返回值为vDSO和vVAR的映射区域的信息，元组的三项依次为首地址、大小和访问权限。vDSO首地址为第二个映射区域的首地址。
    // ///
    // /// 在调用该库的其余API前，需先调用此函数。
    // pub fn load_and_init() -> Vec<(*mut u8, usize, MappingFlags)> {
    //     let regions = crate::load_so();
    //     let vdso = regions[1].0; // vDSO首地址为第二个映射区域的首地址，因为第一个是vVAR。
    //     unsafe{ init_vdso_vtable(vdso as _) };
    //     regions
    // }

    // /// 将已加载的vdso映射到另一个地址空间，并初始化VTABLE。
    // ///
    // /// 该函数的返回值为vDSO和vVAR的映射区域的信息，元组的四项依次为用户虚拟地址、内核虚拟地址、大小和访问权限。vDSO首地址为第二个映射区域的首地址。
    // ///
    // /// 在调用该库的其余API前，需先调用此函数。
    // pub fn map_and_init(vspace: usize) -> Vec<(*mut u8, *mut u8, usize, MappingFlags)> {
    //     let regions = crate::map_so(vspace);
    //     let vdso = regions[1].0; // vDSO首地址为第二个映射区域的首地址，因为第一个是vVAR。
    //     unsafe{ init_vdso_vtable(vdso as _) };
    //     regions
    // }
    // "#,
    //     );

    fn_init_vdso_vtable_str.push_str(
        r#"
/// 在加载vDSO的地址空间（通常是内核）中调用，同时加载vDSO和初始化VTABLE。
/// 
/// 若在一个地址空间中加载再映射到另一个地址空间中，需使用`map_and_init`。
/// 
/// 该函数的返回值为vDSO和vVAR的映射区域的信息，元组的三项依次为首地址、大小和访问权限。vDSO首地址为第二个映射区域的首地址。
/// 
/// 在调用该库的其余API前，需先调用此函数。
pub fn load_and_init(vspace: usize) {
    let vdso = crate::map_so(vspace);
    unsafe{ init_vdso_vtable(vdso as _) };
}
"#,
    );

    // 构建给内核和用户运行时使用的接口
    let mut apis = vec![];

    // api部分
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
        let res = f({});
        #[cfg(feature = "log")]
        log::debug!("Returned from {}.");
        res
    }} else {{
        panic!("{} is not initialized")
    }}
}}
"#,
            name, args, name, name, fn_args, name, name
        ));
    }

    // trait的初始化api部分
    for (name, fns_name) in traits.iter() {
        let init_fn_name = format!("init_vtable_{}", name);

        let fn_args = fns_name
            .iter()
            .map(|fn_name| format!("T::{} as usize", fn_name))
            .collect::<Vec<_>>()
            .join(", ");

        apis.push(format!(
            r#"
pub fn {}<T:{}>() {{
    if let Some(f) = unsafe {{ VDSO_VTABLE.{} }} {{
        #[cfg(feature = "log")]
        log::debug!("Calling {} at 0x{{:x}}.", f as *const () as usize);
        let res = f({});
        #[cfg(feature = "log")]
        log::debug!("Returned from {}.");
        res
    }} else {{
        panic!("{} is not initialized")
    }}
}}
"#,
            init_fn_name, name, init_fn_name, init_fn_name, fn_args, init_fn_name, init_fn_name
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
/// 在自身不加载vDSO，而是已经映射了vDSO的地址空间（通常是用户进程）中调用，传入vDSO的首地址以初始化VTABLE。
/// 
/// 在调用该库的其余API前，需先调用此函数。
pub unsafe fn init_vdso_vtable(base: u64) {
"#;

fn loader_rs_content(config: &BuildConfig) -> String {
    let use_content = format!(
        r#"use alloc::string::ToString;
use core::str::from_utf8;
use crate_interface::{{call_interface, def_interface}};
use include_bytes_aligned::include_bytes_aligned;
pub use page_table_entry::MappingFlags;
use {}::VvarData;
use xmas_elf::program::SegmentData;
use alloc::vec::Vec;
use core::sync::atomic::{{AtomicPtr, Ordering}};
use lazyinit::LazyInit;
"#,
        config.package_name
    );

    //     let interface_content = String::from(
    //         r#"
    // /// 在内核初始化时加载vDSO使用的接口。
    // ///
    // /// 实现了这些接口后，可以调用`loader::load_so`或`api::load_and_init`
    // #[def_interface]
    // pub trait MemIf {
    //     /// 分配用于vDSO和vVAR的空间，返回指向首地址的指针。
    //     ///
    //     /// 保证size为build_vdso传入的config.page_size的整数倍；
    //     /// 要求返回的地址也为config.page_size的整数倍。
    //     ///
    //     /// 若需要实现vDSO和vVAR在多地址空间的共享，则需要在分配时使这块空间可被共享。
    //     fn alloc(size: usize) -> *mut u8;

    //     /// 从`alloc`返回的空间中，设置其中一块的访问权限。
    //     ///
    //     /// 保证addr对齐到build_vdso传入的config.page_size；len为config.page_size的整数倍。
    //     /// 如果从so文件中解析出的段基址不对齐，则panic；段长度不为config.page_size的整数倍则向上取整。
    //     ///
    //     /// `flags`可能包含：READ、WRITE、EXECUTE、USER。
    //     fn protect(addr: *mut u8, len: usize, flags: MappingFlags);
    // }

    // /// 将已加载的vDSO映射到其它地址空间使用的接口。
    // ///
    // /// 实现了这些接口后，可以调用`loader::map_so`或`api::map_and_init`
    // #[def_interface]
    // pub trait UserMemIf {
    //     /// 在地址空间中分配用于vDSO和vVAR的虚存区域（不需同时分配物理页面），返回指向首地址的指针。
    //     ///
    //     /// 保证size为build_vdso传入的config.page_size的整数倍。
    //     /// 要求返回的地址也为config.page_size的整数倍。
    //     fn ualloc(vspace: usize, size: usize) -> *mut u8;

    //     /// 从`alloc`返回的虚存区域中，映射其中一块到某个内核虚拟地址所指示的物理页面并设置权限。
    //     ///
    //     /// 被映射的物理页面可能和其它地址空间共享，也可能由这个地址空间独占。
    //     ///
    //     /// 保证uaddr、kaddr对齐到build_vdso传入的config.page_size；len为config.page_size的整数倍。
    //     ///
    //     /// `flags`可能包含：READ、WRITE、EXECUTE、USER。
    //     fn map(vspace: usize, uaddr: *mut u8, kaddr: *mut u8, len: usize, flags: MappingFlags);
    // }
    // "#,
    //     );

    let interface_content = String::from(
        r#"
/// 因为不同系统中代表物理页的类型（假设为`PhysPage`）不同，
/// 物理页类型可能使用RAII管理（在物理页对象释放时释放实际物理页），
/// 而接口中很难支持泛型，
/// 所以，本库以`*const ManuallyDrop<PhysPage>`的形式管理物理页，
/// 并转化为usize以取消泛型属性并方便作为全局变量。
/// 
/// （`ManuallyDrop`只是为了强调指针指向的物理页不会被自动释放，不一定要求转化的类型中一定带有`ManuallyDrop`。
/// 例如，如果os中使用`Arc<PhysPage>`管理物理页，那么使用`Arc::into_raw()`得到的`*const PhysPage`就可以作为`PhysPagePtr`。）
/// 
/// `PhysPagePtr`的生命周期如下图：
/// 
/// `MemIf::ppage_alloc`将`PhysPage`转化为`PhysPagePtr` ➡ `MemIf::ppage_clone`复制 ➡ 存储在库中
/// 
///                                  ⬇                                                  ⬇
/// 
///                                  ⬇                                       `MemIf::ppage_clone`复制
/// 
///                                  ⬇                                                  ⬇
/// 
///                                  ➡          ➡          `MemIf::map`将`PhysPagePtr`重新转化为`PhysPage`，并加入页表
/// 
/// 保证每个指针的生命周期从`MemIf::ppage_alloc`开始到`MemIf::map`结束，
/// 且每个指针只会`map`一次，`map`之后即不再使用该指针。
/// 
/// 为了实现物理页的共享，内核的vdso加载到的物理页指针需要`clone`后在本库中暂存一份。
/// 下次加载共享的物理页后，将暂存的指针再`clone`一次，并对`clone`后的指针调用`map`。
/// 这样保证了库中暂存的指针仍有效。
/// 虽然内核的物理页指针会被暂存一份，导致无法释放，但在内核加载的vdso页面本来就在内核关闭时才能释放，
/// 因此没有问题。
/// 
/// 加载用户vdso时新分配的物理页则没有`clone`和暂存的过程，在`alloc`后即调用`map`。
/// 因此不会影响`PhysPage`的生命周期管理。
pub type PhysPagePtr = usize;

/// 加载和映射vDSO使用的接口。
///
/// 实现了这些接口后，内核可以通过以下方式实现vDSO模块的初始化：
///
/// - 内核态初始化（以下步骤已封装在`map_and_init`函数中）：
///     1. 在内核调用`map_so`（需保证是首次调用），加载和重定位so文件
///     `map_so`的行为：无论共享数据、代码还是私有数据，均会分配物理页并加载。
///     2. 在内核调用`init_vdso_vtable`，初始化内核空间中的`VDSO_VTABLE`。
/// - 用户态初始化：
///     1. 在内核调用`map_so`（需保证是后续调用），加载和重定位so文件。
///     `map_so`的行为：对于共享数据和代码会映射到已分配的物理页；对于私有数据会重新分配物理页并加载。
///     2. 在用户态调用`init_vdso_vtable`，初始化用户空间中的`VDSO_VTABLE`。
#[def_interface]
pub trait MemIf {
    /// 在地址空间中分配用于vDSO和vVAR的虚存区域（不需同时分配物理页面），返回指向首地址的指针。
    /// 
    /// 保证size为build_vdso传入的config.page_size的整数倍。
    /// 要求返回的地址也为config.page_size的整数倍。
    fn valloc(vspace: usize, size: usize) -> *mut u8;

    /// 分配多块用于vDSO和vVAR的连续物理页，返回`PhysPagePtr`。
    /// 
    /// 保证size为build_vdso传入的config.page_size的整数倍。
    ///
    /// 若需要实现vDSO和vVAR在多地址空间的共享，则需要在分配时使这块空间可被共享（即，可被多次`map`）。
    fn ppage_alloc(size: usize) -> PhysPagePtr;

    /// 从`alloc`返回的虚存区域中，映射其中一块到某个物理页面并设置权限。
    /// 
    /// 被映射的物理页面可能和其它地址空间共享，也可能由这个地址空间独占。（由`shared`指定）
    /// 
    /// 保证vaddr对齐到build_vdso传入的config.page_size；len为config.page_size的整数倍。
    ///
    /// `flags`可能包含：READ、WRITE、EXECUTE、USER。
    fn map(vspace: usize, vaddr: *mut u8, ppage: PhysPagePtr, size: usize, flags: MappingFlags, shared: bool);

    /// 重新设置已映射好的，虚拟首地址为`vspace`区域的权限。
    /// 
    /// 保证vaddr对齐到build_vdso传入的config.page_size。
    fn change_protect(vspace: usize, vaddr: *mut u8, size: usize, flags: MappingFlags);

    /// 获取`vspace`空间中`vaddr`地址对应的内核虚拟地址。
    /// （也就是当前代码可以直接访问的地址）
    fn get_kernel_vaddr(vspace: usize, vaddr: *mut u8) -> *mut u8;

    /// 复制物理页指针，复制前后指向同一块物理页。复制后，参数和返回值对应的两个指针均需可用。
    /// 
    /// 如果物理页使用RAII管理，则需调用其`clone`方法。
    /// 
    /// 如果物理页不使用RAII管理，则可以直接返回参数。
    fn ppage_clone(ppage: PhysPagePtr) -> PhysPagePtr;
}
"#,
    );

    let const_content = format!(
        r#"
const PAGES_SIZE: usize = {};
const VDSO: &[u8] = include_bytes_aligned!(8, "../../{}.so");
const VDSO_SIZE: usize = ((VDSO.len() + PAGES_SIZE - 1) & (!(PAGES_SIZE - 1))) + PAGES_SIZE; // 额外加了一页，用于bss段等未出现在文件中的段
const VVAR_SIZE: usize = (core::mem::size_of::<VvarData>() + PAGES_SIZE - 1) & (!(PAGES_SIZE - 1));
"#,
        config.page_size, config.so_name
    );

    //     let load_so_content = String::from(
    //         r#"
    // static KBASE: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
    // pub(crate) fn load_so() -> Vec<(*mut u8, usize, MappingFlags)> {
    //     let vdso_map = call_interface!(MemIf::alloc(VVAR_SIZE + VDSO_SIZE));
    //     KBASE.store(vdso_map, Ordering::Release);
    //     let mut regions = Vec::new();
    //     #[cfg(feature = "log")]
    //     {
    //         log::info!(
    //             "vVAR: [0x{:016x}, 0x{:016x})",
    //             vdso_map as usize,
    //             (vdso_map as usize) + VVAR_SIZE
    //         );
    //         log::info!(
    //             "vDSO: [0x{:016x}, 0x{:016x})",
    //             (vdso_map as usize) + VVAR_SIZE,
    //             (vdso_map as usize) + VVAR_SIZE + VDSO_SIZE
    //         );
    //     }

    //     // vVAR初始化
    //     #[cfg(feature = "log")]
    //     log::info!("loading vVAR...");
    //     #[cfg(feature = "log")]
    //     log::info!(
    //         "protect: [0x{:016x}, 0x{:016x}), {:?}",
    //         vdso_map as usize,
    //         vdso_map as usize + VVAR_SIZE,
    //         MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER
    //     );
    //     call_interface!(MemIf::protect(
    //         vdso_map,
    //         VVAR_SIZE,
    //         MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER
    //     ));
    //     regions.push((vdso_map, VVAR_SIZE, MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER));
    //     unsafe { (vdso_map as *mut _ as *mut VvarData).write(VvarData::default()) };

    //     // vDSO初始化
    //     #[cfg(feature = "log")]
    //     log::info!("loading vDSO...");

    //     let vdso_elf = xmas_elf::ElfFile::new(VDSO).expect("Error parsing app ELF file.");
    //     if let Some(interp) = vdso_elf
    //         .program_iter()
    //         .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Interp))
    //     {
    //         let interp = match interp.get_data(&vdso_elf) {
    //             Ok(SegmentData::Undefined(data)) => data,
    //             _ => panic!("Invalid data in Interp Elf Program Header"),
    //         };

    //         let interp_path = from_utf8(interp).expect("Interpreter path isn't valid UTF-8");
    //         // remove trailing '\0'
    //         let _interp_path = interp_path.trim_matches(char::from(0)).to_string();
    //         #[cfg(feature = "log")]
    //         log::debug!("Interpreter path: {:?}", _interp_path);
    //     }
    //     let elf_base_addr = Some((vdso_map as usize) + VVAR_SIZE);
    //     let segments = elf_parser::get_elf_segments(&vdso_elf, elf_base_addr);
    //     let relocate_pairs = elf_parser::get_relocate_pairs(&vdso_elf, elf_base_addr);
    //     for segment in segments {
    //         if segment.size == 0 {
    //             #[cfg(feature = "log")]
    //             log::warn!(
    //                 "Segment with size 0 found, skipping: {:?}, {:#x}, {:?}",
    //                 segment.vaddr,
    //                 segment.size,
    //                 segment.flags
    //             );
    //             continue;
    //         }
    //         #[cfg(feature = "log")]
    //         log::debug!(
    //             "{:?}, {:#x}, {:?}",
    //             segment.vaddr,
    //             segment.size,
    //             segment.flags
    //         );

    //         if let Some(data) = segment.data {
    //             assert!(data.len() <= segment.size);
    //             let src = data.as_ptr();
    //             let dst = segment.vaddr.as_usize() as *mut u8;
    //             let count = data.len();
    //             unsafe {
    //                 core::ptr::copy_nonoverlapping(src, dst, count);
    //                 if segment.size > count {
    //                     core::ptr::write_bytes(dst.add(count), 0, segment.size - count);
    //                 }
    //             }
    //         } else {
    //             unsafe { core::ptr::write_bytes(segment.vaddr.as_usize() as *mut u8, 0, segment.size) };
    //         }

    //         assert!(segment.vaddr.as_usize() & (PAGES_SIZE - 1) == 0);
    //         let size = (segment.size + PAGES_SIZE - 1) & (!(PAGES_SIZE - 1));
    //         #[cfg(feature = "log")]
    //         log::info!(
    //             "protect: [0x{:016x}, 0x{:016x}), {:?}",
    //             segment.vaddr.as_usize(),
    //             segment.vaddr.as_usize() + size,
    //             segment.flags
    //         );
    //         call_interface!(MemIf::protect(
    //             segment.vaddr.as_usize() as *mut u8,
    //             size,
    //             segment.flags
    //         ));
    //         regions.push((segment.vaddr.as_usize() as *mut u8, size, segment.flags));
    //     }

    //     for relocate_pair in relocate_pairs {
    //         let src: usize = relocate_pair.src.into();
    //         let dst: usize = relocate_pair.dst.into();
    //         let count = relocate_pair.count;
    //         #[cfg(feature = "log")]
    //         log::info!(
    //             "Relocate: src: 0x{:x}, dst: 0x{:x}, count: {}",
    //             src,
    //             dst,
    //             count
    //         );
    //         unsafe { core::ptr::copy_nonoverlapping(src.to_ne_bytes().as_ptr(), dst as *mut u8, count) }
    //     }

    //     #[cfg(feature = "log")]
    //     log::info!("mapping complete!");

    //     // ((vdso_map as usize) + VVAR_SIZE) as _
    //     regions
    // }
    // "#,
    //     );
    let map_so_content = String::from(
        r#"
/// 内核虚拟地址、内核物理页、大小、flags
static KERNEL_VDSO_REGIONS: LazyInit<Vec<(usize, PhysPagePtr, usize, MappingFlags)>> = LazyInit::new();

/// - 第一次调用：加载并映射vdso。本次调用中，vspace需为当前地址空间。
/// - 后续调用：将已加载的vdso映射到另一个地址空间。
/// 
/// 该函数的返回值为本次映射的vdso首地址（vspace中的虚拟地址）。
pub fn map_so(vspace: usize) -> *mut u8 {
    let vdso_elf = xmas_elf::ElfFile::new(VDSO).expect("Error parsing app ELF file.");
    if let Some(interp) = vdso_elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Interp))
    {
        let interp = match interp.get_data(&vdso_elf) {
            Ok(SegmentData::Undefined(data)) => data,
            _ => panic!("Invalid data in Interp Elf Program Header"),
        };

        let interp_path = from_utf8(interp).expect("Interpreter path isn't valid UTF-8");
        // remove trailing '\0'
        let _interp_path = interp_path.trim_matches(char::from(0)).to_string();
        #[cfg(feature = "log")]
        log::debug!("Interpreter path: {:?}", _interp_path);
    }
    let segments = elf_parser::get_elf_segments(&vdso_elf, Some(0));
    let vdso_size: usize = segments.iter().map(|seg| ((seg.size + PAGES_SIZE - 1) / PAGES_SIZE) * PAGES_SIZE).sum();

    let vbase = call_interface!(MemIf::valloc(vspace, VVAR_SIZE + vdso_size));
    let mut regions = Vec::new();

    // vVAR初始化
    #[cfg(feature = "log")]
    log::info!("mapping vVAR...");
    let vaddr = vbase;
    // ppage用于映射
    // ppage_store用于存储在KERNEL_VDSO_REGIONS中（只有首次调用时有意义）
    let (ppage, ppage_store) = if !KERNEL_VDSO_REGIONS.is_inited() {
        // 首次调用，分配物理页并加载vVAR
        let ppage = call_interface!(MemIf::ppage_alloc(VVAR_SIZE));
        let ppage_clone = call_interface!(MemIf::ppage_clone(ppage));
        (ppage, ppage_clone)
    } else {
        // 后续调用，映射已加载的vVAR
        let origin_ppage = KERNEL_VDSO_REGIONS.get().unwrap()[0].1;
        let ppage = call_interface!(MemIf::ppage_clone(origin_ppage));
        (ppage, ppage)
    };
    let flags = if !KERNEL_VDSO_REGIONS.is_inited() {
        // 首次调用，内核空间的vVAR不设置USER
        MappingFlags::READ | MappingFlags::WRITE
    } else {
        // 后续调用，用户空间的vVAR设置USER
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER
    };
    #[cfg(feature = "log")]
    log::info!(
        "map: vspace: 0x{:016x}, vaddr: 0x{:016x}, ppage_struct_ptr: 0x{:016x}, size: 0x{:x} {:?}, shared: true",
        vspace,
        vaddr as usize,
        ppage,
        VVAR_SIZE,
        flags
    );
    call_interface!(MemIf::map(vspace, vaddr, ppage, VVAR_SIZE, flags, true));
    // 初始化vvar，只在首次调用时写入数据，后续调用时内核加载的vVAR页面已经包含了正确的数据。
    // 只在首次调用时，存储region信息
    if !KERNEL_VDSO_REGIONS.is_inited() {
        unsafe { (vaddr as *mut VvarData).write(VvarData::default()) };
        regions.push((vaddr as usize, ppage_store, VVAR_SIZE, flags));
    }

    // vDSO初始化
    #[cfg(feature = "log")]
    log::info!("mapping vDSO...");
    let elf_base_addr = Some((vbase as usize) + VVAR_SIZE);
    let segments = elf_parser::get_elf_segments(&vdso_elf, elf_base_addr);
    let relocate_pairs = elf_parser::get_relocate_pairs(&vdso_elf, elf_base_addr);
    let mut index = 1;
    for segment in segments {
        if segment.size == 0 {
            #[cfg(feature = "log")]
            log::warn!(
                "Segment with size 0 found, skipping: {:?}, {:#x}, {:?}",
                segment.vaddr,
                segment.size,
                segment.flags
            );
            continue;
        }
        #[cfg(feature = "log")]
        log::debug!(
            "{:?}, {:#x}, {:?}",
            segment.vaddr,
            segment.size,
            segment.flags
        );

        assert!(segment.vaddr.as_usize() & (PAGES_SIZE - 1) == 0);
        let size = (segment.size + PAGES_SIZE - 1) & (!(PAGES_SIZE - 1));
        let vaddr = segment.vaddr.as_mut_ptr();
        let (ppage, ppage_store) = if !KERNEL_VDSO_REGIONS.is_inited() {
            // 首次调用，分配物理页并加载vDSO
            let ppage = call_interface!(MemIf::ppage_alloc(size));
            let ppage_clone = call_interface!(MemIf::ppage_clone(ppage));
            (ppage, ppage_clone)
        } else {
            // 后续调用
            if !segment.flags.contains(MappingFlags::WRITE) {
                // 代码段/只读数据段，使用已加载的vDSO
                let origin_ppage = KERNEL_VDSO_REGIONS.get().unwrap()[index].1;
                let ppage = call_interface!(MemIf::ppage_clone(origin_ppage));
                (ppage, ppage)
            } else {
                // 读写数据段，重新分配物理页，且后续需要加载和重定位
                let ppage = call_interface!(MemIf::ppage_alloc(size));
                (ppage, ppage)
            }
        };
        let flags = if !KERNEL_VDSO_REGIONS.is_inited() {
            // 首次调用，内核空间的vDSO不设置USER
            segment.flags & !MappingFlags::USER
        } else {
            // 后续调用，用户空间的vDSO设置USER
            segment.flags | MappingFlags::USER
        };
        // 首先需以WRITE和!USER权限映射，以便加载和重定位；加载和重定位完成后再设置为最终权限。
        let flags_with_write = flags | MappingFlags::WRITE & !MappingFlags::USER;
        let shared = !segment.flags.contains(MappingFlags::WRITE);
        #[cfg(feature = "log")]
        log::info!(
            "map: vspace: 0x{:016x}, vaddr: 0x{:016x}, ppage_struct_ptr: 0x{:016x}, size: 0x{:x} {:?}, shared: {}",
            vspace,
            vaddr as usize,
            ppage,
            size,
            flags_with_write,
            shared,
        );
        call_interface!(MemIf::map(vspace, vaddr, ppage, size, flags_with_write, shared));
        if !KERNEL_VDSO_REGIONS.is_inited() || !segment.flags.contains(MappingFlags::EXECUTE) {
            // “首次调用”或“后续调用的数据段”，加载和重定位vDSO
            // 因为在“后续调用的数据段”情况下，虚拟地址不一定能直接访问，因此需要转化。
            if let Some(data) = segment.data {
                assert!(data.len() <= size);
                let src = data.as_ptr();
                let dst = call_interface!(MemIf::get_kernel_vaddr(vspace, vaddr));
                let count = data.len();
                unsafe {
                    core::ptr::copy_nonoverlapping(src, dst, count);
                    if size > count {
                        core::ptr::write_bytes(dst.add(count), 0, size - count);
                    }
                }
            } else {
                unsafe { core::ptr::write_bytes(vaddr, 0, size) };
            }
            for relocate_pair in &relocate_pairs {
                let relo_src: usize = relocate_pair.src.into();
                let relo_dst: usize = relocate_pair.dst.into();
                let count = relocate_pair.count;
                if segment.vaddr.as_usize() <= relo_dst
                    && relo_dst < segment.vaddr.as_usize() + size
                {
                    let relo_kdst =
                        call_interface!(MemIf::get_kernel_vaddr(vspace, relo_dst as *mut u8));
                    #[cfg(feature = "log")]
                    log::info!(
                        "Relocate: src: 0x{:x}, udst: 0x{:x}, kdst: 0x{:x}, count: {}",
                        relo_src,
                        relo_dst,
                        relo_kdst as usize,
                        count
                    );
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            relo_src.to_ne_bytes().as_ptr(),
                            relo_kdst,
                            count,
                        )
                    }
                }
            }
        } else {
            // 后续调用的代码段，确认代码段没有重定位
            for relocate_pair in &relocate_pairs {
                let relo_dst: usize = relocate_pair.dst.into();
                if vaddr as usize <= relo_dst && relo_dst < vaddr as usize + size {
                    panic!("Relocate pair found in text section!");
                }
            }
        }
        if flags != flags_with_write {
            #[cfg(feature = "log")]
            log::info!(
                "change_protect: vspace: 0x{:016x}, vaddr: 0x{:016x}, size: 0x{:x}, flags: {:?}",
                vspace,
                vaddr as usize,
                size,
                flags
            );
            call_interface!(MemIf::change_protect(vspace, vaddr, size, flags));
        }
        if !KERNEL_VDSO_REGIONS.is_inited() {
            regions.push((vaddr as usize, ppage_store, size, flags));
        }
        index += 1;
    }

    #[cfg(feature = "log")]
    log::info!("mapping complete!");

    if !KERNEL_VDSO_REGIONS.is_inited() {
        KERNEL_VDSO_REGIONS.init_once(regions);
    }

    ((vbase as usize) + VVAR_SIZE) as _
}
"#,
    );

    // use_content + &interface_content + &const_content + &load_so_content + &map_so_content
    use_content + &interface_content + &const_content + &map_so_content
}
