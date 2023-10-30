#![cfg(feature = "std")]
use tracing_mock::*;

#[test]
fn scoped_clobbers_global() {
    // Reproduces https://github.com/tokio-rs/tracing/issues/2050

    let (scoped, scoped_handle) = subscriber::mock()
        .event(event::msg("before global"))
        .event(event::msg("before drop"))
        .done()
        .run_with_handle();

    let (global, global_handle) = subscriber::mock()
        .event(event::msg("after drop"))
        .done()
        .run_with_handle();

    // Set a scoped default subscriber, returning a guard.
    let guard = tracing::subscriber::set_default(scoped);
    tracing::info!("before global");

    // Now, set the global default.
    tracing::subscriber::set_global_default(global)
        .expect("global default should not already be set");
    // This event should still be collected by the scoped default.
    tracing::info!("before drop");

    // Drop the guard. Now, the global default subscriber should be used.
    drop(guard);
    tracing::info!("after drop");

    scoped_handle.assert_finished();
    global_handle.assert_finished();
}
