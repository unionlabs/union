#![cfg_attr(loom, allow(dead_code, unreachable_pub))]

//! Traits, helpers, and type definitions for asynchronous I/O functionality.
//!
//! This module is the asynchronous version of `std::io`. Primarily, it
//! defines two traits, [`AsyncRead`] and [`AsyncWrite`], which are asynchronous
//! versions of the [`Read`] and [`Write`] traits in the standard library.
//!
//! # AsyncRead and AsyncWrite
//!
//! Like the standard library's [`Read`] and [`Write`] traits, [`AsyncRead`] and
//! [`AsyncWrite`] provide the most general interface for reading and writing
//! input and output. Unlike the standard library's traits, however, they are
//! _asynchronous_ &mdash; meaning that reading from or writing to a `tokio::io`
//! type will _yield_ to the Tokio scheduler when IO is not ready, rather than
//! blocking. This allows other tasks to run while waiting on IO.
//!
//! Another difference is that `AsyncRead` and `AsyncWrite` only contain
//! core methods needed to provide asynchronous reading and writing
//! functionality. Instead, utility methods are defined in the [`AsyncReadExt`]
//! and [`AsyncWriteExt`] extension traits. These traits are automatically
//! implemented for all values that implement `AsyncRead` and `AsyncWrite`
//! respectively.
//!
//! End users will rarely interact directly with `AsyncRead` and
//! `AsyncWrite`. Instead, they will use the async functions defined in the
//! extension traits. Library authors are expected to implement `AsyncRead`
//! and `AsyncWrite` in order to provide types that behave like byte streams.
//!
//! Even with these differences, Tokio's `AsyncRead` and `AsyncWrite` traits
//! can be used in almost exactly the same manner as the standard library's
//! `Read` and `Write`. Most types in the standard library that implement `Read`
//! and `Write` have asynchronous equivalents in `tokio` that implement
//! `AsyncRead` and `AsyncWrite`, such as [`File`] and [`TcpStream`].
//!
//! For example, the standard library documentation introduces `Read` by
//! [demonstrating][std_example] reading some bytes from a [`std::fs::File`]. We
//! can do the same with [`tokio::fs::File`][`File`]:
//!
//! ```no_run
//! use tokio::io::{self, AsyncReadExt};
//! use tokio::fs::File;
//!
//! #[tokio::main]
//! async fn main() -> io::Result<()> {
//!     let mut f = File::open("foo.txt").await?;
//!     let mut buffer = [0; 10];
//!
//!     // read up to 10 bytes
//!     let n = f.read(&mut buffer).await?;
//!
//!     println!("The bytes: {:?}", &buffer[..n]);
//!     Ok(())
//! }
//! ```
//!
//! [`File`]: crate::fs::File
//! [`TcpStream`]: crate::net::TcpStream
//! [`std::fs::File`]: std::fs::File
//! [std_example]: std::io#read-and-write
//!
//! ## Buffered Readers and Writers
//!
//! Byte-based interfaces are unwieldy and can be inefficient, as we'd need to be
//! making near-constant calls to the operating system. To help with this,
//! `std::io` comes with [support for _buffered_ readers and writers][stdbuf],
//! and therefore, `tokio::io` does as well.
//!
//! Tokio provides an async version of the [`std::io::BufRead`] trait,
//! [`AsyncBufRead`]; and async [`BufReader`] and [`BufWriter`] structs, which
//! wrap readers and writers. These wrappers use a buffer, reducing the number
//! of calls and providing nicer methods for accessing exactly what you want.
//!
//! For example, [`BufReader`] works with the [`AsyncBufRead`] trait to add
//! extra methods to any async reader:
//!
//! ```no_run
//! use tokio::io::{self, BufReader, AsyncBufReadExt};
//! use tokio::fs::File;
//!
//! #[tokio::main]
//! async fn main() -> io::Result<()> {
//!     let f = File::open("foo.txt").await?;
//!     let mut reader = BufReader::new(f);
//!     let mut buffer = String::new();
//!
//!     // read a line into buffer
//!     reader.read_line(&mut buffer).await?;
//!
//!     println!("{}", buffer);
//!     Ok(())
//! }
//! ```
//!
//! [`BufWriter`] doesn't add any new ways of writing; it just buffers every call
//! to [`write`](crate::io::AsyncWriteExt::write). However, you **must** flush
//! [`BufWriter`] to ensure that any buffered data is written.
//!
//! ```no_run
//! use tokio::io::{self, BufWriter, AsyncWriteExt};
//! use tokio::fs::File;
//!
//! #[tokio::main]
//! async fn main() -> io::Result<()> {
//!     let f = File::create("foo.txt").await?;
//!     {
//!         let mut writer = BufWriter::new(f);
//!
//!         // Write a byte to the buffer.
//!         writer.write(&[42u8]).await?;
//!
//!         // Flush the buffer before it goes out of scope.
//!         writer.flush().await?;
//!
//!     } // Unless flushed or shut down, the contents of the buffer is discarded on drop.
//!
//!     Ok(())
//! }
//! ```
//!
//! [stdbuf]: std::io#bufreader-and-bufwriter
//! [`std::io::BufRead`]: std::io::BufRead
//! [`AsyncBufRead`]: crate::io::AsyncBufRead
//! [`BufReader`]: crate::io::BufReader
//! [`BufWriter`]: crate::io::BufWriter
//!
//! ## Implementing AsyncRead and AsyncWrite
//!
//! Because they are traits, we can implement [`AsyncRead`] and [`AsyncWrite`] for
//! our own types, as well. Note that these traits must only be implemented for
//! non-blocking I/O types that integrate with the futures type system. In
//! other words, these types must never block the thread, and instead the
//! current task is notified when the I/O resource is ready.
//!
//! ## Conversion to and from Sink/Stream
//!
//! It is often convenient to encapsulate the reading and writing of
//! bytes and instead work with a [`Sink`] or [`Stream`] of some data
//! type that is encoded as bytes and/or decoded from bytes. Tokio
//! provides some utility traits in the [tokio-util] crate that
//! abstract the asynchronous buffering that is required and allows
//! you to write [`Encoder`] and [`Decoder`] functions working with a
//! buffer of bytes, and then use that ["codec"] to transform anything
//! that implements [`AsyncRead`] and [`AsyncWrite`] into a `Sink`/`Stream` of
//! your structured data.
//!
//! [tokio-util]: https://docs.rs/tokio-util/0.6/tokio_util/codec/index.html
//!
//! # Standard input and output
//!
//! Tokio provides asynchronous APIs to standard [input], [output], and [error].
//! These APIs are very similar to the ones provided by `std`, but they also
//! implement [`AsyncRead`] and [`AsyncWrite`].
//!
//! Note that the standard input / output APIs  **must** be used from the
//! context of the Tokio runtime, as they require Tokio-specific features to
//! function. Calling these functions outside of a Tokio runtime will panic.
//!
//! [input]: fn@stdin
//! [output]: fn@stdout
//! [error]: fn@stderr
//!
//! # `std` re-exports
//!
//! Additionally, [`Error`], [`ErrorKind`], [`Result`], and [`SeekFrom`] are
//! re-exported from `std::io` for ease of use.
//!
//! [`AsyncRead`]: trait@AsyncRead
//! [`AsyncWrite`]: trait@AsyncWrite
//! [`AsyncReadExt`]: trait@AsyncReadExt
//! [`AsyncWriteExt`]: trait@AsyncWriteExt
//! ["codec"]: https://docs.rs/tokio-util/0.6/tokio_util/codec/index.html
//! [`Encoder`]: https://docs.rs/tokio-util/0.6/tokio_util/codec/trait.Encoder.html
//! [`Decoder`]: https://docs.rs/tokio-util/0.6/tokio_util/codec/trait.Decoder.html
//! [`Error`]: struct@Error
//! [`ErrorKind`]: enum@ErrorKind
//! [`Result`]: type@Result
//! [`Read`]: std::io::Read
//! [`SeekFrom`]: enum@SeekFrom
//! [`Sink`]: https://docs.rs/futures/0.3/futures/sink/trait.Sink.html
//! [`Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
//! [`Write`]: std::io::Write
cfg_io_blocking! {
    pub(crate) mod blocking;
}

