#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "linux",
    target_os = "android",
    target_os = "emscripten",
))]
use nix::time::clock_getcpuclockid;
use nix::time::{clock_gettime, ClockId};

#[cfg(not(target_os = "redox"))]
#[test]
pub fn test_clock_getres() {
    assert!(nix::time::clock_getres(ClockId::CLOCK_REALTIME).is_ok());
}

#[test]
pub fn test_clock_gettime() {
    assert!(clock_gettime(ClockId::CLOCK_REALTIME).is_ok());
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "linux",
    target_os = "android",
    target_os = "emscripten",
))]
#[test]
pub fn test_clock_getcpuclockid() {
    let clock_id = clock_getcpuclockid(nix::unistd::Pid::this()).unwrap();
    assert!(clock_gettime(clock_id).is_ok());
}

#[cfg(not(target_os = "redox"))]
#[test]
pub fn test_clock_id_res() {
    assert!(ClockId::CLOCK_REALTIME.res().is_ok());
}

#[test]
pub fn test_clock_id_now() {
    assert!(ClockId::CLOCK_REALTIME.now().is_ok());
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "linux",
    target_os = "android",
    target_os = "emscripten",
))]
#[test]
pub fn test_clock_id_pid_cpu_clock_id() {
    assert!(ClockId::pid_cpu_clock_id(nix::unistd::Pid::this())
        .map(ClockId::now)
        .is_ok());
}
