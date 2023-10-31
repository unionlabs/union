#[cfg(not(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
    target_os = "wasi",
)))]
#[test]
fn test_mknodat() {
    use rustix::fs::{cwd, mknodat, openat, statat, unlinkat, AtFlags, FileType, Mode, OFlags};

    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(cwd(), tmp.path(), OFlags::RDONLY, Mode::empty()).unwrap();

    // Create a regular file. Not supported on FreeBSD or OpenBSD.
    #[cfg(not(any(target_os = "freebsd", target_os = "openbsd")))]
    {
        mknodat(&dir, "foo", FileType::RegularFile, Mode::empty(), 0).unwrap();
        let stat = statat(&dir, "foo", AtFlags::empty()).unwrap();
        assert_eq!(FileType::from_raw_mode(stat.st_mode), FileType::RegularFile);
        unlinkat(&dir, "foo", AtFlags::empty()).unwrap();
    }

    mknodat(&dir, "foo", FileType::Fifo, Mode::empty(), 0).unwrap();
    let stat = statat(&dir, "foo", AtFlags::empty()).unwrap();
    assert_eq!(FileType::from_raw_mode(stat.st_mode), FileType::Fifo);
    unlinkat(&dir, "foo", AtFlags::empty()).unwrap();
}
