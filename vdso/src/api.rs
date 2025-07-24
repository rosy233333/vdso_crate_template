use core::mem::MaybeUninit;
use core::sync::atomic::Ordering;

use structs::argument::*;
use structs::shared::*;

use crate::{VVAR_DATA, get_data_base};

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    let data_base = get_data_base() as *mut MaybeUninit<VvarData>;
    unsafe {
        data_base.write(MaybeUninit::new(VvarData::new()));
    }

    VVAR_DATA.store(data_base as *mut VvarData, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn api_example() -> ArgumentExample {
    let vvar_data = unsafe { &mut *VVAR_DATA.load(Ordering::Acquire) };
    ArgumentExample {
        i: vvar_data.example.i,
    }
}
