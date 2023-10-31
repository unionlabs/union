#![warn(rust_2018_idioms)]
#![cfg(all(unix, feature = "full"))]

use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use std::{
    future::Future,
    io::{self, ErrorKind, Read, Write},
    task::{Context, Waker},
};

use nix::unistd::{close, read, write};

use futures::poll;

use tokio::io::unix::{AsyncFd, AsyncFdReadyGuard};
use tokio_test::{assert_err, assert_pending};

struct TestWaker {
    inner: Arc<TestWakerInner>,
    waker: Waker,
}

#[derive(Default)]
struct TestWakerInner {
    awoken: AtomicBool,
}

impl futures::task::ArcWake for TestWakerInner {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.awoken.store(true, Ordering::SeqCst);
    }
}

impl TestWaker {
    fn new() -> Self {
        let inner: Arc<TestWakerInner> = Default::default();

        Self {
            inner: inner.clone(),
            waker: futures::task::waker(inner),
        }
    }

    fn awoken(&self) -> bool {
        self.inner.awoken.swap(false, Ordering::SeqCst)
    }

    fn context(&self) -> Context<'_> {
        Context::from_waker(&self.waker)
    }
}

#[derive(Debug)]
struct FileDescriptor {
    fd: RawFd,
}

impl AsRawFd for FileDescriptor {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Read for &FileDescriptor {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        read(self.fd, buf).map_err(io::Error::from)
    }
}

impl Read for FileDescriptor {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (self as &Self).read(buf)
    }
}

impl Write for &FileDescriptor {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write(self.fd, buf).map_err(io::Error::from)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Write for FileDescriptor {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (self as &Self).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (self as &Self).flush()
    }
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        let _ = close(self.fd);
    }
}

fn set_nonblocking(fd: RawFd) {
    use nix::fcntl::{OFlag, F_GETFL, F_SETFL};

    let flags = nix::fcntl::fcntl(fd, F_GETFL).expect("fcntl(F_GETFD)");

    if flags < 0 {
        panic!(
            "bad return value from fcntl(F_GETFL): {} ({:?})",
            flags,
            nix::Error::last()
        );
    }

    let flags = OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK;

    nix::fcntl::fcntl(fd, F_SETFL(flags)).expect("fcntl(F_SETFD)");
}

fn socketpair() -> (FileDescriptor, FileDescriptor) {
    use nix::sys::socket::{self, AddressFamily, SockFlag, SockType};

    let (fd_a, fd_b) = socket::socketpair(
        AddressFamily::Unix,
        SockType::Stream,
        None,
        SockFlag::empty(),
    )
    .expect("socketpair");
    let fds = (FileDescriptor { fd: fd_a }, FileDescriptor { fd: fd_b });

    set_nonblocking(fds.0.fd);
    set_nonblocking(fds.1.fd);

    fds
}

fn drain(mut fd: &FileDescriptor) {
    let mut buf = [0u8; 512];

    loop {
        match fd.read(&mut buf[..]) {
            Err(e) if e.kind() == ErrorKind::WouldBlock => break,
            Ok(0) => panic!("unexpected EOF"),
            Err(e) => panic!("unexpected error: {:?}", e),
            Ok(_) => continue,
        }
    }
}

#[tokio::test]
async fn initially_writable() {
    let (a, b) = socketpair();

    let afd_a = AsyncFd::new(a).unwrap();
    let afd_b = AsyncFd::new(b).unwrap();

    afd_a.writable().await.unwrap().clear_ready();
    afd_b.writable().await.unwrap().clear_ready();

    tokio::select! {
        biased;
        _ = tokio::time::sleep(Duration::from_millis(10)) => {},
        _ = afd_a.readable() => panic!("Unexpected readable state"),
        _ = afd_b.readable() => panic!("Unexpected readable state"),
    }
}

