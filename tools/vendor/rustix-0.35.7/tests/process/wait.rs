use libc::{kill, SIGSTOP};
use rustix::process;
use serial_test::serial;
use std::process::{Command, Stdio};

// These tests must execute serially to prevent race condition, where
// `test_wait` waits for the child process spawned in `test_waitpid`, causing
// the tests to get stuck.

#[test]
#[serial]
fn test_waitpid() {
    let child = Command::new("yes")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to execute child");
    unsafe { kill(child.id() as _, SIGSTOP) };

    let pid = unsafe { process::Pid::from_raw(child.id() as _) };
    let status = process::waitpid(pid, process::WaitOptions::UNTRACED)
        .expect("failed to wait")
        .unwrap();
    assert!(status.stopped());
}
