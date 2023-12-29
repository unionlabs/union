fn main() {
    let cfg = autocfg::new();

    if cfg.probe_rustc_version(1, 61) {
        println!("cargo:rustc-cfg=has_const_fn_trait_bound");
    }
}
