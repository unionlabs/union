#[cfg(feature = "fs")]
#[test]
fn test_seals() {
    use rustix::fd::FromFd;
    use rustix::fs::{
        fcntl_add_seals, fcntl_get_seals, ftruncate, memfd_create, MemfdFlags, SealFlags,
    };
    use std::fs::File;
    use std::io::Write;

    let fd = match memfd_create("test", MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING) {
        Ok(fd) => fd,
        Err(rustix::io::Errno::NOSYS) => return,
        Err(err) => Err(err).unwrap(),
    };
    let mut file = File::from_fd(fd.into());

    let old = fcntl_get_seals(&file).unwrap();
    assert_eq!(old, SealFlags::empty());

    writeln!(&mut file, "Hello!").unwrap();

    fcntl_add_seals(&file, SealFlags::GROW).unwrap();

    let now = fcntl_get_seals(&file).unwrap();
    assert_eq!(now, SealFlags::GROW);

    // We sealed growing, so this should fail.
    writeln!(&mut file, "World?").unwrap_err();

    // We can still shrink for now.
    ftruncate(&mut file, 1).unwrap();

    fcntl_add_seals(&file, SealFlags::SHRINK).unwrap();

    let now = fcntl_get_seals(&file).unwrap();
    assert_eq!(now, SealFlags::GROW | SealFlags::SHRINK);

    // We sealed shrinking, so this should fail.
    ftruncate(&mut file, 0).unwrap_err();
}
