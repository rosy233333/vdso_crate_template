//! 用于创建可在编译期由环境变量指定的常量。
//!
//! 使用该模块的原因是vDSO的接口不支持泛型，因此许多常量都需要在编译期确定。
//!
//! 通过使用编译期的环境变量改变常量的值，可以为vDSO模块带来一定的灵活性。
//!
//! ## 包含
//!
//! - [`mut_cfg!`](`crate::mut_cfg!`)
//! - [`use_mut_cfg!`](`crate::use_mut_cfg!`)

/// 生成可变配置常量的宏。
///
/// 如果编译过程中指定了与某个常量同名的环境变量，则该常量的值将被替换为环境变量的值。
///
/// 否则，常量将使用提供的默认值。
///
/// 使用方法：
///
/// - 在`build.rs`中调用该宏，传入需要生成的常量列表。
/// - 每个常量的声明格式与正常的常量声明相同。
/// - 目前常量的类型只支持数值和布尔类型，不支持字符串类型。
/// - 常量的默认值支持表达式，但引用mut_cfg中的其它常量时只能由后者引用前者。
/// - 若表达式中引用的常量被环境变量覆盖，则引用的值为环境变量的值。
#[macro_export]
macro_rules! mut_cfg {
    ($(const $i:ident: $t:ty = $d:expr;)*) => {
        use std::path::Path;
        use std::option::Option;
        use std::result::Result;

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir).join("mut_cfgs.rs");

        $(
            let $i: $t = option_env!(stringify!($i)).map_or($d, |env| env.parse::<$t>().unwrap());
        )*

        let mut mut_cfg = String::new();

        $(
            mut_cfg += format!(
                    r#"pub const {}: {} = {};
"#,
                    stringify!($i),
                    stringify!($t),
                    $i
                ).as_str();
        )*

        std::fs::write(&out_path, mut_cfg).unwrap();

        println!("cargo:rerun-if-changed=src/*");
        println!("cargo:rerun-if-changed={}", out_path.display());
        $(
            println!("cargo:rerun-if-env-changed={}", stringify!($i));
        )*
    };
}

/// 引入生成的可变配置常量模块的宏。
///
/// 直接不带参数地调用该宏即可在当前模块中引入所有生成的可变配置常量。
#[macro_export]
macro_rules! use_mut_cfg {
    () => {
#[rustfmt::skip]
mod mut_cfgs {
    include!(concat!(env!("OUT_DIR"), "/mut_cfgs.rs"));
}

        pub use mut_cfgs::*;
    };
}
