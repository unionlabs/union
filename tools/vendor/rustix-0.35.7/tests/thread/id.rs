use rustix::thread;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[test]
fn test_gettid() {
    assert_eq!(thread::gettid(), thread::gettid());
}
