#![deny(rust_2018_idioms)]

use std::io::{Read, Seek, SeekFrom, Write};

use tempfile::{spooled_tempfile, SpooledTempFile};

#[test]
fn test_automatic_rollover() {
    let mut t = spooled_tempfile(10);
    let mut buf = Vec::new();

    assert!(!t.is_rolled());
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 0);
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 0);
    assert_eq!(buf.as_slice(), b"");
    buf.clear();

    assert_eq!(t.write(b"abcde").unwrap(), 5);

    assert!(!t.is_rolled());
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 5);
    assert_eq!(buf.as_slice(), b"abcde");

    assert_eq!(t.write(b"fghijklmno").unwrap(), 10);

    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 15);
    assert!(t.is_rolled());
}

#[test]
fn test_explicit_rollover() {
    let mut t = SpooledTempFile::new(100);
    assert_eq!(t.write(b"abcdefghijklmnopqrstuvwxyz").unwrap(), 26);
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 26);
    assert!(!t.is_rolled());

    // roll over explicitly
    assert!(t.roll().is_ok());
    assert!(t.is_rolled());
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 26);

    let mut buf = Vec::new();
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 0);
    assert_eq!(buf.as_slice(), b"");
    buf.clear();

    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 26);
    assert_eq!(buf.as_slice(), b"abcdefghijklmnopqrstuvwxyz");
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 26);
}

// called by test_seek_{buffer, file}
// assumes t is empty and offset is 0 to start
fn test_seek(t: &mut SpooledTempFile) {
    assert_eq!(t.write(b"abcdefghijklmnopqrstuvwxyz").unwrap(), 26);

    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 26); // tell()
    assert_eq!(t.seek(SeekFrom::Current(-1)).unwrap(), 25);
    assert_eq!(t.seek(SeekFrom::Current(1)).unwrap(), 26);
    assert_eq!(t.seek(SeekFrom::Current(1)).unwrap(), 27);
    assert_eq!(t.seek(SeekFrom::Current(-27)).unwrap(), 0);
    assert!(t.seek(SeekFrom::Current(-1)).is_err());
    assert!(t.seek(SeekFrom::Current(-1245)).is_err());

    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(t.seek(SeekFrom::Start(1)).unwrap(), 1);
    assert_eq!(t.seek(SeekFrom::Start(26)).unwrap(), 26);
    assert_eq!(t.seek(SeekFrom::Start(27)).unwrap(), 27);
    // // these are build errors
    // assert!(t.seek(SeekFrom::Start(-1)).is_err());
    // assert!(t.seek(SeekFrom::Start(-1000)).is_err());

    assert_eq!(t.seek(SeekFrom::End(0)).unwrap(), 26);
    assert_eq!(t.seek(SeekFrom::End(-1)).unwrap(), 25);
    assert_eq!(t.seek(SeekFrom::End(-26)).unwrap(), 0);
    assert!(t.seek(SeekFrom::End(-27)).is_err());
    assert!(t.seek(SeekFrom::End(-99)).is_err());
    assert_eq!(t.seek(SeekFrom::End(1)).unwrap(), 27);
    assert_eq!(t.seek(SeekFrom::End(1)).unwrap(), 27);
}

#[test]
fn test_seek_buffer() {
    let mut t = spooled_tempfile(100);
    test_seek(&mut t);
}

#[test]
fn test_seek_file() {
    let mut t = SpooledTempFile::new(10);
    test_seek(&mut t);
}

fn test_seek_read(t: &mut SpooledTempFile) {
    assert_eq!(t.write(b"abcdefghijklmnopqrstuvwxyz").unwrap(), 26);

    let mut buf = Vec::new();

    // we're at the end
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 0);
    assert_eq!(buf.as_slice(), b"");
    buf.clear();

    // seek to start, read whole thing
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 26);
    assert_eq!(buf.as_slice(), b"abcdefghijklmnopqrstuvwxyz");
    buf.clear();

    // now we're at the end again
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 26); // tell()
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 0);
    assert_eq!(buf.as_slice(), b"");
    buf.clear();

    // seek to somewhere in the middle, read a bit
    assert_eq!(t.seek(SeekFrom::Start(5)).unwrap(), 5);
    let mut buf = [0; 5];
    assert!(t.read_exact(&mut buf).is_ok());
    assert_eq!(buf, *b"fghij");

    // read again from current spot
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 10); // tell()
    assert!(t.read_exact(&mut buf).is_ok());
    assert_eq!(buf, *b"klmno");

    let mut buf = [0; 15];
    // partial read
    assert_eq!(t.read(&mut buf).unwrap(), 11);
    assert_eq!(buf[0..11], *b"pqrstuvwxyz");

    // try to read off the end: UnexpectedEof
    assert!(t.read_exact(&mut buf).is_err());
}

#[test]
fn test_seek_read_buffer() {
    let mut t = spooled_tempfile(100);
    test_seek_read(&mut t);
}

#[test]
fn test_seek_read_file() {
    let mut t = SpooledTempFile::new(10);
    test_seek_read(&mut t);
}

