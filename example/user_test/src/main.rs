use crate::map::map_vdso;
use libvdsoexample::*;

mod map;

fn main() {
    env_logger::init();
    log::info!("Starting VDSO test...");
    let map = map_vdso().expect("Failed to map VDSO");
    let example: ArgumentExample = get_shared();
    assert!(
        example.i == 0,
        "Expected get_shared() to return 0, got {}",
        example.i
    );
    set_shared(1);
    let example: ArgumentExample = get_shared();
    assert!(
        example.i == 1,
        "Expected get_shared() to return 1, got {}",
        example.i
    );
    let example: ArgumentExample = get_private();
    assert!(
        example.i == 0,
        "Expected get_shared() to return 1, got {}",
        example.i
    );
    set_private(1);
    let example: ArgumentExample = get_private();
    assert!(
        example.i == 1,
        "Expected get_shared() to return 1, got {}",
        example.i
    );

    assert_eq!(test_args(Some(1), Ok(2), (3, 4)), (Some(2), Ok(3), (4, 5)));
    println!("Test passed!");
    drop(map);
}
