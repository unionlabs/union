use std::os::unix::prelude::*;
use tempfile::tempfile;

use nix::fcntl;
use nix::errno::Errno;
use nix::pty::openpty;
use nix::sys::termios::{self, LocalFlags, OutputFlags, tcgetattr};
use nix::unistd::{read, write, close};

/// Helper function analogous to `std::io::Write::write_all`, but for `RawFd`s
fn write_all(f: RawFd, buf: &[u8]) {
    let mut len = 0;
    while len < buf.len() {
        len += write(f, &buf[len..]).unwrap();
    }
}

// Test tcgetattr on a terminal
#[test]
fn test_tcgetattr_pty() {
    // openpty uses ptname(3) internally
    let _m = crate::PTSNAME_MTX.lock();

    let pty = openpty(None, None).expect("openpty failed");
    assert!(termios::tcgetattr(pty.slave).is_ok());
    close(pty.master).expect("closing the master failed");
    close(pty.slave).expect("closing the slave failed");
}

// Test tcgetattr on something that isn't a terminal
#[test]
fn test_tcgetattr_enotty() {
    let file = tempfile().unwrap();
    assert_eq!(termios::tcgetattr(file.as_raw_fd()).err(),
               Some(Errno::ENOTTY));
}

// Test tcgetattr on an invalid file descriptor
#[test]
fn test_tcgetattr_ebadf() {
    assert_eq!(termios::tcgetattr(-1).err(),
               Some(Errno::EBADF));
}

// Test modifying output flags
#[test]
fn test_output_flags() {
    // openpty uses ptname(3) internally
    let _m = crate::PTSNAME_MTX.lock();

    // Open one pty to get attributes for the second one
    let mut termios = {
        let pty = openpty(None, None).expect("openpty failed");
        assert!(pty.master > 0);
        assert!(pty.slave > 0);
        let termios = tcgetattr(pty.slave).expect("tcgetattr failed");
        close(pty.master).unwrap();
        close(pty.slave).unwrap();
        termios
    };

    // Make sure postprocessing '\r' isn't specified by default or this test is useless.
    assert!(!termios.output_flags.contains(OutputFlags::OPOST | OutputFlags::OCRNL));

    // Specify that '\r' characters should be transformed to '\n'
    // OPOST is specified to enable post-processing
    termios.output_flags.insert(OutputFlags::OPOST | OutputFlags::OCRNL);

    // Open a pty
    let pty = openpty(None, &termios).unwrap();
    assert!(pty.master > 0);
    assert!(pty.slave > 0);

    // Write into the master
    let string = "foofoofoo\r";
    write_all(pty.master, string.as_bytes());

    // Read from the slave verifying that the output has been properly transformed
    let mut buf = [0u8; 10];
    crate::read_exact(pty.slave, &mut buf);
    let transformed_string = "foofoofoo\n";
    close(pty.master).unwrap();
    close(pty.slave).unwrap();
    assert_eq!(&buf, transformed_string.as_bytes());
}

// Test modifying local flags
#[test]
fn test_local_flags() {
    // openpty uses ptname(3) internally
    let _m = crate::PTSNAME_MTX.lock();

    // Open one pty to get attributes for the second one
    let mut termios = {
        let pty = openpty(None, None).unwrap();
        assert!(pty.master > 0);
        assert!(pty.slave > 0);
        let termios = tcgetattr(pty.slave).unwrap();
        close(pty.master).unwrap();
        close(pty.slave).unwrap();
        termios
    };

    // Make sure echo is specified by default or this test is useless.
    assert!(termios.local_flags.contains(LocalFlags::ECHO));

    // Disable local echo
    termios.local_flags.remove(LocalFlags::ECHO);

    // Open a new pty with our modified termios settings
    let pty = openpty(None, &termios).unwrap();
    assert!(pty.master > 0);
    assert!(pty.slave > 0);

    // Set the master is in nonblocking mode or reading will never return.
    let flags = fcntl::fcntl(pty.master, fcntl::F_GETFL).unwrap();
    let new_flags = fcntl::OFlag::from_bits_truncate(flags) | fcntl::OFlag::O_NONBLOCK;
    fcntl::fcntl(pty.master, fcntl::F_SETFL(new_flags)).unwrap();

    // Write into the master
    let string = "foofoofoo\r";
    write_all(pty.master, string.as_bytes());

    // Try to read from the master, which should not have anything as echoing was disabled.
    let mut buf = [0u8; 10];
    let read = read(pty.master, &mut buf).unwrap_err();
    close(pty.master).unwrap();
    close(pty.slave).unwrap();
    assert_eq!(read, Errno::EAGAIN);
}
