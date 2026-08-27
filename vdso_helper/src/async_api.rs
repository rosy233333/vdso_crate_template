//! 辅助build_vdso组装async接口。

/// 目前这个宏不会产生代码，而是用作build_vdso读取的标记。
///
/// build_vdso读取该宏的三个参数，提取对应的内容，组装成一个async接口。
#[macro_export]
macro_rules! async_api {
    ($async_fn_name:ident, $fut_struct_name:ident, $poll_fn_name:ident) => {};
}
