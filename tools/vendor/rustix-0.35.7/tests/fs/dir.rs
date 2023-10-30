#[test]
fn test_dir() {
    let t = rustix::fs::openat(
        rustix::fs::cwd(),
        rustix::cstr!("."),
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    let dir = rustix::fs::Dir::read_from(&t).unwrap();

    let _file = rustix::fs::openat(
        &t,
        rustix::cstr!("Cargo.toml"),
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    let mut saw_dot = false;
    let mut saw_dotdot = false;
    let mut saw_cargo_toml = false;
    for entry in dir {
        let entry = entry.unwrap();
        if entry.file_name() == rustix::cstr!(".") {
            saw_dot = true;
        } else if entry.file_name() == rustix::cstr!("..") {
            saw_dotdot = true;
        } else if entry.file_name() == rustix::cstr!("Cargo.toml") {
            saw_cargo_toml = true;
        }
    }
    assert!(saw_dot);
    assert!(saw_dotdot);
    assert!(saw_cargo_toml);
}
