//! vDSO共享库。
//!
//! 需要编译为so文件中的功能代码，参考该库的写法。
//!
//! 其与编写一般的库代码十分相似，除了以下这些不同：
//!
//! ## 共享数据与私有数据
//!
//! 该库的静态变量分为共享数据和私有数据两类。前者会在不同的地址空间中共享，后者只在该地址空间中使用，不同地址空间持有不同的拷贝。
//!
//! 使用`vdso_helper`库中的`vvar_data!`宏声明的全局变量即为共享数据。
//!
//! 共享数据声明后，可使用`get_vvar_data!`宏获取其引用。
//!
//! ## API
//!
//! 为了让项目的`build_vdso`库自动完成该库的构建，因此本库内的API需要遵从以下约定：
//!
//! 1. 所有的API均放置在本模块的`api`子模块中。
//! 2. 目前，API仅支持函数形式。并且函数需要声明为`#[unsafe(no_mangle)]`和`pub extern "C"`。
//! 3. 函数的参数和返回值用到的自定义数据结构，均需要声明为`pub`和`#[repr(C)]`（例如此处的`ArgumentExample`）。
//! 4. 该库导出的所有函数和数据结构均需要导出在根模块中。
//!     （例如，导出子模块中的`pub`符号时，需要使用`pub use submod::*;`，而非`pub use submod;`）
//!
#![no_std]

mod api;

use core::sync::atomic::AtomicUsize;
use vdso_helper::vvar_data;

pub use api::*;

vvar_data! {
    example: AtomicUsize
}

static PRIVATE_DATA_EXAMPLE: AtomicUsize = AtomicUsize::new(0);

#[repr(C)]
pub struct ArgumentExample {
    pub i: usize,
}
