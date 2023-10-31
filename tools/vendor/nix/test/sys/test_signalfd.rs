use std::convert::TryFrom;

#[test]
fn test_signalfd() {
    use nix::sys::signalfd::SignalFd;
    use nix::sys::signal::{self, raise, Signal, SigSet};

    // Grab the mutex for altering signals so we don't interfere with other tests.
    let _m = crate::SIGNAL_MTX.lock();

    // Block the SIGUSR1 signal from automatic processing for this thread
    let mut mask = SigSet::empty();
    mask.add(signal::SIGUSR1);
    mask.thread_block().unwrap();

    let mut fd = SignalFd::new(&mask).unwrap();

    // Send a SIGUSR1 signal to the current process. Note that this uses `raise` instead of `kill`
    // because `kill` with `getpid` isn't correct during multi-threaded execution like during a
    // cargo test session. Instead use `raise` which does the correct thing by default.
    raise(signal::SIGUSR1).expect("Error: raise(SIGUSR1) failed");

    // And now catch that same signal.
    let res = fd.read_signal().unwrap().unwrap();
    let signo = Signal::try_from(res.ssi_signo as i32).unwrap();
    assert_eq!(signo, signal::SIGUSR1);
}
