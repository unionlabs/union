#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[test]
fn test_futimens() {
    use rustix::fs::{cwd, fstat, futimens, openat, Mode, OFlags, Timespec, Timestamps};

    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(cwd(), tmp.path(), OFlags::RDONLY, Mode::empty()).unwrap();

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
    futimens(&foo, &times).unwrap();

    let after = fstat(&foo).unwrap();

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
}
