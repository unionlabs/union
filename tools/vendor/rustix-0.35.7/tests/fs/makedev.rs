use rustix::fs::{major, makedev, minor};

#[test]
fn makedev_roundtrip() {
    let maj = 0x2324_2526;
    let min = 0x6564_6361;
    let dev = makedev(maj, min);
    assert_eq!(maj, major(dev));
    assert_eq!(min, minor(dev));
}
