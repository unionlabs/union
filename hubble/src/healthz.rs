use std::sync::atomic::{AtomicBool, Ordering};

use axum::http::StatusCode;

static GLOBAL_FLAG: AtomicBool = AtomicBool::new(true);

pub fn set_unhealthy() {
    GLOBAL_FLAG.store(false, Ordering::SeqCst);
}

/// Handler for a very simple healthcheck of the system. Mainly useful for
/// catching deadlocks as Hubble spawns many asynchronous tasks.
pub async fn handler() -> StatusCode {
    if GLOBAL_FLAG.load(Ordering::Relaxed) {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
