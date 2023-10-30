use nix::errno::Errno;
use nix::unistd::*;
use nix::unistd::ForkResult::*;
use nix::sys::signal::*;
use nix::sys::wait::*;
use libc::_exit;

#[test]
#[cfg(not(target_os = "redox"))]
fn test_wait_signal() {
    let _m = crate::FORK_MTX.lock();

    // Safe: The child only calls `pause` and/or `_exit`, which are async-signal-safe.
    match unsafe{fork()}.expect("Error: Fork Failed") {
      Child => {
          pause();
          unsafe { _exit(123) }
      },
      Parent { child } => {
          kill(child, Some(SIGKILL)).expect("Error: Kill Failed");
          assert_eq!(waitpid(child, None), Ok(WaitStatus::Signaled(child, SIGKILL, false)));
      },
    }
}

#[test]
fn test_wait_exit() {
    let _m = crate::FORK_MTX.lock();

    // Safe: Child only calls `_exit`, which is async-signal-safe.
    match unsafe{fork()}.expect("Error: Fork Failed") {
      Child => unsafe { _exit(12); },
      Parent { child } => {
          assert_eq!(waitpid(child, None), Ok(WaitStatus::Exited(child, 12)));
      },
    }
}

#[test]
fn test_waitstatus_from_raw() {
    let pid = Pid::from_raw(1);
    assert_eq!(WaitStatus::from_raw(pid, 0x0002), Ok(WaitStatus::Signaled(pid, Signal::SIGINT, false)));
    assert_eq!(WaitStatus::from_raw(pid, 0x0200), Ok(WaitStatus::Exited(pid, 2)));
    assert_eq!(WaitStatus::from_raw(pid, 0x7f7f), Err(Errno::EINVAL));
}

#[test]
fn test_waitstatus_pid() {
    let _m = crate::FORK_MTX.lock();

    match unsafe{fork()}.unwrap() {
        Child => unsafe { _exit(0) },
        Parent { child } => {
            let status = waitpid(child, None).unwrap();
            assert_eq!(status.pid(), Some(child));
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
// FIXME: qemu-user doesn't implement ptrace on most arches
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod ptrace {
    use nix::sys::ptrace::{self, Options, Event};
    use nix::sys::signal::*;
    use nix::sys::wait::*;
    use nix::unistd::*;
    use nix::unistd::ForkResult::*;
    use libc::_exit;
    use crate::*;

    fn ptrace_child() -> ! {
        ptrace::traceme().unwrap();
        // As recommended by ptrace(2), raise SIGTRAP to pause the child
        // until the parent is ready to continue
        raise(SIGTRAP).unwrap();
        unsafe { _exit(0) }
    }

    fn ptrace_parent(child: Pid) {
        // Wait for the raised SIGTRAP
        assert_eq!(waitpid(child, None), Ok(WaitStatus::Stopped(child, SIGTRAP)));
        // We want to test a syscall stop and a PTRACE_EVENT stop
        assert!(ptrace::setoptions(child, Options::PTRACE_O_TRACESYSGOOD | Options::PTRACE_O_TRACEEXIT).is_ok());

        // First, stop on the next system call, which will be exit()
        assert!(ptrace::syscall(child, None).is_ok());
        assert_eq!(waitpid(child, None), Ok(WaitStatus::PtraceSyscall(child)));
        // Then get the ptrace event for the process exiting
        assert!(ptrace::cont(child, None).is_ok());
        assert_eq!(waitpid(child, None), Ok(WaitStatus::PtraceEvent(child, SIGTRAP, Event::PTRACE_EVENT_EXIT as i32)));
        // Finally get the normal wait() result, now that the process has exited
        assert!(ptrace::cont(child, None).is_ok());
        assert_eq!(waitpid(child, None), Ok(WaitStatus::Exited(child, 0)));
    }

    #[test]
    fn test_wait_ptrace() {
        require_capability!("test_wait_ptrace", CAP_SYS_PTRACE);
        let _m = crate::FORK_MTX.lock();

        match unsafe{fork()}.expect("Error: Fork Failed") {
            Child => ptrace_child(),
            Parent { child } => ptrace_parent(child),
        }
    }
}
