#[test]
fn test_timespec_layout() {
    #[cfg(not(target_os = "redox"))]
    use rustix::fs::{UTIME_NOW, UTIME_OMIT};
    use rustix::time::{Nsecs, Secs, Timespec};

    let tv_sec: Secs = 0;
    let tv_nsec: Nsecs = 0;
    let _ = Timespec { tv_sec, tv_nsec };

    #[cfg(not(target_os = "redox"))]
    let _ = Timespec {
        tv_sec,
        tv_nsec: UTIME_NOW,
    };
    #[cfg(not(target_os = "redox"))]
    let _ = Timespec {
        tv_sec,
        tv_nsec: UTIME_OMIT,
    };
    let _ = Timespec { tv_sec, tv_nsec: 0 };
    let _ = Timespec {
        tv_sec,
        tv_nsec: 999_999_999,
    };
}
