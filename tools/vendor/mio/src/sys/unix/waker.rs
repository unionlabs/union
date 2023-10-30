#[cfg(any(target_os = "linux", target_os = "android"))]
mod eventfd {
    use crate::sys::Selector;
    use crate::{Interest, Token};

    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::os::unix::io::FromRawFd;

    /// Waker backed by `eventfd`.
    ///
    /// `eventfd` is effectively an 64 bit counter. All writes must be of 8
    /// bytes (64 bits) and are converted (native endian) into an 64 bit
    /// unsigned integer and added to the count. Reads must also be 8 bytes and
    /// reset the count to 0, returning the count.
    #[derive(Debug)]
    pub struct Waker {
        fd: File,
    }

    impl Waker {
        pub fn new(selector: &Selector, token: Token) -> io::Result<Waker> {
            syscall!(eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK)).and_then(|fd| {
                // Turn the file descriptor into a file first so we're ensured
                // it's closed when dropped, e.g. when register below fails.
                let file = unsafe { File::from_raw_fd(fd) };
                selector
                    .register(fd, token, Interest::READABLE)
                    .map(|()| Waker { fd: file })
            })
        }

        pub fn wake(&self) -> io::Result<()> {
            let buf: [u8; 8] = 1u64.to_ne_bytes();
            match (&self.fd).write(&buf) {
                Ok(_) => Ok(()),
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                    // Writing only blocks if the counter is going to overflow.
                    // So we'll reset the counter to 0 and wake it again.
                    self.reset()?;
                    self.wake()
                }
                Err(err) => Err(err),
            }
        }

        /// Reset the eventfd object, only need to call this if `wake` fails.
        fn reset(&self) -> io::Result<()> {
            let mut buf: [u8; 8] = 0u64.to_ne_bytes();
            match (&self.fd).read(&mut buf) {
                Ok(_) => Ok(()),
                // If the `Waker` hasn't been awoken yet this will return a
                // `WouldBlock` error which we can safely ignore.
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => Ok(()),
                Err(err) => Err(err),
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::eventfd::Waker;

#[cfg(any(target_os = "freebsd", target_os = "ios", target_os = "macos"))]
mod kqueue {
    use crate::sys::Selector;
    use crate::Token;

    use std::io;

    /// Waker backed by kqueue user space notifications (`EVFILT_USER`).
    ///
    /// The implementation is fairly simple, first the kqueue must be setup to
    /// receive waker events this done by calling `Selector.setup_waker`. Next
    /// we need access to kqueue, thus we need to duplicate the file descriptor.
    /// Now waking is as simple as adding an event to the kqueue.
    #[derive(Debug)]
    pub struct Waker {
        selector: Selector,
        token: Token,
    }

    impl Waker {
        pub fn new(selector: &Selector, token: Token) -> io::Result<Waker> {
            selector.try_clone().and_then(|selector| {
                selector
                    .setup_waker(token)
                    .map(|()| Waker { selector, token })
            })
        }

        pub fn wake(&self) -> io::Result<()> {
            self.selector.wake(self.token)
        }
    }
}

#[cfg(any(target_os = "freebsd", target_os = "ios", target_os = "macos"))]
pub use self::kqueue::Waker;

#[cfg(any(
    target_os = "dragonfly",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
))]
mod pipe {
    use crate::sys::unix::Selector;
    use crate::{Interest, Token};

    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::os::unix::io::FromRawFd;

    /// Waker backed by a unix pipe.
    ///
    /// Waker controls both the sending and receiving ends and empties the pipe
    /// if writing to it (waking) fails.
    #[derive(Debug)]
    pub struct Waker {
        sender: File,
        receiver: File,
    }

    impl Waker {
        pub fn new(selector: &Selector, token: Token) -> io::Result<Waker> {
            let mut fds = [-1; 2];
            syscall!(pipe2(fds.as_mut_ptr(), libc::O_NONBLOCK | libc::O_CLOEXEC))?;
            // Turn the file descriptors into files first so we're ensured
            // they're closed when dropped, e.g. when register below fails.
            let sender = unsafe { File::from_raw_fd(fds[1]) };
            let receiver = unsafe { File::from_raw_fd(fds[0]) };
            selector
                .register(fds[0], token, Interest::READABLE)
                .map(|()| Waker { sender, receiver })
        }

        pub fn wake(&self) -> io::Result<()> {
            // The epoll emulation on some illumos systems currently requires
            // the pipe buffer to be completely empty for an edge-triggered
            // wakeup on the pipe read side.
            #[cfg(target_os = "illumos")]
            self.empty();

            match (&self.sender).write(&[1]) {
                Ok(_) => Ok(()),
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                    // The reading end is full so we'll empty the buffer and try
                    // again.
                    self.empty();
                    self.wake()
                }
                Err(ref err) if err.kind() == io::ErrorKind::Interrupted => self.wake(),
                Err(err) => Err(err),
            }
        }

        /// Empty the pipe's buffer, only need to call this if `wake` fails.
        /// This ignores any errors.
        fn empty(&self) {
            let mut buf = [0; 4096];
            loop {
                match (&self.receiver).read(&mut buf) {
                    Ok(n) if n > 0 => continue,
                    _ => return,
                }
            }
        }
    }
}

#[cfg(any(
    target_os = "dragonfly",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
))]
pub use self::pipe::Waker;
