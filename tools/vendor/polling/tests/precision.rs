use std::io;
use std::time::{Duration, Instant};

use polling::Poller;

#[test]
fn below_ms() -> io::Result<()> {
    let poller = Poller::new()?;
    let mut events = Vec::new();

    let dur = Duration::from_micros(100);
    let margin = Duration::from_micros(500);
    let mut lowest = Duration::from_secs(1000);

    for _ in 0..1_000 {
        let now = Instant::now();
        let n = poller.wait(&mut events, Some(dur))?;
        let elapsed = now.elapsed();

        assert_eq!(n, 0);
        assert!(elapsed >= dur);
        lowest = lowest.min(elapsed);
    }

    if cfg!(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly",
    )) {
        assert!(lowest < dur + margin);
    }
    Ok(())
}

#[test]
fn above_ms() -> io::Result<()> {
    let poller = Poller::new()?;
    let mut events = Vec::new();

    let dur = Duration::from_micros(3_100);
    let margin = Duration::from_micros(500);
    let mut lowest = Duration::from_secs(1000);

    for _ in 0..1_000 {
        let now = Instant::now();
        let n = poller.wait(&mut events, Some(dur))?;
        let elapsed = now.elapsed();

        assert_eq!(n, 0);
        assert!(elapsed >= dur);
        lowest = lowest.min(elapsed);
    }

    if cfg!(any(
        target_os = "linux",
        target_os = "android",
        target_os = "illumos",
        target_os = "solaris",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly",
    )) {
        assert!(lowest < dur + margin);
    }
    Ok(())
}
