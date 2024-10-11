use rustc_version::{version_meta, Channel};

fn main() {
    let version_meta = version_meta().unwrap();

    // To use nightly features we declare this and then we can use
    // #[cfg(nightly)]
    // for nightly only features
    if version_meta.channel == Channel::Nightly {
        println!("cargo:rustc-cfg=nightly")
    }
}
