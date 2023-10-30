#[cfg(any(target_os = "android", target_os = "linux"))]
#[test]
fn test_eventfd() {
    use rustix::io::{eventfd, read, write, EventfdFlags};
    use std::mem::size_of;
    use std::thread;

    let efd = eventfd(0, EventfdFlags::CLOEXEC).unwrap();

    let child = thread::spawn(move || {
        for u in [1_u64, 3, 6, 11, 5000].iter() {
            assert_eq!(write(&efd, &u.to_ne_bytes()).unwrap(), size_of::<u64>());
        }
        efd
    });

    let efd = child.join().unwrap();

    let mut bytes = [0_u8; size_of::<u64>()];
    let s = read(&efd, &mut bytes).unwrap();
    assert_eq!(s, bytes.len());
    let u = u64::from_ne_bytes(bytes);
    assert_eq!(u, 5021);
}
