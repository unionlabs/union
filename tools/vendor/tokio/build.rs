use autocfg::AutoCfg;

const CONST_THREAD_LOCAL_PROBE: &str = r#"
{
    thread_local! {
        static MY_PROBE: usize = const { 10 };
    }

    MY_PROBE.with(|val| *val)
}
"#;

fn main() {
    let mut enable_const_thread_local = false;

    match AutoCfg::new() {
        Ok(ac) => {
            // These checks prefer to call only `probe_rustc_version` if that is
            // enough to determine whether the feature is supported. This is
            // because the `probe_expression` call involves a call to rustc,
            // which the `probe_rustc_version` call avoids.

            // Const-initialized thread locals were stabilized in 1.59.
            if ac.probe_rustc_version(1, 60) {
                enable_const_thread_local = true;
            } else if ac.probe_rustc_version(1, 59) {
                // This compiler claims to be 1.59, but there are some nightly
                // compilers that claim to be 1.59 without supporting the
                // feature. Explicitly probe to check if code using them
                // compiles.
                //
                // The oldest nightly that supports the feature is 2021-12-06.
                if ac.probe_expression(CONST_THREAD_LOCAL_PROBE) {
                    enable_const_thread_local = true;
                }
            }
        }

        Err(e) => {
            // If we couldn't detect the compiler version and features, just
            // print a warning. This isn't a fatal error: we can still build
            // Tokio, we just can't enable cfgs automatically.
            println!(
                "cargo:warning=tokio: failed to detect compiler features: {}",
                e
            );
        }
    }

    if !enable_const_thread_local {
        // To disable this feature on compilers that support it, you can
        // explicitly pass this flag with the following environment variable:
        //
        // RUSTFLAGS="--cfg tokio_no_const_thread_local"
        autocfg::emit("tokio_no_const_thread_local")
    }
}
