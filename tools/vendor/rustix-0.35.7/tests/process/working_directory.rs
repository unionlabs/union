#![cfg(feature = "fs")]

#[cfg(not(any(target_os = "fuchsia", target_os = "macos")))]
use rustix::fs::{Mode, OFlags};
use tempfile::{tempdir, TempDir};

#[allow(unused)]
fn tmpdir() -> TempDir {
    tempdir().expect("expected to be able to create a temporary directory")
}

/// Disable this test on macos because GHA has a weird system folder structure
/// that makes this test fail.
#[cfg(not(target_os = "macos"))]
#[test]
fn test_changing_working_directory() {
    let tmpdir = tmpdir();

    let orig_cwd = rustix::process::getcwd(Vec::new()).expect("get the cwd");

    #[cfg(not(target_os = "fuchsia"))]
    let orig_fd_cwd = rustix::fs::openat(rustix::fs::cwd(), ".", OFlags::RDONLY, Mode::empty())
        .expect("get a fd for the current directory");

    rustix::process::chdir(tmpdir.path()).expect("changing dir to the tmp");
    let ch1_cwd = rustix::process::getcwd(Vec::new()).expect("get the cwd");

    assert_ne!(orig_cwd, ch1_cwd, "The cwd hasn't changed!");
    assert_eq!(
        ch1_cwd.to_string_lossy(),
        tmpdir.path().to_string_lossy(),
        "The cwd is not the same as the tmpdir"
    );

    #[cfg(not(target_os = "fuchsia"))]
    rustix::process::fchdir(orig_fd_cwd).expect("changing dir to the original");
    #[cfg(target_os = "fushcia")]
    rustix::process::chdir(orig_cwd).expect("changing dir to the original");
    let ch2_cwd = rustix::process::getcwd(ch1_cwd).expect("get the cwd");

    assert_eq!(
        orig_cwd, ch2_cwd,
        "The cwd wasn't changed back to the its original position"
    );
}
