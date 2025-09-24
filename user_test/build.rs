use build_vdso::*;

fn main() {
    let mut config = BuildConfig::new("../vdso", "vdso");
    config.out_dir = String::from("../output");
    build_vdso(&config);
}