fn test_overwrite_middle(t: &mut SpooledTempFile) {
    assert_eq!(t.write(b"abcdefghijklmnopqrstuvwxyz").unwrap(), 26);

    assert_eq!(t.seek(SeekFrom::Start(10)).unwrap(), 10);
    assert_eq!(t.write(b"0123456789").unwrap(), 10);
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);

    let mut buf = Vec::new();
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 26);
    assert_eq!(buf.as_slice(), b"abcdefghij0123456789uvwxyz");
}

#[test]
fn test_overwrite_middle_of_buffer() {
    let mut t = spooled_tempfile(100);
    test_overwrite_middle(&mut t);
}

#[test]
fn test_overwrite_middle_of_file() {
    let mut t = SpooledTempFile::new(10);
    test_overwrite_middle(&mut t);
}

#[test]
fn test_overwrite_and_extend_buffer() {
    let mut t = spooled_tempfile(100);
    assert_eq!(t.write(b"abcdefghijklmnopqrstuvwxyz").unwrap(), 26);
    assert_eq!(t.seek(SeekFrom::End(-5)).unwrap(), 21);
    assert_eq!(t.write(b"0123456789").unwrap(), 10);
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    let mut buf = Vec::new();
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 31);
    assert_eq!(buf.as_slice(), b"abcdefghijklmnopqrstu0123456789");
    assert!(!t.is_rolled());
}

#[test]
fn test_overwrite_and_extend_rollover() {
    let mut t = SpooledTempFile::new(20);
    assert_eq!(t.write(b"abcdefghijklmno").unwrap(), 15);
    assert!(!t.is_rolled());
    assert_eq!(t.seek(SeekFrom::End(-5)).unwrap(), 10);
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 10); // tell()
    assert!(!t.is_rolled());
    assert_eq!(t.write(b"0123456789)!@#$%^&*(").unwrap(), 20);
    assert!(t.is_rolled());
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 30); // tell()
    let mut buf = Vec::new();
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 30);
    assert_eq!(buf.as_slice(), b"abcdefghij0123456789)!@#$%^&*(");
}

fn test_sparse(t: &mut SpooledTempFile) {
    assert_eq!(t.write(b"abcde").unwrap(), 5);
    assert_eq!(t.seek(SeekFrom::Current(5)).unwrap(), 10);
    assert_eq!(t.write(b"klmno").unwrap(), 5);
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    let mut buf = Vec::new();
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 15);
    assert_eq!(buf.as_slice(), b"abcde\0\0\0\0\0klmno");
}

#[test]
fn test_sparse_buffer() {
    let mut t = spooled_tempfile(100);
    test_sparse(&mut t);
}

#[test]
fn test_sparse_file() {
    let mut t = SpooledTempFile::new(1);
    test_sparse(&mut t);
}

#[test]
fn test_sparse_write_rollover() {
    let mut t = spooled_tempfile(10);
    assert_eq!(t.write(b"abcde").unwrap(), 5);
    assert!(!t.is_rolled());
    assert_eq!(t.seek(SeekFrom::Current(5)).unwrap(), 10);
    assert!(!t.is_rolled());
    assert_eq!(t.write(b"klmno").unwrap(), 5);
    assert!(t.is_rolled());
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    let mut buf = Vec::new();
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 15);
    assert_eq!(buf.as_slice(), b"abcde\0\0\0\0\0klmno");
}

fn test_set_len(t: &mut SpooledTempFile) {
    let mut buf: Vec<u8> = Vec::new();

    assert_eq!(t.write(b"abcdefghijklmnopqrstuvwxyz").unwrap(), 26);

    // truncate to 10 bytes
    assert!(t.set_len(10).is_ok());

    // position should not have moved
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 26); // tell()

    assert_eq!(t.read_to_end(&mut buf).unwrap(), 0);
    assert_eq!(buf.as_slice(), b"");
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 26); // tell()
    buf.clear();

    // read whole thing
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 10);
    assert_eq!(buf.as_slice(), b"abcdefghij");
    buf.clear();

    // set_len to expand beyond the end
    assert!(t.set_len(40).is_ok());
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 10); // tell()
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 40);
    assert_eq!(
        buf.as_slice(),
        &b"abcdefghij\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"[..]
    );
}

#[test]
fn test_set_len_buffer() {
    let mut t = spooled_tempfile(100);
    test_set_len(&mut t);
}

#[test]
fn test_set_len_file() {
    let mut t = spooled_tempfile(100);
    test_set_len(&mut t);
}

#[test]
fn test_set_len_rollover() {
    let mut buf: Vec<u8> = Vec::new();

    let mut t = spooled_tempfile(10);
    assert_eq!(t.write(b"abcde").unwrap(), 5);
    assert!(!t.is_rolled());
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 5); // tell()

    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 5);
    assert_eq!(buf.as_slice(), b"abcde");
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 5); // tell()
    buf.clear();

    assert!(t.set_len(20).is_ok());
    assert!(t.is_rolled());
    assert_eq!(t.seek(SeekFrom::Current(0)).unwrap(), 5); // tell()
    assert_eq!(t.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(t.read_to_end(&mut buf).unwrap(), 20);
    assert_eq!(buf.as_slice(), b"abcde\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
}
