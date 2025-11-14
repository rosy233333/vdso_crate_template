pub struct BuildConfig {
    /// 目标架构，有效值为"x86_64"、"aarch64"、"riscv64"
    pub arch: String,
    /// vdso源代码所在目录
    pub src_dir: String,
    /// vdso包名称
    pub package_name: String,
    /// 编译输出目录，会输出链接脚本和最终的.so文件
    /// 若在build.rs中调用，则应传入环境变量OUT_DIR的值
    pub out_dir: String,
    /// 生成的vdso库的soname，默认为"lib" + package_name
    pub so_name: String,
    /// 编译模式，"debug"或"release"
    pub mode: String,
    /// 冗长度，0表示不冗长，1表示冗长输出，2表示更冗长输出
    pub verbose: usize,
    /// 生成的api库的名称，默认为"lib" + package_name
    /// 该库会被拷贝到输出目录，并可被调用者依赖
    pub api_lib_name: String,
    /// 编译vdso使用的工具链版本
    /// 默认为"nightly"，可指定具体版本号，如"nightly-2025-09-12"
    pub toolchain: String,
}

impl BuildConfig {
    /// 创建一个新的BuildConfig实例。
    ///
    /// 尽量使用new()函数创建实例，并在创建后手动修改需要修改的字段。
    ///
    /// 默认值为：
    ///
    /// - arch: "riscv64"
    /// - out_dir: "."
    /// - so_name: "lib" + package_name
    /// - mode: "release"
    /// - verbose: 0
    /// - api_lib_name: "lib" + package_name
    /// - toolchain: "nightly"
    ///
    /// 其他字段必须手动指定。
    ///
    /// 字段中若使用相对路径，则相对路径均相对于调用build_vdso函数的程序的工作目录。
    ///
    /// （如果在build.rs中调用，则为相对于build.rs的相对路径）
    pub fn new(src_dir: &str, package_name: &str) -> Self {
        Self {
            arch: "riscv64".to_string(),
            src_dir: src_dir.to_string(),
            package_name: package_name.to_string(),
            out_dir: ".".to_string(),
            so_name: "lib".to_string() + package_name,
            mode: "release".to_string(),
            verbose: 0,
            api_lib_name: "lib".to_string() + package_name,
            toolchain: "nightly".to_string(),
        }
    }
}