#[tokio::test]
async fn reset_readable() {
    let (a, mut b) = socketpair();

    let afd_a = AsyncFd::new(a).unwrap();

    let readable = afd_a.readable();
    tokio::pin!(readable);

    tokio::select! {
        _ = readable.as_mut() => panic!(),
        _ = tokio::time::sleep(Duration::from_millis(10)) => {}
    }

    b.write_all(b"0").unwrap();

    let mut guard = readable.await.unwrap();

    guard
        .try_io(|_| afd_a.get_ref().read(&mut [0]))
        .unwrap()
        .unwrap();

    // `a` is not readable, but the reactor still thinks it is
    // (because we have not observed a not-ready error yet)
    afd_a.readable().await.unwrap().retain_ready();

    // Explicitly clear the ready state
    guard.clear_ready();

    let readable = afd_a.readable();
    tokio::pin!(readable);

    tokio::select! {
        _ = readable.as_mut() => panic!(),
        _ = tokio::time::sleep(Duration::from_millis(10)) => {}
    }

    b.write_all(b"0").unwrap();

    // We can observe the new readable event
    afd_a.readable().await.unwrap().clear_ready();
}

#[tokio::test]
async fn reset_writable() {
    let (a, b) = socketpair();

    let afd_a = AsyncFd::new(a).unwrap();

    let mut guard = afd_a.writable().await.unwrap();

    // Write until we get a WouldBlock. This also clears the ready state.
    while guard
        .try_io(|_| afd_a.get_ref().write(&[0; 512][..]))
        .is_ok()
    {}

    // Writable state should be cleared now.
    let writable = afd_a.writable();
    tokio::pin!(writable);

    tokio::select! {
        _ = writable.as_mut() => panic!(),
        _ = tokio::time::sleep(Duration::from_millis(10)) => {}
    }

    // Read from the other side; we should become writable now.
    drain(&b);

    let _ = writable.await.unwrap();
}

#[derive(Debug)]
struct ArcFd<T>(Arc<T>);
impl<T: AsRawFd> AsRawFd for ArcFd<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[tokio::test]
async fn drop_closes() {
    let (a, mut b) = socketpair();

    let afd_a = AsyncFd::new(a).unwrap();

    assert_eq!(
        ErrorKind::WouldBlock,
        b.read(&mut [0]).err().unwrap().kind()
    );

    std::mem::drop(afd_a);

    assert_eq!(0, b.read(&mut [0]).unwrap());

    // into_inner does not close the fd

    let (a, mut b) = socketpair();
    let afd_a = AsyncFd::new(a).unwrap();
    let _a: FileDescriptor = afd_a.into_inner();

    assert_eq!(
        ErrorKind::WouldBlock,
        b.read(&mut [0]).err().unwrap().kind()
    );

    // Drop closure behavior is delegated to the inner object
    let (a, mut b) = socketpair();
    let arc_fd = Arc::new(a);
    let afd_a = AsyncFd::new(ArcFd(arc_fd.clone())).unwrap();
    std::mem::drop(afd_a);

    assert_eq!(
        ErrorKind::WouldBlock,
        b.read(&mut [0]).err().unwrap().kind()
    );

    std::mem::drop(arc_fd); // suppress unnecessary clone clippy warning
}

#[tokio::test]
async fn reregister() {
    let (a, _b) = socketpair();

    let afd_a = AsyncFd::new(a).unwrap();
    let a = afd_a.into_inner();
    AsyncFd::new(a).unwrap();
}

#[tokio::test]
async fn try_io() {
    let (a, mut b) = socketpair();

    b.write_all(b"0").unwrap();

    let afd_a = AsyncFd::new(a).unwrap();

    let mut guard = afd_a.readable().await.unwrap();

    afd_a.get_ref().read_exact(&mut [0]).unwrap();

    // Should not clear the readable state
    let _ = guard.try_io(|_| Ok(()));

    // Still readable...
    let _ = afd_a.readable().await.unwrap();

    // Should clear the readable state
    let _ = guard.try_io(|_| io::Result::<()>::Err(ErrorKind::WouldBlock.into()));

    // Assert not readable
    let readable = afd_a.readable();
    tokio::pin!(readable);

    tokio::select! {
        _ = readable.as_mut() => panic!(),
        _ = tokio::time::sleep(Duration::from_millis(10)) => {}
    }

    // Write something down b again and make sure we're reawoken
    b.write_all(b"0").unwrap();
    let _ = readable.await.unwrap();
}

