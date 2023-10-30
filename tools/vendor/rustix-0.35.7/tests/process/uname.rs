#[test]
fn test_uname() {
    let name: rustix::process::Uname = rustix::process::uname();

    assert!(!name.sysname().to_bytes().is_empty());
    assert!(!name.nodename().to_bytes().is_empty());
    assert!(!name.release().to_bytes().is_empty());
    assert!(!name.version().to_bytes().is_empty());
    assert!(!name.machine().to_bytes().is_empty());

    #[cfg(any(target_os = "android", target_os = "linux"))]
    assert!(!name.domainname().to_bytes().is_empty());
}
