#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]
#![cfg(unix)]

mod support {
    pub mod signal;
}
use support::signal::send_signal;

use tokio::signal::unix::{signal, SignalKind};

#[tokio::test]
async fn notify_both() {
    let kind = SignalKind::user_defined2();

    let mut signal1 = signal(kind).expect("failed to create signal1");
    let mut signal2 = signal(kind).expect("failed to create signal2");

    send_signal(libc::SIGUSR2);

    signal1.recv().await;
    signal2.recv().await;
}
