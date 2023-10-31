use rustix::process;

#[test]
fn test_getuid() {
    assert_eq!(process::getuid(), process::getuid());
    unsafe {
        assert_eq!(process::getuid().as_raw(), libc::getuid());
        assert_eq!(process::getuid().is_root(), libc::getuid() == 0);
    }
}

#[test]
fn test_getgid() {
    assert_eq!(process::getgid(), process::getgid());
    unsafe {
        assert_eq!(process::getgid().as_raw(), libc::getgid());
        assert_eq!(process::getgid().is_root(), libc::getgid() == 0);
    }
}

#[test]
fn test_geteuid() {
    assert_eq!(process::geteuid(), process::geteuid());
    unsafe {
        assert_eq!(process::geteuid().as_raw(), libc::geteuid());
        assert_eq!(process::geteuid().is_root(), libc::geteuid() == 0);
    }
}

#[test]
fn test_getegid() {
    assert_eq!(process::getegid(), process::getegid());
    unsafe {
        assert_eq!(process::getegid().as_raw(), libc::getegid());
        assert_eq!(process::getegid().is_root(), libc::getegid() == 0);
    }
}

#[test]
fn test_getpid() {
    assert_eq!(process::getpid(), process::getpid());
    unsafe {
        assert_eq!(
            process::getpid().as_raw_nonzero().get() as libc::pid_t,
            libc::getpid()
        );
        assert_eq!(process::getpid().is_init(), libc::getpid() == 1);
    }
}

#[test]
fn test_getppid() {
    assert_eq!(process::getppid(), process::getppid());
    unsafe {
        assert_eq!(
            process::Pid::as_raw(process::getppid()) as libc::pid_t,
            libc::getppid()
        );
        if let Some(ppid) = process::getppid() {
            assert_eq!(ppid.is_init(), libc::getppid() == 1);
        } else {
            assert_eq!(libc::getppid(), 0);
        }
    }
}
