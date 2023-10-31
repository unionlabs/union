#[cfg(not(target_os = "redox"))]
#[test]
fn test_flock() {
    use rustix::fs::{cwd, flock, openat, FlockOperation, Mode, OFlags};

    let f = openat(cwd(), "Cargo.toml", OFlags::RDONLY, Mode::empty()).unwrap();
    flock(&f, FlockOperation::LockExclusive).unwrap();
    flock(&f, FlockOperation::Unlock).unwrap();
    let g = openat(cwd(), "Cargo.toml", OFlags::RDONLY, Mode::empty()).unwrap();
    flock(&g, FlockOperation::LockExclusive).unwrap();
    flock(&g, FlockOperation::Unlock).unwrap();
    drop(f);
    drop(g);

    let f = openat(cwd(), "Cargo.toml", OFlags::RDONLY, Mode::empty()).unwrap();
    flock(&f, FlockOperation::LockShared).unwrap();
    let g = openat(cwd(), "Cargo.toml", OFlags::RDONLY, Mode::empty()).unwrap();
    flock(&g, FlockOperation::LockShared).unwrap();
    flock(&f, FlockOperation::Unlock).unwrap();
    flock(&g, FlockOperation::Unlock).unwrap();
    drop(f);
    drop(g);

    let f = openat(cwd(), "Cargo.toml", OFlags::RDONLY, Mode::empty()).unwrap();
    flock(&f, FlockOperation::LockShared).unwrap();
    flock(&f, FlockOperation::LockExclusive).unwrap();
    flock(&f, FlockOperation::Unlock).unwrap();
    let g = openat(cwd(), "Cargo.toml", OFlags::RDONLY, Mode::empty()).unwrap();
    flock(&g, FlockOperation::LockShared).unwrap();
    flock(&g, FlockOperation::LockExclusive).unwrap();
    flock(&g, FlockOperation::Unlock).unwrap();
    drop(f);
    drop(g);
}