mod async_buf_read;
pub use self::async_buf_read::AsyncBufRead;

mod async_read;
pub use self::async_read::AsyncRead;

mod async_seek;
pub use self::async_seek::AsyncSeek;

mod async_write;
pub use self::async_write::AsyncWrite;

mod read_buf;
pub use self::read_buf::ReadBuf;

// Re-export some types from `std::io` so that users don't have to deal
// with conflicts when `use`ing `tokio::io` and `std::io`.
#[doc(no_inline)]
pub use std::io::{Error, ErrorKind, Result, SeekFrom};

cfg_io_driver_impl! {
    pub(crate) mod driver;

    cfg_net! {
        pub use driver::{Interest, Ready};
    }

    mod poll_evented;

    #[cfg(not(loom))]
    pub(crate) use poll_evented::PollEvented;
}

cfg_aio! {
    /// BSD-specific I/O types.
    pub mod bsd {
        mod poll_aio;

        pub use poll_aio::{Aio, AioEvent, AioSource};
    }
}

cfg_net_unix! {
    mod async_fd;

    pub mod unix {
        //! Asynchronous IO structures specific to Unix-like operating systems.
        pub use super::async_fd::{AsyncFd, AsyncFdReadyGuard, AsyncFdReadyMutGuard, TryIoError};
    }
}

cfg_io_std! {
    mod stdio_common;

    mod stderr;
    pub use stderr::{stderr, Stderr};

    mod stdin;
    pub use stdin::{stdin, Stdin};

    mod stdout;
    pub use stdout::{stdout, Stdout};
}

cfg_io_util! {
    mod split;
    pub use split::{split, ReadHalf, WriteHalf};

    pub(crate) mod seek;
    pub(crate) mod util;
    pub use util::{
        copy, copy_bidirectional, copy_buf, duplex, empty, repeat, sink, AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt,
        BufReader, BufStream, BufWriter, DuplexStream, Empty, Lines, Repeat, Sink, Split, Take,
    };
}

cfg_not_io_util! {
    cfg_process! {
        pub(crate) mod util;
    }
}

cfg_io_blocking! {
    /// Types in this module can be mocked out in tests.
    mod sys {
        // TODO: don't rename
        pub(crate) use crate::blocking::spawn_blocking as run;
        pub(crate) use crate::blocking::JoinHandle as Blocking;
    }
}
