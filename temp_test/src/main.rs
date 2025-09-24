use std::hint::black_box;

fn main() {
    let mut config = build_vdso::build_config::BuildConfig::new("../vdso", "vdso");
    config.out_dir = String::from("../output");
    build_vdso::build_vdso(&config);

    let data = vdso::VvarData::default();
    black_box(data);
    println!("aaaaaaaa");
}
