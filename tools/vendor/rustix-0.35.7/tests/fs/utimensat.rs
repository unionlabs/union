#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[test]
fn test_utimensat() {
    use rustix::fs::{cwd, openat, statat, utimensat, AtFlags, Mode, OFlags, Timespec, Timestamps};

    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(
        cwd(),
        tmp.path(),
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();

    let _ = openat(
        &dir,
        "foo",
        OFlags::CREATE | OFlags::WRONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();

    let times = Timestamps {
        last_access: Timespec {
            tv_sec: 44000,
            tv_nsec: 45000,
        },
        last_modification: Timespec {
            tv_sec: 46000,
            tv_nsec: 47000,
        },
    };
    utimensat(&dir, "foo", &times, AtFlags::empty()).unwrap();

    let after = statat(&dir, "foo", AtFlags::empty()).unwrap();

    assert_eq!(times.last_modification.tv_sec as u64, after.st_mtime as u64);
    #[cfg(not(target_os = "netbsd"))]
    assert_eq!(
        times.last_modification.tv_nsec as u64,
        after.st_mtime_nsec as u64
    );
    #[cfg(target_os = "netbsd")]
    assert_eq!(
        times.last_modification.tv_nsec as u64,
        after.st_mtimensec as u64
    );
    assert!(times.last_access.tv_sec as u64 >= after.st_atime as u64);
    #[cfg(not(target_os = "netbsd"))]
    assert!(
        times.last_access.tv_sec as u64 > after.st_atime as u64
            || times.last_access.tv_nsec as u64 >= after.st_atime_nsec as u64
    );
    #[cfg(target_os = "netbsd")]
    assert!(
        times.last_access.tv_sec as u64 > after.st_atime as u64
            || times.last_access.tv_nsec as u64 >= after.st_atimensec as u64
    );
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[test]
fn test_utimensat_noent() {
    use rustix::fs::{cwd, openat, utimensat, AtFlags, Mode, OFlags, Timespec, Timestamps};

    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(
        cwd(),
        tmp.path(),
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();

    let times = Timestamps {
        last_access: Timespec {
            tv_sec: 44000,
            tv_nsec: 45000,
        },
        last_modification: Timespec {
            tv_sec: 46000,
            tv_nsec: 47000,
        },
    };
    assert_eq!(
        utimensat(&dir, "foo", &times, AtFlags::empty()).unwrap_err(),
        rustix::io::Errno::NOENT
    );
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[test]
fn test_utimensat_notdir() {
    use rustix::fs::{cwd, openat, utimensat, AtFlags, Mode, OFlags, Timespec, Timestamps};

    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(
        cwd(),
        tmp.path(),
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();

    let foo = openat(
        &dir,
        "foo",
        OFlags::CREATE | OFlags::WRONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();

    let times = Timestamps {
        last_access: Timespec {
            tv_sec: 44000,
            tv_nsec: 45000,
        },
        last_modification: Timespec {
            tv_sec: 46000,
            tv_nsec: 47000,
        },
    };
    assert_eq!(
        utimensat(&foo, "bar", &times, AtFlags::empty()).unwrap_err(),
        rustix::io::Errno::NOTDIR
    );
}
