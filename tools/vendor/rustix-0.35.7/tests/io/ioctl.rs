// `is_read_write` is not yet implemented on Windows. And `ioctl_fionread`
// on Windows doesn't work on files.
#[cfg(not(windows))]
#[test]
fn test_ioctls() {
    let file = std::fs::File::open("Cargo.toml").unwrap();

    #[cfg(feature = "net")]
    assert_eq!(rustix::io::is_read_write(&file).unwrap(), (true, false));

    assert_eq!(
        rustix::io::ioctl_fionread(&file).unwrap(),
        file.metadata().unwrap().len()
    );
}
