#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[test]
fn test_fcntl_dupfd_cloexec() {
    use rustix::fd::AsFd;
    use std::os::unix::io::AsRawFd;

    let file = rustix::fs::openat(
        rustix::fs::cwd(),
        "Cargo.toml",
        rustix::fs::OFlags::RDONLY,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    let new = rustix::fs::fcntl_dupfd_cloexec(&file, 700).unwrap();
    assert_eq!(new.as_fd().as_raw_fd(), 700);
}
