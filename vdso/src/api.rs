use core::mem::MaybeUninit;

use structs::VvarData;

use crate::{VVAR_DATA, get_data_base};

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    let data_base = get_data_base() as *mut MaybeUninit<VvarData>;
    unsafe {
        data_base.write(MaybeUninit::new(VvarData::new()));
    }

    VVAR_DATA.init_once(unsafe { &mut *(data_base as *mut VvarData) });
}

#[unsafe(no_mangle)]
pub extern "C" fn api_example() -> usize {
    let vvar_data = VVAR_DATA.get().expect("VvarData not initialized");
    vvar_data.example.i
}
