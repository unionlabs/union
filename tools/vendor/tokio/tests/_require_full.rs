#![cfg(not(any(feature = "full", target_arch = "wasm32")))]
compile_error!("run main Tokio tests with `--features full`");
