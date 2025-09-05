use core::mem::MaybeUninit;
use core::sync::atomic::Ordering;

use structs::argument::*;
use structs::shared::*;

use crate::get_data_base;
use crate::get_vvar_data;
use crate::init_vvar_data;

/// 初始化vDSO。
/// 若vDSO在多个地址空间中共享，则只需调用一次。
#[unsafe(no_mangle)]
pub extern "C" fn init() {
    init_vvar_data();
}

#[unsafe(no_mangle)]
pub extern "C" fn get_example() -> ArgumentExample {
    let vvar_data = unsafe { get_vvar_data() };
    ArgumentExample {
        i: vvar_data.example.i,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn set_example(i: usize) {
    let vvar_data = unsafe { get_vvar_data() };
    vvar_data.example.i = i;
}
