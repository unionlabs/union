use crate::io::sys;
use crate::io::{AsyncRead, AsyncWrite, ReadBuf};

use std::cmp;
use std::future::Future;
use std::io;
use std::io::prelude::*;
use std::pin::Pin;
use std::task::Poll::*;
use std::task::{Context, Poll};

use self::State::*;

/// `T` should not implement _both_ Read and Write.
#[derive(Debug)]
pub(crate) struct Blocking<T> {
    inner: Option<T>,
    state: State<T>,
    /// `true` if the lower IO layer needs flushing.
    need_flush: bool,
}

#[derive(Debug)]
pub(crate) struct Buf {
    buf: Vec<u8>,
    pos: usize,
}

pub(crate) const MAX_BUF: usize = 16 * 1024;

#[derive(Debug)]
enum State<T> {
    Idle(Option<Buf>),
    Busy(sys::Blocking<(io::Result<usize>, Buf, T)>),
}

cfg_io_std! {
    impl<T> Blocking<T> {
        pub(crate) fn new(inner: T) -> Blocking<T> {
            Blocking {
                inner: Some(inner),
                state: State::Idle(Some(Buf::with_capacity(0))),
                need_flush: false,
            }
        }
    }
}

impl<T> AsyncRead for Blocking<T>
where
    T: Read + Unpin + Send + 'static,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        dst: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            match self.state {
                Idle(ref mut buf_cell) => {
                    let mut buf = buf_cell.take().unwrap();

                    if !buf.is_empty() {
                        buf.copy_to(dst);
                        *buf_cell = Some(buf);
                        return Ready(Ok(()));
                    }

                    buf.ensure_capacity_for(dst);
                    let mut inner = self.inner.take().unwrap();

                    self.state = Busy(sys::run(move || {
                        let res = buf.read_from(&mut inner);
                        (res, buf, inner)
                    }));
                }
                Busy(ref mut rx) => {
                    let (res, mut buf, inner) = ready!(Pin::new(rx).poll(cx))?;
                    self.inner = Some(inner);

                    match res {
                        Ok(_) => {
                            buf.copy_to(dst);
                            self.state = Idle(Some(buf));
                            return Ready(Ok(()));
                        }
                        Err(e) => {
                            assert!(buf.is_empty());

                            self.state = Idle(Some(buf));
                            return Ready(Err(e));
                        }
                    }
                }
            }
        }
    }
}

impl<T> AsyncWrite for Blocking<T>
where
    T: Write + Unpin + Send + 'static,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        src: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match self.state {
                Idle(ref mut buf_cell) => {
                    let mut buf = buf_cell.take().unwrap();

                    assert!(buf.is_empty());

                    let n = buf.copy_from(src);
                    let mut inner = self.inner.take().unwrap();

                    self.state = Busy(sys::run(move || {
                        let n = buf.len();
                        let res = buf.write_to(&mut inner).map(|_| n);

                        (res, buf, inner)
                    }));
                    self.need_flush = true;

                    return Ready(Ok(n));
                }
                Busy(ref mut rx) => {
                    let (res, buf, inner) = ready!(Pin::new(rx).poll(cx))?;
                    self.state = Idle(Some(buf));
                    self.inner = Some(inner);

                    // If error, return
                    res?;
                }
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        loop {
            let need_flush = self.need_flush;
            match self.state {
                // The buffer is not used here
                Idle(ref mut buf_cell) => {
                    if need_flush {
                        let buf = buf_cell.take().unwrap();
                        let mut inner = self.inner.take().unwrap();

                        self.state = Busy(sys::run(move || {
                            let res = inner.flush().map(|_| 0);
                            (res, buf, inner)
                        }));

                        self.need_flush = false;
                    } else {
                        return Ready(Ok(()));
                    }
                }
                Busy(ref mut rx) => {
                    let (res, buf, inner) = ready!(Pin::new(rx).poll(cx))?;
                    self.state = Idle(Some(buf));
                    self.inner = Some(inner);

                    // If error, return
                    res?;
                }
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}

/// Repeats operations that are interrupted.
macro_rules! uninterruptibly {
    ($e:expr) => {{
        loop {
            match $e {
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                res => break res,
            }
        }
    }};
}

impl Buf {
    pub(crate) fn with_capacity(n: usize) -> Buf {
        Buf {
            buf: Vec::with_capacity(n),
            pos: 0,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn len(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub(crate) fn copy_to(&mut self, dst: &mut ReadBuf<'_>) -> usize {
        let n = cmp::min(self.len(), dst.remaining());
        dst.put_slice(&self.bytes()[..n]);
        self.pos += n;

        if self.pos == self.buf.len() {
            self.buf.truncate(0);
            self.pos = 0;
        }

        n
    }

    pub(crate) fn copy_from(&mut self, src: &[u8]) -> usize {
        assert!(self.is_empty());

        let n = cmp::min(src.len(), MAX_BUF);

        self.buf.extend_from_slice(&src[..n]);
        n
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.buf[self.pos..]
    }

    pub(crate) fn ensure_capacity_for(&mut self, bytes: &ReadBuf<'_>) {
        assert!(self.is_empty());

        let len = cmp::min(bytes.remaining(), MAX_BUF);

        if self.buf.len() < len {
            self.buf.reserve(len - self.buf.len());
        }

        unsafe {
            self.buf.set_len(len);
        }
    }

    pub(crate) fn read_from<T: Read>(&mut self, rd: &mut T) -> io::Result<usize> {
        let res = uninterruptibly!(rd.read(&mut self.buf));

        if let Ok(n) = res {
            self.buf.truncate(n);
        } else {
            self.buf.clear();
        }

        assert_eq!(self.pos, 0);

        res
    }

    pub(crate) fn write_to<T: Write>(&mut self, wr: &mut T) -> io::Result<()> {
        assert_eq!(self.pos, 0);

        // `write_all` already ignores interrupts
        let res = wr.write_all(&self.buf);
        self.buf.clear();
        res
    }
}

cfg_fs! {
    impl Buf {
        pub(crate) fn discard_read(&mut self) -> i64 {
            let ret = -(self.bytes().len() as i64);
            self.pos = 0;
            self.buf.truncate(0);
            ret
        }
    }
}
