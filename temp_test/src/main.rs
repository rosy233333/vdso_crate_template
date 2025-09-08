use std::sync::atomic::AtomicUsize;

fn main() {
    let addr1: usize = vdso_helper::get_vvar_data!(example) as *const _ as usize;
    let addr2: usize = vdso_helper::get_vvar_data!(example, 0x2000) as *const _ as usize;
    println!("addr1: {:#x}, addr2: {:#x}", addr1, addr2);
}

vdso_helper::vvar_data! {
    example: AtomicUsize,
}
