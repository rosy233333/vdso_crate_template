use crate::map::map_vdso;
use api::*;

mod map;

fn main() {
    env_logger::init();
    log::info!("Starting VDSO test...");
    let map = map_vdso().expect("Failed to map VDSO");
    init();
    let example: ArgumentExample = api_example();
    assert!(
        example.i == 42,
        "Expected api_example() to return 42, got {}",
        example.i
    );
    println!("Test passed!");
    drop(map);
}
