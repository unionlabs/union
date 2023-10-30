/// Test that we can set a file timestamp to a date past the year 2038 with
/// `utimensat` and read it back again.
///
/// See tests/time/y2038.rs for more information about y2038 testing.
#[cfg(not(all(target_env = "musl", target_pointer_width = "32")))]
#[cfg(not(all(target_os = "android", target_pointer_width = "32")))]
#[cfg(not(all(target_os = "emscripten", target_pointer_width = "32")))]
#[test]
fn test_y2038_with_utimensat() {
    use rustix::fs::{
        cwd, fstat, openat, statat, utimensat, AtFlags, Mode, OFlags, Timespec, Timestamps,
    };
    use std::convert::TryInto;

    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(&cwd(), tmp.path(), OFlags::RDONLY, Mode::empty()).unwrap();

    let m_sec = 1_u64 << 32;
    let m_nsec = 17_u32;
    let a_sec = m_sec + 1;
    let a_nsec = m_nsec + 1;

    let timestamps = Timestamps {
        last_modification: Timespec {
            tv_sec: m_sec as _,
            tv_nsec: m_nsec as _,
        },
        last_access: Timespec {
            tv_sec: a_sec as _,
            tv_nsec: a_nsec as _,
        },
    };
    let _ = openat(&dir, "foo", OFlags::CREATE | OFlags::WRONLY, Mode::RUSR).unwrap();

    match utimensat(&dir, "foo", &timestamps, AtFlags::empty()) {
        Ok(()) => (),

        // On 32-bit platforms, accept `EOVERFLOW`, meaning that y2038 support
        // is not available in this version of the OS.
        #[cfg(target_pointer_width = "32")]
        Err(rustix::io::Errno::OVERFLOW) => return,

        Err(e) => panic!("unexpected error: {:?}", e),
    }

    // Use `statat` to read back the timestamp.
    let stat = statat(&dir, "foo", AtFlags::empty()).unwrap();

    assert_eq!(
        TryInto::<u64>::try_into(stat.st_mtime).unwrap() as u64,
        m_sec
    );
    assert_eq!(stat.st_mtime_nsec as u32, m_nsec);
    assert!(TryInto::<u64>::try_into(stat.st_atime).unwrap() as u64 >= a_sec);
    assert!(
        TryInto::<u64>::try_into(stat.st_atime).unwrap() as u64 > a_sec
            || stat.st_atime_nsec as u32 >= a_nsec
    );

    // Now test the same thing, but with `fstat`.
    let file = openat(&dir, "foo", OFlags::RDONLY, Mode::empty()).unwrap();
    let stat = fstat(&file).unwrap();

    assert_eq!(
        TryInto::<u64>::try_into(stat.st_mtime).unwrap() as u64,
        m_sec
    );
    assert_eq!(stat.st_mtime_nsec as u32, m_nsec);
    assert!(TryInto::<u64>::try_into(stat.st_atime).unwrap() as u64 >= a_sec);
    assert!(
        TryInto::<u64>::try_into(stat.st_atime).unwrap() as u64 > a_sec
            || stat.st_atime_nsec as u32 >= a_nsec
    );
}

/// Test that we can set a file timestamp to a date past the year 2038 with
/// `futimens` and read it back again.
///
/// See tests/time/y2038.rs for more information about y2038 testing.
#[cfg(not(all(target_env = "musl", target_pointer_width = "32")))]
#[cfg(not(all(target_os = "android", target_pointer_width = "32")))]
#[cfg(not(all(target_os = "emscripten", target_pointer_width = "32")))]
#[test]
fn test_y2038_with_futimens() {
    use rustix::fs::{
        cwd, fstat, futimens, openat, statat, AtFlags, Mode, OFlags, Timespec, Timestamps,
    };
    use std::convert::TryInto;

    let tmp = tempfile::tempdir().unwrap();
    let dir = openat(&cwd(), tmp.path(), OFlags::RDONLY, Mode::empty()).unwrap();

    let m_sec = 1_u64 << 32;
    let m_nsec = 17_u32;
    let a_sec = m_sec + 1;
    let a_nsec = m_nsec + 1;

    let timestamps = Timestamps {
        last_modification: Timespec {
            tv_sec: m_sec as _,
            tv_nsec: m_nsec as _,
        },
        last_access: Timespec {
            tv_sec: a_sec as _,
            tv_nsec: a_nsec as _,
        },
    };
    let file = openat(&dir, "foo", OFlags::CREATE | OFlags::WRONLY, Mode::RUSR).unwrap();

    match futimens(&file, &timestamps) {
        Ok(()) => (),

        // On 32-bit platforms, accept `EOVERFLOW`, meaning that y2038 support
        // is not available in this version of the OS.
        #[cfg(target_pointer_width = "32")]
        Err(rustix::io::Errno::OVERFLOW) => return,

        Err(e) => panic!("unexpected error: {:?}", e),
    }

    // Use `statat` to read back the timestamp.
    let stat = statat(&dir, "foo", AtFlags::empty()).unwrap();

    assert_eq!(TryInto::<u64>::try_into(stat.st_mtime).unwrap(), m_sec);
    assert_eq!(stat.st_mtime_nsec as u32, m_nsec);
    assert!(TryInto::<u64>::try_into(stat.st_atime).unwrap() >= a_sec);
    assert!(
        TryInto::<u64>::try_into(stat.st_atime).unwrap() > a_sec
            || stat.st_atime_nsec as u32 >= a_nsec
    );

    // Now test the same thing, but with `fstat`.
    let file = openat(&dir, "foo", OFlags::RDONLY, Mode::empty()).unwrap();
    let stat = fstat(&file).unwrap();

    assert_eq!(
        TryInto::<u64>::try_into(stat.st_mtime).unwrap() as u64,
        m_sec
    );
    assert_eq!(stat.st_mtime_nsec as u32, m_nsec);
    assert!(TryInto::<u64>::try_into(stat.st_atime).unwrap() as u64 >= a_sec);
    assert!(
        TryInto::<u64>::try_into(stat.st_atime).unwrap() as u64 > a_sec
            || stat.st_atime_nsec as u32 >= a_nsec
    );
}
