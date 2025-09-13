fn main() {
    let config = build_vdso::build_config::BuildConfig::new("../vdso", "vdso");
    build_vdso::build_vdso(&config);
}
