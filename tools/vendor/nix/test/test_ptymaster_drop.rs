#[cfg(not(any(target_os = "redox", target_os = "fuchsia")))]
mod t {
    use nix::fcntl::OFlag;
    use nix::pty::*;
    use nix::unistd::close;
    use std::os::unix::io::AsRawFd;

    /// Regression test for Issue #659
    ///
    /// `PtyMaster` should panic rather than double close the file descriptor
    /// This must run in its own test process because it deliberately creates a
    /// race condition.
    #[test]
    #[should_panic(expected = "Closing an invalid file descriptor!")]
    fn test_double_close() {
        let m = posix_openpt(OFlag::O_RDWR).unwrap();
        close(m.as_raw_fd()).unwrap();
        drop(m);            // should panic here
    }
}
