#![cfg(not(target_os = "redox"))]

use rustix::fs::{Dir, DirEntry};
use std::collections::HashMap;

#[test]
fn dir_entries() {
    let tmpdir = tempfile::tempdir().expect("construct tempdir");
    let dirfd = std::fs::File::open(tmpdir.path()).expect("open tempdir as file");
    let mut dir = Dir::read_from(dirfd).expect("construct Dir from dirfd");

    let entries = read_entries(&mut dir);
    assert_eq!(entries.len(), 0, "no files in directory");

    let _f1 = std::fs::File::create(tmpdir.path().join("file1")).expect("create file1");

    let entries = read_entries(&mut dir);
    assert!(
        entries.get("file1").is_some(),
        "directory contains `file1`: {:?}",
        entries
    );
    assert_eq!(entries.len(), 1);

    let _f2 = std::fs::File::create(tmpdir.path().join("file2")).expect("create file1");
    let entries = read_entries(&mut dir);
    assert!(
        entries.get("file1").is_some(),
        "directory contains `file1`: {:?}",
        entries
    );
    assert!(
        entries.get("file2").is_some(),
        "directory contains `file2`: {:?}",
        entries
    );
    assert_eq!(entries.len(), 2);
}

fn read_entries(dir: &mut Dir) -> HashMap<String, DirEntry> {
    dir.rewind();
    let mut out = HashMap::new();
    loop {
        match dir.read() {
            Some(e) => {
                let e = e.expect("non-error entry");
                let name = e.file_name().to_str().expect("utf8 filename").to_owned();
                if name != "." && name != ".." {
                    out.insert(name, e);
                }
            }
            None => break,
        }
    }
    out
}

#[test]
fn dir_from_openat() {
    let dirfd = rustix::fs::openat(
        rustix::fs::cwd(),
        ".",
        rustix::fs::OFlags::RDONLY,
        rustix::fs::Mode::empty(),
    )
    .expect("open cwd as file");
    let _dir = Dir::read_from(dirfd).expect("construct Dir from dirfd");
}
