use rustix::rand::{getrandom, GetRandomFlags};

#[test]
fn test_getrandom() {
    let mut buf = [0_u8; 256];
    let _ = getrandom(&mut buf, GetRandomFlags::empty());
}