#[tokio::test]
async fn multiple_waiters() {
    let (a, mut b) = socketpair();
    let afd_a = Arc::new(AsyncFd::new(a).unwrap());

    let barrier = Arc::new(tokio::sync::Barrier::new(11));

    let mut tasks = Vec::new();
    for _ in 0..10 {
        let afd_a = afd_a.clone();
        let barrier = barrier.clone();

        let f = async move {
            let notify_barrier = async {
                barrier.wait().await;
                futures::future::pending::<()>().await;
            };

            tokio::select! {
                biased;
                guard = afd_a.readable() => {
                    tokio::task::yield_now().await;
                    guard.unwrap().clear_ready()
                },
                _ = notify_barrier => unreachable!(),
            }

            std::mem::drop(afd_a);
        };

        tasks.push(tokio::spawn(f));
    }

    let mut all_tasks = futures::future::try_join_all(tasks);

    tokio::select! {
        r = std::pin::Pin::new(&mut all_tasks) => {
            r.unwrap(); // propagate panic
            panic!("Tasks exited unexpectedly")
        },
        _ = barrier.wait() => {}
    };

    b.write_all(b"0").unwrap();

    all_tasks.await.unwrap();
}

#[tokio::test]
async fn poll_fns() {
    let (a, b) = socketpair();
    let afd_a = Arc::new(AsyncFd::new(a).unwrap());
    let afd_b = Arc::new(AsyncFd::new(b).unwrap());

    // Fill up the write side of A
    while afd_a.get_ref().write(&[0; 512]).is_ok() {}

    let waker = TestWaker::new();

    assert_pending!(afd_a.as_ref().poll_read_ready(&mut waker.context()));

    let afd_a_2 = afd_a.clone();
    let r_barrier = Arc::new(tokio::sync::Barrier::new(2));
    let barrier_clone = r_barrier.clone();

    let read_fut = tokio::spawn(async move {
        // Move waker onto this task first
        assert_pending!(poll!(futures::future::poll_fn(|cx| afd_a_2
            .as_ref()
            .poll_read_ready(cx))));
        barrier_clone.wait().await;

        let _ = futures::future::poll_fn(|cx| afd_a_2.as_ref().poll_read_ready(cx)).await;
    });

    let afd_a_2 = afd_a.clone();
    let w_barrier = Arc::new(tokio::sync::Barrier::new(2));
    let barrier_clone = w_barrier.clone();

    let mut write_fut = tokio::spawn(async move {
        // Move waker onto this task first
        assert_pending!(poll!(futures::future::poll_fn(|cx| afd_a_2
            .as_ref()
            .poll_write_ready(cx))));
        barrier_clone.wait().await;

        let _ = futures::future::poll_fn(|cx| afd_a_2.as_ref().poll_write_ready(cx)).await;
    });

    r_barrier.wait().await;
    w_barrier.wait().await;

    let readable = afd_a.readable();
    tokio::pin!(readable);

    tokio::select! {
        _ = &mut readable => unreachable!(),
        _ = tokio::task::yield_now() => {}
    }

    // Make A readable. We expect that 'readable' and 'read_fut' will both complete quickly
    afd_b.get_ref().write_all(b"0").unwrap();

    let _ = tokio::join!(readable, read_fut);

    // Our original waker should _not_ be awoken (poll_read_ready retains only the last context)
    assert!(!waker.awoken());

    // The writable side should not be awoken
    tokio::select! {
        _ = &mut write_fut => unreachable!(),
        _ = tokio::time::sleep(Duration::from_millis(5)) => {}
    }

    // Make it writable now
    drain(afd_b.get_ref());

    // now we should be writable (ie - the waker for poll_write should still be registered after we wake the read side)
    let _ = write_fut.await;
}

