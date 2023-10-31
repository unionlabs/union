use rustix::fd::{AsFd, AsRawFd, FromRawFd, IntoRawFd, OwnedFd};
#[cfg(not(windows))]
use rustix::io::{poll, retry_on_intr};
use rustix::io::{PollFd, PollFlags};

#[cfg(not(windows))]
#[test]
fn test_poll() {
    use rustix::io::{pipe, read, write};

    // Create a pipe.
    let (reader, writer) = pipe().unwrap();
    let mut poll_fds = [PollFd::new(&reader, PollFlags::IN)];
    assert_eq!(poll_fds[0].as_fd().as_raw_fd(), reader.as_fd().as_raw_fd());

    // `poll` should say there's nothing ready to be read from the pipe.
    let num = retry_on_intr(|| poll(&mut poll_fds, 0)).unwrap();
    assert_eq!(num, 0);
    assert!(poll_fds[0].revents().is_empty());
    assert_eq!(poll_fds[0].as_fd().as_raw_fd(), reader.as_fd().as_raw_fd());

    // Write a byte to the pipe.
    assert_eq!(retry_on_intr(|| write(&writer, b"a")).unwrap(), 1);

    // `poll` should now say there's data to be read.
    let num = retry_on_intr(|| poll(&mut poll_fds, -1)).unwrap();
    assert_eq!(num, 1);
    assert_eq!(poll_fds[0].revents(), PollFlags::IN);
    assert_eq!(poll_fds[0].as_fd().as_raw_fd(), reader.as_fd().as_raw_fd());

    let mut temp = poll_fds[0].clone();
    assert_eq!(temp.revents(), PollFlags::IN);
    temp.clear_revents();
    assert!(temp.revents().is_empty());

    // Read the byte from the pipe.
    let mut buf = [b'\0'];
    assert_eq!(retry_on_intr(|| read(&reader, &mut buf)).unwrap(), 1);
    assert_eq!(buf[0], b'a');
    assert_eq!(poll_fds[0].as_fd().as_raw_fd(), reader.as_fd().as_raw_fd());

    // Poll should now say there's no more data to be read.
    let num = retry_on_intr(|| poll(&mut poll_fds, 0)).unwrap();
    assert_eq!(num, 0);
    assert!(poll_fds[0].revents().is_empty());
    assert_eq!(poll_fds[0].as_fd().as_raw_fd(), reader.as_fd().as_raw_fd());
}

#[test]
fn test_poll_fd_set_fd() {
    // Make up some file descriptors so that we can test that set_fd works.
    let a = unsafe { OwnedFd::from_raw_fd(777) };
    let mut poll_fd = PollFd::new(&a, PollFlags::empty());
    assert_eq!(poll_fd.as_fd().as_raw_fd(), 777);

    let b = unsafe { OwnedFd::from_raw_fd(888) };
    poll_fd.set_fd(&b);
    assert_eq!(poll_fd.as_fd().as_raw_fd(), 888);

    // Don't attempt to close our made-up file descriptors.
    let _ = a.into_raw_fd();
    let _ = b.into_raw_fd();
}
