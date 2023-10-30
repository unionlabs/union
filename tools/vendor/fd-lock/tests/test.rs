use fd_lock::RwLock;
use std::fs::File;
use std::io::ErrorKind;

use tempfile::tempdir;

#[test]
fn double_read_lock() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("lockfile");

    let l0 = RwLock::new(File::create(&path).unwrap());
    let l1 = RwLock::new(File::open(path).unwrap());

    let _g0 = l0.try_read().unwrap();
    let _g1 = l1.try_read().unwrap();
}

#[test]
fn double_write_lock() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("lockfile");

    let mut l0 = RwLock::new(File::create(&path).unwrap());
    let mut l1 = RwLock::new(File::open(path).unwrap());

    let g0 = l0.try_write().unwrap();

    let err = l1.try_write().unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::WouldBlock));

    drop(g0);
}