fn assert_pending<T: std::fmt::Debug, F: Future<Output = T>>(f: F) -> std::pin::Pin<Box<F>> {
    let mut pinned = Box::pin(f);

    assert_pending!(pinned
        .as_mut()
        .poll(&mut Context::from_waker(futures::task::noop_waker_ref())));

    pinned
}

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn driver_shutdown_wakes_currently_pending() {
    let rt = rt();

    let (a, _b) = socketpair();
    let afd_a = {
        let _enter = rt.enter();
        AsyncFd::new(a).unwrap()
    };

    let readable = assert_pending(afd_a.readable());

    std::mem::drop(rt);

    // The future was initialized **before** dropping the rt
    assert_err!(futures::executor::block_on(readable));

    // The future is initialized **after** dropping the rt.
    assert_err!(futures::executor::block_on(afd_a.readable()));
}

#[test]
fn driver_shutdown_wakes_future_pending() {
    let rt = rt();

    let (a, _b) = socketpair();
    let afd_a = {
        let _enter = rt.enter();
        AsyncFd::new(a).unwrap()
    };

    std::mem::drop(rt);

    assert_err!(futures::executor::block_on(afd_a.readable()));
}

#[test]
fn driver_shutdown_wakes_pending_race() {
    // TODO: make this a loom test
    for _ in 0..100 {
        let rt = rt();

        let (a, _b) = socketpair();
        let afd_a = {
            let _enter = rt.enter();
            AsyncFd::new(a).unwrap()
        };

        let _ = std::thread::spawn(move || std::mem::drop(rt));

        // This may or may not return an error (but will be awoken)
        let _ = futures::executor::block_on(afd_a.readable());

        // However retrying will always return an error
        assert_err!(futures::executor::block_on(afd_a.readable()));
    }
}

async fn poll_readable<T: AsRawFd>(fd: &AsyncFd<T>) -> std::io::Result<AsyncFdReadyGuard<'_, T>> {
    futures::future::poll_fn(|cx| fd.poll_read_ready(cx)).await
}

async fn poll_writable<T: AsRawFd>(fd: &AsyncFd<T>) -> std::io::Result<AsyncFdReadyGuard<'_, T>> {
    futures::future::poll_fn(|cx| fd.poll_write_ready(cx)).await
}

#[test]
fn driver_shutdown_wakes_currently_pending_polls() {
    let rt = rt();

    let (a, _b) = socketpair();
    let afd_a = {
        let _enter = rt.enter();
        AsyncFd::new(a).unwrap()
    };

    while afd_a.get_ref().write(&[0; 512]).is_ok() {} // make not writable

    let readable = assert_pending(poll_readable(&afd_a));
    let writable = assert_pending(poll_writable(&afd_a));

    std::mem::drop(rt);

    // Attempting to poll readiness when the rt is dropped is an error
    assert_err!(futures::executor::block_on(readable));
    assert_err!(futures::executor::block_on(writable));
}

#[test]
fn driver_shutdown_wakes_poll() {
    let rt = rt();

    let (a, _b) = socketpair();
    let afd_a = {
        let _enter = rt.enter();
        AsyncFd::new(a).unwrap()
    };

    std::mem::drop(rt);

    assert_err!(futures::executor::block_on(poll_readable(&afd_a)));
    assert_err!(futures::executor::block_on(poll_writable(&afd_a)));
}

#[test]
fn driver_shutdown_wakes_poll_race() {
    // TODO: make this a loom test
    for _ in 0..100 {
        let rt = rt();

        let (a, _b) = socketpair();
        let afd_a = {
            let _enter = rt.enter();
            AsyncFd::new(a).unwrap()
        };

        while afd_a.get_ref().write(&[0; 512]).is_ok() {} // make not writable

        let _ = std::thread::spawn(move || std::mem::drop(rt));

        // The poll variants will always return an error in this case
        assert_err!(futures::executor::block_on(poll_readable(&afd_a)));
        assert_err!(futures::executor::block_on(poll_writable(&afd_a)));
    }
}
