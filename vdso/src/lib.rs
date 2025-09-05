//! vDSO共享库的主模块。
//!
//! 需要在vDSO共享库中实现的功能，都应在该模块中实现。
//!
//! ## 共享数据与私有数据
//!
//! 该模块的静态变量分为共享数据和私有数据两类。前者会在不同的地址空间中共享，后者只在该地址空间中使用，不同地址空间持有不同的拷贝。
//!
//! 在该模块中直接创建的全局变量即为私有数据。
//!
//! 调用`init`函数后，`VVAR_DATA`即被设置为指向共享数据结构`VvarData`的指针。
//!
//! 将该模块需要用到的所有共享数据结构声明在`structs::shared`中、且声明为`#[repr(C)]`，并将它们全部放入`VvarData`数据结构（例如当前的`structs::shared::SharedExample`）。
//!
//! 共享数据机制的实现依赖于`init`函数和`get_data_base`函数，因此不应轻易修改这些函数。
//!
//! ## API
//!
//! 为了让项目的`api`模块自动生成调用该模块内API的代码，因此本模块内的API需要遵从以下约定：
//!
//! 1. 所有的API均放置在本模块的`api`子模块中。
//! 2. 目前，API仅支持函数形式。并且函数需要声明为`#[unsafe(no_mangle)]`和`pub extern "C"`。
//! 3. 函数的参数和返回值用到的自定义数据结构，均在`structs::arguments`中定义，且声明为`#[repr(C)]`（例如当前的`structs::arguments::ArgumentExample`）。
//!
#![no_std]

mod api;

use core::{mem::MaybeUninit, sync::atomic::AtomicPtr};
use structs::shared::VvarData;

pub use api::*;

/// Safety:
///     the offset of this function in the `.text`
///     section must be little than 0x1000.
///     The `#[inline(never)]` attribute and the
///     offset requirement can make it work ok.
#[inline(never)]
fn get_data_base() -> usize {
    let pc = unsafe { hal::asm::get_pc() };
    const VSCHED_DATA_SIZE: usize = (core::mem::size_of::<VvarData>() + config::PAGES_SIZE_4K - 1)
        & (!(config::PAGES_SIZE_4K - 1));
    (pc & config::DATA_SEC_MASK) - VSCHED_DATA_SIZE
}

fn init_vvar_data() {
    let data_base = get_data_base() as *mut MaybeUninit<VvarData>;
    unsafe {
        data_base.write(MaybeUninit::new(VvarData::new()));
    }
}

/// SAFETY: 必须在init_vvar_data后调用，也就是只能在api中init以外的函数中使用。
unsafe fn get_vvar_data() -> &'static mut VvarData {
    let data_base = get_data_base() as *mut MaybeUninit<VvarData>;
    (*data_base).assume_init_mut()
}

#[cfg(all(target_os = "linux", not(test)))]
mod lang_item {
    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> ! {
        loop {}
    }
}
