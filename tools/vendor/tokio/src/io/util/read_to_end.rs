use crate::io::util::vec_with_initialized::{into_read_buf_parts, VecU8, VecWithInitialized};
use crate::io::AsyncRead;

use pin_project_lite::pin_project;
use std::future::Future;
use std::io;
use std::marker::PhantomPinned;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct ReadToEnd<'a, R: ?Sized> {
        reader: &'a mut R,
        buf: VecWithInitialized<&'a mut Vec<u8>>,
        // The number of bytes appended to buf. This can be less than buf.len() if
        // the buffer was not empty when the operation was started.
        read: usize,
        // Make this future `!Unpin` for compatibility with async trait methods.
        #[pin]
        _pin: PhantomPinned,
    }
}

pub(crate) fn read_to_end<'a, R>(reader: &'a mut R, buffer: &'a mut Vec<u8>) -> ReadToEnd<'a, R>
where
    R: AsyncRead + Unpin + ?Sized,
{
    ReadToEnd {
        reader,
        buf: VecWithInitialized::new(buffer),
        read: 0,
        _pin: PhantomPinned,
    }
}

pub(super) fn read_to_end_internal<V: VecU8, R: AsyncRead + ?Sized>(
    buf: &mut VecWithInitialized<V>,
    mut reader: Pin<&mut R>,
    num_read: &mut usize,
    cx: &mut Context<'_>,
) -> Poll<io::Result<usize>> {
    loop {
        let ret = ready!(poll_read_to_end(buf, reader.as_mut(), cx));
        match ret {
            Err(err) => return Poll::Ready(Err(err)),
            Ok(0) => return Poll::Ready(Ok(mem::replace(num_read, 0))),
            Ok(num) => {
                *num_read += num;
            }
        }
    }
}

/// Tries to read from the provided AsyncRead.
///
/// The length of the buffer is increased by the number of bytes read.
fn poll_read_to_end<V: VecU8, R: AsyncRead + ?Sized>(
    buf: &mut VecWithInitialized<V>,
    read: Pin<&mut R>,
    cx: &mut Context<'_>,
) -> Poll<io::Result<usize>> {
    // This uses an adaptive system to extend the vector when it fills. We want to
    // avoid paying to allocate and zero a huge chunk of memory if the reader only
    // has 4 bytes while still making large reads if the reader does have a ton
    // of data to return. Simply tacking on an extra DEFAULT_BUF_SIZE space every
    // time is 4,500 times (!) slower than this if the reader has a very small
    // amount of data to return.
    buf.reserve(32);

    // Get a ReadBuf into the vector.
    let mut read_buf = buf.get_read_buf();

    let filled_before = read_buf.filled().len();
    let poll_result = read.poll_read(cx, &mut read_buf);
    let filled_after = read_buf.filled().len();
    let n = filled_after - filled_before;

    // Update the length of the vector using the result of poll_read.
    let read_buf_parts = into_read_buf_parts(read_buf);
    buf.apply_read_buf(read_buf_parts);

    match poll_result {
        Poll::Pending => {
            // In this case, nothing should have been read. However we still
            // update the vector in case the poll_read call initialized parts of
            // the vector's unused capacity.
            debug_assert_eq!(filled_before, filled_after);
            Poll::Pending
        }
        Poll::Ready(Err(err)) => {
            debug_assert_eq!(filled_before, filled_after);
            Poll::Ready(Err(err))
        }
        Poll::Ready(Ok(())) => Poll::Ready(Ok(n)),
    }
}

impl<A> Future for ReadToEnd<'_, A>
where
    A: AsyncRead + ?Sized + Unpin,
{
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();

        read_to_end_internal(me.buf, Pin::new(*me.reader), me.read, cx)
    }
}
