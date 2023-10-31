use rustix::process::nice;
#[cfg(not(target_os = "redox"))]
use rustix::process::{getpriority_process, setpriority_process};

#[cfg(not(target_os = "freebsd"))] // FreeBSD's nice(3) doesn't return the old value.
#[test]
fn test_priorities() {
    let old = nice(0).unwrap();

    #[cfg(not(target_os = "redox"))]
    {
        let get_prio = getpriority_process(None).unwrap();
        assert_eq!(get_prio, old);
    }

    // Lower the priority by one.
    let new = nice(1).unwrap();

    // If the test wasn't running with the lowest priority initially, test that
    // we were able to lower the priority.
    if old < 19 {
        assert_eq!(old + 1, new);
    }

    let get = nice(0).unwrap();
    assert_eq!(new, get);

    #[cfg(not(target_os = "redox"))]
    {
        let get_prio = getpriority_process(None).unwrap();
        assert_eq!(get_prio, new);

        setpriority_process(None, get + 1).unwrap();
        let now = getpriority_process(None).unwrap();

        // If the test wasn't running with the lowest priority initially, test
        // that we were able to lower the priority.
        if get < 19 {
            assert_eq!(get + 1, now);
        }
        setpriority_process(None, get + 10000).unwrap();
        let now = getpriority_process(None).unwrap();
        // Linux's max is 19; Darwin's max is 20.
        assert!(now >= 19 && now <= 20);
        // Darwin appears to return `EPERM` on an out of range `nice`.
        if let Ok(again) = nice(1) {
            assert_eq!(now, again);
        }
    }
}

/// FreeBSD's `nice` doesn't return the new nice value, so use a specialized
/// test.
#[cfg(target_os = "freebsd")]
#[test]
fn test_priorities() {
    let start = getpriority_process(None).unwrap();

    let _ = nice(0).unwrap();

    let now = getpriority_process(None).unwrap();
    assert_eq!(start, now);

    let _ = nice(1).unwrap();

    let now = getpriority_process(None).unwrap();
    assert_eq!(start + 1, now);

    setpriority_process(None, start + 2).unwrap();

    let now = getpriority_process(None).unwrap();
    assert_eq!(start + 2, now);

    setpriority_process(None, 10000).unwrap();

    let now = getpriority_process(None).unwrap();
    assert_eq!(now, 20);

    let _ = nice(1).unwrap();

    let now = getpriority_process(None).unwrap();
    assert_eq!(now, 20);
}
