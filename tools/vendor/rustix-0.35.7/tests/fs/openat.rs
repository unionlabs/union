use std::fs::File;

use io_lifetimes::{FromFd, IntoFd};
use rustix::fs::{cwd, openat, Mode, OFlags};
use std::io::Write;

#[test]
fn test_openat_tmpfile() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(
        cwd(),
        tmp.path(),
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    let f = match openat(
        &dir,
        ".",
        OFlags::WRONLY | OFlags::CLOEXEC | OFlags::TMPFILE,
        Mode::from_bits_truncate(0o644),
    ) {
        Ok(f) => Ok(Some(File::from_fd(f.into_fd()))),
        // TODO: Factor out the `Err`, once we no longer support Rust 1.48.
        Err(rustix::io::Errno::OPNOTSUPP)
        | Err(rustix::io::Errno::ISDIR)
        | Err(rustix::io::Errno::NOENT) => Ok(None),
        Err(e) => Err(e),
    };
    if let Some(mut f) = f.unwrap() {
        write!(f, "hello world").unwrap();
    }
}
