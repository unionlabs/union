#![warn(rust_2018_idioms)]
#![cfg(all(target_os = "freebsd", feature = "net"))]

use mio_aio::{AioCb, AioFsyncMode, LioCb};
use std::{
    future::Future,
    mem,
    os::unix::io::{AsRawFd, RawFd},
    pin::Pin,
    task::{Context, Poll},
};
use tempfile::tempfile;
use tokio::io::bsd::{Aio, AioSource};
use tokio_test::assert_pending;

mod aio {
    use super::*;

    /// Adapts mio_aio::AioCb (which implements mio::event::Source) to AioSource
    struct WrappedAioCb<'a>(AioCb<'a>);
    impl<'a> AioSource for WrappedAioCb<'a> {
        fn register(&mut self, kq: RawFd, token: usize) {
            self.0.register_raw(kq, token)
        }
        fn deregister(&mut self) {
            self.0.deregister_raw()
        }
    }

    /// A very crude implementation of an AIO-based future
    struct FsyncFut(Aio<WrappedAioCb<'static>>);

    impl Future for FsyncFut {
        type Output = std::io::Result<()>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let poll_result = self.0.poll_ready(cx);
            match poll_result {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(_ev)) => {
                    // At this point, we could clear readiness.  But there's no
                    // point, since we're about to drop the Aio.
                    let result = (*self.0).0.aio_return();
                    match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(e) => Poll::Ready(Err(e.into())),
                    }
                }
            }
        }
    }

    /// Low-level AIO Source
    ///
    /// An example bypassing mio_aio and Nix to demonstrate how the kevent
    /// registration actually works, under the hood.
    struct LlSource(Pin<Box<libc::aiocb>>);

    impl AioSource for LlSource {
        fn register(&mut self, kq: RawFd, token: usize) {
            let mut sev: libc::sigevent = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            sev.sigev_notify = libc::SIGEV_KEVENT;
            sev.sigev_signo = kq;
            sev.sigev_value = libc::sigval {
                sival_ptr: token as *mut libc::c_void,
            };
            self.0.aio_sigevent = sev;
        }

        fn deregister(&mut self) {
            unsafe {
                self.0.aio_sigevent = mem::zeroed();
            }
        }
    }

    struct LlFut(Aio<LlSource>);

    impl Future for LlFut {
        type Output = std::io::Result<()>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let poll_result = self.0.poll_ready(cx);
            match poll_result {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(_ev)) => {
                    let r = unsafe { libc::aio_return(self.0 .0.as_mut().get_unchecked_mut()) };
                    assert_eq!(0, r);
                    Poll::Ready(Ok(()))
                }
            }
        }
    }

    /// A very simple object that can implement AioSource and can be reused.
    ///
    /// mio_aio normally assumes that each AioCb will be consumed on completion.
    /// This somewhat contrived example shows how an Aio object can be reused
    /// anyway.
    struct ReusableFsyncSource {
        aiocb: Pin<Box<AioCb<'static>>>,
        fd: RawFd,
        token: usize,
    }
    impl ReusableFsyncSource {
        fn fsync(&mut self) {
            self.aiocb.register_raw(self.fd, self.token);
            self.aiocb.fsync(AioFsyncMode::O_SYNC).unwrap();
        }
        fn new(aiocb: AioCb<'static>) -> Self {
            ReusableFsyncSource {
                aiocb: Box::pin(aiocb),
                fd: 0,
                token: 0,
            }
        }
        fn reset(&mut self, aiocb: AioCb<'static>) {
            self.aiocb = Box::pin(aiocb);
        }
    }
    impl AioSource for ReusableFsyncSource {
        fn register(&mut self, kq: RawFd, token: usize) {
            self.fd = kq;
            self.token = token;
        }
        fn deregister(&mut self) {
            self.fd = 0;
        }
    }

    struct ReusableFsyncFut<'a>(&'a mut Aio<ReusableFsyncSource>);
    impl<'a> Future for ReusableFsyncFut<'a> {
        type Output = std::io::Result<()>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let poll_result = self.0.poll_ready(cx);
            match poll_result {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(ev)) => {
                    // Since this future uses a reusable Aio, we must clear
                    // its readiness here.  That makes the future
                    // non-idempotent; the caller can't poll it repeatedly after
                    // it has already returned Ready.  But that's ok; most
                    // futures behave this way.
                    self.0.clear_ready(ev);
                    let result = (*self.0).aiocb.aio_return();
                    match result {
                        Ok(_) => Poll::Ready(Ok(())),
                        Err(e) => Poll::Ready(Err(e.into())),
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn fsync() {
        let f = tempfile().unwrap();
        let fd = f.as_raw_fd();
        let aiocb = AioCb::from_fd(fd, 0);
        let source = WrappedAioCb(aiocb);
        let mut poll_aio = Aio::new_for_aio(source).unwrap();
        (*poll_aio).0.fsync(AioFsyncMode::O_SYNC).unwrap();
        let fut = FsyncFut(poll_aio);
        fut.await.unwrap();
    }

    #[tokio::test]
    async fn ll_fsync() {
        let f = tempfile().unwrap();
        let fd = f.as_raw_fd();
        let mut aiocb: libc::aiocb = unsafe { mem::MaybeUninit::zeroed().assume_init() };
        aiocb.aio_fildes = fd;
        let source = LlSource(Box::pin(aiocb));
        let mut poll_aio = Aio::new_for_aio(source).unwrap();
        let r = unsafe {
            let p = (*poll_aio).0.as_mut().get_unchecked_mut();
            libc::aio_fsync(libc::O_SYNC, p)
        };
        assert_eq!(0, r);
        let fut = LlFut(poll_aio);
        fut.await.unwrap();
    }

    /// A suitably crafted future type can reuse an Aio object
    #[tokio::test]
    async fn reuse() {
        let f = tempfile().unwrap();
        let fd = f.as_raw_fd();
        let aiocb0 = AioCb::from_fd(fd, 0);
        let source = ReusableFsyncSource::new(aiocb0);
        let mut poll_aio = Aio::new_for_aio(source).unwrap();
        poll_aio.fsync();
        let fut0 = ReusableFsyncFut(&mut poll_aio);
        fut0.await.unwrap();

        let aiocb1 = AioCb::from_fd(fd, 0);
        poll_aio.reset(aiocb1);
        let mut ctx = Context::from_waker(futures::task::noop_waker_ref());
        assert_pending!(poll_aio.poll_ready(&mut ctx));
        poll_aio.fsync();
        let fut1 = ReusableFsyncFut(&mut poll_aio);
        fut1.await.unwrap();
    }
}

mod lio {
    use super::*;

    struct WrappedLioCb<'a>(LioCb<'a>);
    impl<'a> AioSource for WrappedLioCb<'a> {
        fn register(&mut self, kq: RawFd, token: usize) {
            self.0.register_raw(kq, token)
        }
        fn deregister(&mut self) {
            self.0.deregister_raw()
        }
    }

    /// A very crude lio_listio-based Future
    struct LioFut(Option<Aio<WrappedLioCb<'static>>>);

    impl Future for LioFut {
        type Output = std::io::Result<Vec<isize>>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let poll_result = self.0.as_mut().unwrap().poll_ready(cx);
            match poll_result {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(_ev)) => {
                    // At this point, we could clear readiness.  But there's no
                    // point, since we're about to drop the Aio.
                    let r = self.0.take().unwrap().into_inner().0.into_results(|iter| {
                        iter.map(|lr| lr.result.unwrap()).collect::<Vec<isize>>()
                    });
                    Poll::Ready(Ok(r))
                }
            }
        }
    }

    /// Minimal example demonstrating reuse of an Aio object with lio
    /// readiness.  mio_aio::LioCb actually does something similar under the
    /// hood.
    struct ReusableLioSource {
        liocb: Option<LioCb<'static>>,
        fd: RawFd,
        token: usize,
    }
    impl ReusableLioSource {
        fn new(liocb: LioCb<'static>) -> Self {
            ReusableLioSource {
                liocb: Some(liocb),
                fd: 0,
                token: 0,
            }
        }
        fn reset(&mut self, liocb: LioCb<'static>) {
            self.liocb = Some(liocb);
        }
        fn submit(&mut self) {
            self.liocb
                .as_mut()
                .unwrap()
                .register_raw(self.fd, self.token);
            self.liocb.as_mut().unwrap().submit().unwrap();
        }
    }
    impl AioSource for ReusableLioSource {
        fn register(&mut self, kq: RawFd, token: usize) {
            self.fd = kq;
            self.token = token;
        }
        fn deregister(&mut self) {
            self.fd = 0;
        }
    }
    struct ReusableLioFut<'a>(&'a mut Aio<ReusableLioSource>);
    impl<'a> Future for ReusableLioFut<'a> {
        type Output = std::io::Result<Vec<isize>>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let poll_result = self.0.poll_ready(cx);
            match poll_result {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(ev)) => {
                    // Since this future uses a reusable Aio, we must clear
                    // its readiness here.  That makes the future
                    // non-idempotent; the caller can't poll it repeatedly after
                    // it has already returned Ready.  But that's ok; most
                    // futures behave this way.
                    self.0.clear_ready(ev);
                    let r = (*self.0).liocb.take().unwrap().into_results(|iter| {
                        iter.map(|lr| lr.result.unwrap()).collect::<Vec<isize>>()
                    });
                    Poll::Ready(Ok(r))
                }
            }
        }
    }

    /// An lio_listio operation with one write element
    #[tokio::test]
    async fn onewrite() {
        const WBUF: &[u8] = b"abcdef";
        let f = tempfile().unwrap();

        let mut builder = mio_aio::LioCbBuilder::with_capacity(1);
        builder = builder.emplace_slice(
            f.as_raw_fd(),
            0,
            &WBUF[..],
            0,
            mio_aio::LioOpcode::LIO_WRITE,
        );
        let liocb = builder.finish();
        let source = WrappedLioCb(liocb);
        let mut poll_aio = Aio::new_for_lio(source).unwrap();

        // Send the operation to the kernel
        (*poll_aio).0.submit().unwrap();
        let fut = LioFut(Some(poll_aio));
        let v = fut.await.unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0] as usize, WBUF.len());
    }

    /// A suitably crafted future type can reuse an Aio object
    #[tokio::test]
    async fn reuse() {
        const WBUF: &[u8] = b"abcdef";
        let f = tempfile().unwrap();

        let mut builder0 = mio_aio::LioCbBuilder::with_capacity(1);
        builder0 = builder0.emplace_slice(
            f.as_raw_fd(),
            0,
            &WBUF[..],
            0,
            mio_aio::LioOpcode::LIO_WRITE,
        );
        let liocb0 = builder0.finish();
        let source = ReusableLioSource::new(liocb0);
        let mut poll_aio = Aio::new_for_aio(source).unwrap();
        poll_aio.submit();
        let fut0 = ReusableLioFut(&mut poll_aio);
        let v = fut0.await.unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0] as usize, WBUF.len());

        // Now reuse the same Aio
        let mut builder1 = mio_aio::LioCbBuilder::with_capacity(1);
        builder1 = builder1.emplace_slice(
            f.as_raw_fd(),
            0,
            &WBUF[..],
            0,
            mio_aio::LioOpcode::LIO_WRITE,
        );
        let liocb1 = builder1.finish();
        poll_aio.reset(liocb1);
        let mut ctx = Context::from_waker(futures::task::noop_waker_ref());
        assert_pending!(poll_aio.poll_ready(&mut ctx));
        poll_aio.submit();
        let fut1 = ReusableLioFut(&mut poll_aio);
        let v = fut1.await.unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0] as usize, WBUF.len());
    }
}
