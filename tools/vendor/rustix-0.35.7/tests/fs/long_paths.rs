#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[test]
fn test_long_paths() {
    use rustix::fs::{cwd, mkdirat, openat, Mode, OFlags};

    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(cwd(), tmp.path(), OFlags::RDONLY, Mode::empty()).unwrap();

    #[cfg(libc)]
    const PATH_MAX: usize = libc::PATH_MAX as usize;
    #[cfg(linux_raw)]
    const PATH_MAX: usize = linux_raw_sys::general::PATH_MAX as usize;

    mkdirat(&dir, "a", Mode::RUSR | Mode::XUSR | Mode::WUSR).unwrap();

    let mut long_path = String::new();
    for _ in 0..PATH_MAX / 5 {
        long_path.push_str("a/../");
    }

    let mut too_long_path = String::new();
    for _ in 0..PATH_MAX / 4 {
        too_long_path.push_str("a/../");
    }

    let _ = openat(&dir, &long_path, OFlags::RDONLY, Mode::empty()).unwrap();
    let _ = openat(&dir, &too_long_path, OFlags::RDONLY, Mode::empty()).unwrap_err();
}
