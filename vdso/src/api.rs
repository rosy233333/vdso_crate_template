use core::mem::MaybeUninit;
use core::sync::atomic::Ordering;

// use structs::argument::*;
use vdso_helper::get_vvar_data;
// use structs::shared::*;

use crate::{ArgumentExample, PRIVATE_DATA_EXAMPLE};

// /// 初始化vDSO。
// /// 若vDSO在多个地址空间中共享，则只需调用一次。
// #[unsafe(no_mangle)]
// pub extern "C" fn init() {
//     init_vvar_data();
// }

#[unsafe(no_mangle)]
pub extern "C" fn get_shared() -> ArgumentExample {
    ArgumentExample {
        i: get_vvar_data!(example).load(Ordering::Acquire),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn set_shared(i: usize) {
    get_vvar_data!(example).store(i, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn get_private() -> ArgumentExample {
    ArgumentExample {
        i: PRIVATE_DATA_EXAMPLE.load(Ordering::Acquire),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn set_private(i: usize) {
    PRIVATE_DATA_EXAMPLE.store(i, Ordering::Release);
}
