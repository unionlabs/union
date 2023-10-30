fn main() {
    let ac = autocfg::new();

    #[cfg(unix)]
    ac.emit_trait_cfg("std::os::fd::AsFd", "fd");
    #[cfg(windows)]
    ac.emit_trait_cfg("std::os::windows::io::AsHandle", "fd");

    autocfg::rerun_path("build.rs");
}
