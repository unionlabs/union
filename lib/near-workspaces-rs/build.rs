fn main() {
    println!("cargo:rerun-if-env-changed=NEAR_SANDBOX_BIN_PATH");

    let doc_build = cfg!(doc) || std::env::var("DOCS_RS").is_ok();
    let env_bin = std::env::var("NEAR_SANDBOX_BIN_PATH").is_ok();
    if !doc_build && !env_bin && cfg!(feature = "install") {
        near_sandbox_utils::install().expect("Could not install sandbox");
    }
}
