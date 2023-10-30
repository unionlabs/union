use crate::io::AsyncRead;

use bytes::BufMut;
use pin_project_lite::pin_project;
use std::future::Future;
use std::io;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::task::{Context, Poll};

pub(crate) fn read_buf<'a, R, B>(reader: &'a mut R, buf: &'a mut B) -> ReadBuf<'a, R, B>
where
    R: AsyncRead + Unpin,
    B: BufMut,
{
    ReadBuf {
        reader,
        buf,
        _pin: PhantomPinned,
    }
}

pin_project! {
    /// Future returned by [`read_buf`](crate::io::AsyncReadExt::read_buf).
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct ReadBuf<'a, R, B> {
        reader: &'a mut R,
        buf: &'a mut B,
        #[pin]
        _pin: PhantomPinned,
    }
}

impl<R, B> Future for ReadBuf<'_, R, B>
where
    R: AsyncRead + Unpin,
    B: BufMut,
{
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        use crate::io::ReadBuf;
        use std::mem::MaybeUninit;

        let me = self.project();

        if !me.buf.has_remaining_mut() {
            return Poll::Ready(Ok(0));
        }

        let n = {
            let dst = me.buf.chunk_mut();
            let dst = unsafe { &mut *(dst as *mut _ as *mut [MaybeUninit<u8>]) };
            let mut buf = ReadBuf::uninit(dst);
            let ptr = buf.filled().as_ptr();
            ready!(Pin::new(me.reader).poll_read(cx, &mut buf)?);

            // Ensure the pointer does not change from under us
            assert_eq!(ptr, buf.filled().as_ptr());
            buf.filled().len()
        };

        // Safety: This is guaranteed to be the number of initialized (and read)
        // bytes due to the invariants provided by `ReadBuf::filled`.
        unsafe {
            me.buf.advance_mut(n);
        }

        Poll::Ready(Ok(n))
    }
}
