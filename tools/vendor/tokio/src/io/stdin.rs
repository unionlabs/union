use crate::io::blocking::Blocking;
use crate::io::{AsyncRead, ReadBuf};

use std::io;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

cfg_io_std! {
    /// A handle to the standard input stream of a process.
    ///
    /// The handle implements the [`AsyncRead`] trait, but beware that concurrent
    /// reads of `Stdin` must be executed with care.
    ///
    /// This handle is best used for non-interactive uses, such as when a file
    /// is piped into the application. For technical reasons, `stdin` is
    /// implemented by using an ordinary blocking read on a separate thread, and
    /// it is impossible to cancel that read. This can make shutdown of the
    /// runtime hang until the user presses enter.
    ///
    /// For interactive uses, it is recommended to spawn a thread dedicated to
    /// user input and use blocking IO directly in that thread.
    ///
    /// Created by the [`stdin`] function.
    ///
    /// [`stdin`]: fn@stdin
    /// [`AsyncRead`]: trait@AsyncRead
    #[derive(Debug)]
    pub struct Stdin {
        std: Blocking<std::io::Stdin>,
    }

    /// Constructs a new handle to the standard input of the current process.
    ///
    /// This handle is best used for non-interactive uses, such as when a file
    /// is piped into the application. For technical reasons, `stdin` is
    /// implemented by using an ordinary blocking read on a separate thread, and
    /// it is impossible to cancel that read. This can make shutdown of the
    /// runtime hang until the user presses enter.
    ///
    /// For interactive uses, it is recommended to spawn a thread dedicated to
    /// user input and use blocking IO directly in that thread.
    pub fn stdin() -> Stdin {
        let std = io::stdin();
        Stdin {
            std: Blocking::new(std),
        }
    }
}

#[cfg(unix)]
impl std::os::unix::io::AsRawFd for Stdin {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        std::io::stdin().as_raw_fd()
    }
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for Stdin {
    fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
        std::io::stdin().as_raw_handle()
    }
}

impl AsyncRead for Stdin {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.std).poll_read(cx, buf)
    }
}
