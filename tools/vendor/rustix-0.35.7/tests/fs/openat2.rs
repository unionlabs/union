use rustix::fd::AsFd;
use rustix::fs::{cwd, mkdirat, openat, openat2, symlinkat, Mode, OFlags, ResolveFlags};
use rustix::io::OwnedFd;
use rustix::{io, path};
use std::os::unix::io::AsRawFd;

/// Like `openat2`, but keep retrying until it fails or succeeds.
fn openat2_more<Fd: AsFd, P: path::Arg>(
    dirfd: Fd,
    path: P,
    oflags: OFlags,
    mode: Mode,
    resolve: ResolveFlags,
) -> io::Result<OwnedFd> {
    let path = path.as_cow_c_str().unwrap().into_owned();
    loop {
        match openat2(dirfd.as_fd(), &path, oflags, mode, resolve) {
            Ok(file) => return Ok(file),
            Err(io::Errno::AGAIN) => continue,
            Err(err) => return Err(err),
        }
    }
}

#[test]
fn test_openat2() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(cwd(), tmp.path(), OFlags::RDONLY, Mode::empty()).unwrap();

    // Detect whether `openat2` is available.
    match openat2(
        &dir,
        ".",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::empty(),
    ) {
        Ok(_file) => (),
        Err(io::Errno::NOSYS) => return,
        Err(_err) => return,
    }

    // Create a file.
    let _ = openat2_more(
        &dir,
        "test.txt",
        OFlags::WRONLY | OFlags::CREATE | OFlags::TRUNC | OFlags::CLOEXEC,
        Mode::RUSR,
        ResolveFlags::empty(),
    )
    .unwrap();

    // Test `NO_SYMLINKS`.
    symlinkat("test.txt", &dir, "symlink.txt").unwrap();
    let _ = openat2_more(
        &dir,
        "symlink.txt",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::empty(),
    )
    .unwrap();
    let _ = openat2_more(
        &dir,
        "symlink.txt",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_MAGICLINKS,
    )
    .unwrap();
    let _ = openat2_more(
        &dir,
        "symlink.txt",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS,
    )
    .unwrap_err();

    // Test `NO_MAGICLINKS`.
    let test = openat2_more(
        &dir,
        "test.txt",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::empty(),
    )
    .unwrap();
    let _ = openat2_more(
        &dir,
        &format!("/proc/self/fd/{}", test.as_fd().as_raw_fd()),
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::empty(),
    )
    .unwrap();
    let _ = openat2_more(
        &dir,
        &format!("/proc/self/fd/{}", test.as_fd().as_raw_fd()),
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS,
    )
    .unwrap_err();
    let _ = openat2_more(
        &dir,
        &format!("/proc/self/fd/{}", test.as_fd().as_raw_fd()),
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_MAGICLINKS,
    )
    .unwrap_err();

    // Test `NO_XDEV`.
    let root = openat2_more(
        &dir,
        "/",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::empty(),
    )
    .unwrap();
    let _ = openat2_more(
        &root,
        "proc",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::empty(),
    )
    .unwrap();
    let _ = openat2_more(
        &root,
        "proc",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_XDEV,
    )
    .unwrap_err();

    // Test `BENEATH`.
    let _ = openat2_more(
        &dir,
        "..",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::empty(),
    )
    .unwrap();
    let _ = openat2_more(
        &dir,
        "..",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH,
    )
    .unwrap_err();

    // Test `IN_ROOT`.
    let _ = openat2_more(
        &dir,
        "/proc",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::empty(),
    )
    .unwrap();
    let _ = openat2_more(
        &dir,
        "/proc",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::IN_ROOT,
    )
    .unwrap_err();
    mkdirat(&dir, "proc", Mode::RUSR | Mode::XUSR).unwrap();
    let _ = openat2_more(
        &dir,
        "/proc",
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::IN_ROOT,
    )
    .unwrap();
}
