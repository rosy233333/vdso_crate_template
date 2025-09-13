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
    /// 生成的vdso库的soname
    pub so_name: String,
    /// 编译模式，"debug"或"release"
    pub mode: String,
    /// 冗长度，0表示不冗长，1表示冗长输出，2表示更冗长输出
    pub verbose: usize,
}

impl BuildConfig {
    /// 创建一个新的BuildConfig实例
    /// 尽量使用new()函数创建实例，并在创建后手动修改需要修改的字段。
    /// 默认值为：
    /// - arch: "riscv64"
    /// - out_dir: "."
    /// - so_name: "lib" + package_name
    /// - mode: "release"
    /// - verbose: 0
    pub fn new(src_dir: &str, package_name: &str) -> Self {
        Self {
            arch: "riscv64".to_string(),
            src_dir: src_dir.to_string(),
            package_name: package_name.to_string(),
            out_dir: ".".to_string(),
            so_name: "lib".to_string() + package_name,
            mode: "release".to_string(),
            verbose: 0,
        }
    }
}
