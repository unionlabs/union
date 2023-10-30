// Empty `build.rs` so that `[package] links = ...` works in `Cargo.toml`.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
