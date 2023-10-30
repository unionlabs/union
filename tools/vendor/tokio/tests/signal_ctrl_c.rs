#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]
#![cfg(unix)]

mod support {
    pub mod signal;
}
use support::signal::send_signal;

use tokio::signal;
use tokio::sync::oneshot;
use tokio_test::assert_ok;

#[tokio::test]
async fn ctrl_c() {
    let ctrl_c = signal::ctrl_c();

    let (fire, wait) = oneshot::channel();

    // NB: simulate a signal coming in by exercising our signal handler
    // to avoid complications with sending SIGINT to the test process
    tokio::spawn(async {
        wait.await.expect("wait failed");
        send_signal(libc::SIGINT);
    });

    let _ = fire.send(());

    assert_ok!(ctrl_c.await);
}
