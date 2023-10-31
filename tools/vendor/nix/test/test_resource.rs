#[cfg(not(any(target_os = "redox", target_os = "fuchsia", target_os = "illumos")))]
use nix::sys::resource::{getrlimit, setrlimit, Resource};

/// Tests the RLIMIT_NOFILE functionality of getrlimit(), where the resource RLIMIT_NOFILE refers
/// to the maximum file descriptor number that can be opened by the process (aka the maximum number
/// of file descriptors that the process can open, since Linux 4.5).
///
/// We first fetch the existing file descriptor maximum values using getrlimit(), then edit the
/// soft limit to make sure it has a new and distinct value to the hard limit. We then setrlimit()
/// to put the new soft limit in effect, and then getrlimit() once more to ensure the limits have
/// been updated.
#[test]
#[cfg(not(any(target_os = "redox", target_os = "fuchsia", target_os = "illumos")))]
pub fn test_resource_limits_nofile() {
    let (soft_limit, hard_limit) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();

    let soft_limit = Some(soft_limit.map_or(1024, |v| v - 1));
    assert_ne!(soft_limit, hard_limit);
    setrlimit(Resource::RLIMIT_NOFILE, soft_limit, hard_limit).unwrap();

    let (new_soft_limit, _) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();
    assert_eq!(new_soft_limit, soft_limit);
}
