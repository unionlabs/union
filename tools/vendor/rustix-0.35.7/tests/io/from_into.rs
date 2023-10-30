#[cfg(feature = "fs")]
#[cfg(not(target_os = "redox"))]
#[test]
fn test_owned() {
    use rustix::fd::AsFd;
    #[cfg(unix)]
    use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
    #[cfg(target_os = "wasi")]
    use std::os::wasi::io::{AsRawFd, FromRawFd, IntoRawFd};

    let file = rustix::fs::openat(
        rustix::fs::cwd(),
        "Cargo.toml",
        rustix::fs::OFlags::RDONLY,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    let raw = file.as_raw_fd();
    assert_eq!(raw, file.as_fd().as_raw_fd());

    let inner = file.into_raw_fd();
    assert_eq!(raw, inner);

    let new = unsafe { rustix::io::OwnedFd::from_raw_fd(inner) };
    let mut buf = [0_u8; 4];
    let _ = rustix::io::read(&new, &mut buf).unwrap();
}
