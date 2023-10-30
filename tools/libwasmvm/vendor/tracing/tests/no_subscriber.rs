#![cfg(feature = "std")]

use tracing::subscriber::{self, NoSubscriber};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn no_subscriber_disables_global() {
    // Reproduces https://github.com/tokio-rs/tracing/issues/1999
    let (subscriber, handle) = tracing_mock::subscriber::mock().done().run_with_handle();
    subscriber::set_global_default(subscriber).expect("setting global default must succeed");
    subscriber::with_default(NoSubscriber::default(), || {
        tracing::info!("this should not be recorded");
    });
    handle.assert_finished();
}
