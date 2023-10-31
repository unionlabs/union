use nix::{
    errno::Errno,
    poll::{PollFlags, poll, PollFd},
    unistd::{write, pipe}
};

macro_rules! loop_while_eintr {
    ($poll_expr: expr) => {
        loop {
            match $poll_expr {
                Ok(nfds) => break nfds,
                Err(Errno::EINTR) => (),
                Err(e) => panic!("{}", e)
            }
        }
    }
}

#[test]
fn test_poll() {
    let (r, w) = pipe().unwrap();
    let mut fds = [PollFd::new(r, PollFlags::POLLIN)];

    // Poll an idle pipe.  Should timeout
    let nfds = loop_while_eintr!(poll(&mut fds, 100));
    assert_eq!(nfds, 0);
    assert!(!fds[0].revents().unwrap().contains(PollFlags::POLLIN));

    write(w, b".").unwrap();

    // Poll a readable pipe.  Should return an event.
    let nfds = poll(&mut fds, 100).unwrap();
    assert_eq!(nfds, 1);
    assert!(fds[0].revents().unwrap().contains(PollFlags::POLLIN));
}

// ppoll(2) is the same as poll except for how it handles timeouts and signals.
// Repeating the test for poll(2) should be sufficient to check that our
// bindings are correct.
#[cfg(any(target_os = "android",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "linux"))]
#[test]
fn test_ppoll() {
    use nix::poll::ppoll;
    use nix::sys::signal::SigSet;
    use nix::sys::time::{TimeSpec, TimeValLike};

    let timeout = TimeSpec::milliseconds(1);
    let (r, w) = pipe().unwrap();
    let mut fds = [PollFd::new(r, PollFlags::POLLIN)];

    // Poll an idle pipe.  Should timeout
    let sigset = SigSet::empty();
    let nfds = loop_while_eintr!(ppoll(&mut fds, Some(timeout), sigset));
    assert_eq!(nfds, 0);
    assert!(!fds[0].revents().unwrap().contains(PollFlags::POLLIN));

    write(w, b".").unwrap();

    // Poll a readable pipe.  Should return an event.
    let nfds = ppoll(&mut fds, Some(timeout), SigSet::empty()).unwrap();
    assert_eq!(nfds, 1);
    assert!(fds[0].revents().unwrap().contains(PollFlags::POLLIN));
}

#[test]
fn test_pollfd_fd() {
    use std::os::unix::io::AsRawFd;

    let pfd = PollFd::new(0x1234, PollFlags::empty());
    assert_eq!(pfd.as_raw_fd(), 0x1234);
}

#[test]
fn test_pollfd_events() {
    let mut pfd = PollFd::new(-1, PollFlags::POLLIN);
    assert_eq!(pfd.events(), PollFlags::POLLIN);
    pfd.set_events(PollFlags::POLLOUT);
    assert_eq!(pfd.events(), PollFlags::POLLOUT);
}
