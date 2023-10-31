use std::io::prelude::*;
use std::os::unix::prelude::*;

use libc::off_t;
use nix::sys::sendfile::*;
use tempfile::tempfile;

cfg_if! {
    if #[cfg(any(target_os = "android", target_os = "linux"))] {
        use nix::unistd::{close, pipe, read};
    } else if #[cfg(any(target_os = "freebsd", target_os = "ios", target_os = "macos"))] {
        use std::net::Shutdown;
        use std::os::unix::net::UnixStream;
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[test]
fn test_sendfile_linux() {
    const CONTENTS: &[u8] = b"abcdef123456";
    let mut tmp = tempfile().unwrap();
    tmp.write_all(CONTENTS).unwrap();

    let (rd, wr) = pipe().unwrap();
    let mut offset: off_t = 5;
    let res = sendfile(wr, tmp.as_raw_fd(), Some(&mut offset), 2).unwrap();

    assert_eq!(2, res);

    let mut buf = [0u8; 1024];
    assert_eq!(2, read(rd, &mut buf).unwrap());
    assert_eq!(b"f1", &buf[0..2]);
    assert_eq!(7, offset);

    close(rd).unwrap();
    close(wr).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_sendfile64_linux() {
    const CONTENTS: &[u8] = b"abcdef123456";
    let mut tmp = tempfile().unwrap();
    tmp.write_all(CONTENTS).unwrap();

    let (rd, wr) = pipe().unwrap();
    let mut offset: libc::off64_t = 5;
    let res = sendfile64(wr, tmp.as_raw_fd(), Some(&mut offset), 2).unwrap();

    assert_eq!(2, res);

    let mut buf = [0u8; 1024];
    assert_eq!(2, read(rd, &mut buf).unwrap());
    assert_eq!(b"f1", &buf[0..2]);
    assert_eq!(7, offset);

    close(rd).unwrap();
    close(wr).unwrap();
}

#[cfg(target_os = "freebsd")]
#[test]
fn test_sendfile_freebsd() {
    // Declare the content
    let header_strings = vec!["HTTP/1.1 200 OK\n", "Content-Type: text/plain\n", "\n"];
    let body = "Xabcdef123456";
    let body_offset = 1;
    let trailer_strings = vec!["\n", "Served by Make Believe\n"];

    // Write the body to a file
    let mut tmp = tempfile().unwrap();
    tmp.write_all(body.as_bytes()).unwrap();

    // Prepare headers and trailers for sendfile
    let headers: Vec<&[u8]> = header_strings.iter().map(|s| s.as_bytes()).collect();
    let trailers: Vec<&[u8]> = trailer_strings.iter().map(|s| s.as_bytes()).collect();

    // Prepare socket pair
    let (mut rd, wr) = UnixStream::pair().unwrap();

    // Call the test method
    let (res, bytes_written) = sendfile(
        tmp.as_raw_fd(),
        wr.as_raw_fd(),
        body_offset as off_t,
        None,
        Some(headers.as_slice()),
        Some(trailers.as_slice()),
        SfFlags::empty(),
        0,
    );
    assert!(res.is_ok());
    wr.shutdown(Shutdown::Both).unwrap();

    // Prepare the expected result
    let expected_string =
        header_strings.concat() + &body[body_offset..] + &trailer_strings.concat();

    // Verify the message that was sent
    assert_eq!(bytes_written as usize, expected_string.as_bytes().len());

    let mut read_string = String::new();
    let bytes_read = rd.read_to_string(&mut read_string).unwrap();
    assert_eq!(bytes_written as usize, bytes_read);
    assert_eq!(expected_string, read_string);
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
#[test]
fn test_sendfile_darwin() {
    // Declare the content
    let header_strings = vec!["HTTP/1.1 200 OK\n", "Content-Type: text/plain\n", "\n"];
    let body = "Xabcdef123456";
    let body_offset = 1;
    let trailer_strings = vec!["\n", "Served by Make Believe\n"];

    // Write the body to a file
    let mut tmp = tempfile().unwrap();
    tmp.write_all(body.as_bytes()).unwrap();

    // Prepare headers and trailers for sendfile
    let headers: Vec<&[u8]> = header_strings.iter().map(|s| s.as_bytes()).collect();
    let trailers: Vec<&[u8]> = trailer_strings.iter().map(|s| s.as_bytes()).collect();

    // Prepare socket pair
    let (mut rd, wr) = UnixStream::pair().unwrap();

    // Call the test method
    let (res, bytes_written) = sendfile(
        tmp.as_raw_fd(),
        wr.as_raw_fd(),
        body_offset as off_t,
        None,
        Some(headers.as_slice()),
        Some(trailers.as_slice()),
    );
    assert!(res.is_ok());
    wr.shutdown(Shutdown::Both).unwrap();

    // Prepare the expected result
    let expected_string =
        header_strings.concat() + &body[body_offset..] + &trailer_strings.concat();

    // Verify the message that was sent
    assert_eq!(bytes_written as usize, expected_string.as_bytes().len());

    let mut read_string = String::new();
    let bytes_read = rd.read_to_string(&mut read_string).unwrap();
    assert_eq!(bytes_written as usize, bytes_read);
    assert_eq!(expected_string, read_string);
}
