use build_vdso::*;

fn main() {
    let config = BuildConfig::new("../vdso", "vdso");
    build_vdso(&config);
}
