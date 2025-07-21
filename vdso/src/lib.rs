#![no_std]

mod api;

use lazyinit::LazyInit;
use structs::VvarData;

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

static VVAR_DATA: LazyInit<&mut VvarData> = LazyInit::new();

#[cfg(all(target_os = "linux", not(test)))]
mod lang_item {
    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> ! {
        loop {}
    }
}
