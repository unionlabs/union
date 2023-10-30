//! Unix handling of child processes.
//!
//! Right now the only "fancy" thing about this is how we implement the
//! `Future` implementation on `Child` to get the exit status. Unix offers
//! no way to register a child with epoll, and the only real way to get a
//! notification when a process exits is the SIGCHLD signal.
//!
//! Signal handling in general is *super* hairy and complicated, and it's even
//! more complicated here with the fact that signals are coalesced, so we may
//! not get a SIGCHLD-per-child.
//!
//! Our best approximation here is to check *all spawned processes* for all
//! SIGCHLD signals received. To do that we create a `Signal`, implemented in
//! the `tokio-net` crate, which is a stream over signals being received.
//!
//! Later when we poll the process's exit status we simply check to see if a
//! SIGCHLD has happened since we last checked, and while that returns "yes" we
//! keep trying.
//!
//! Note that this means that this isn't really scalable, but then again
//! processes in general aren't scalable (e.g. millions) so it shouldn't be that
//! bad in theory...

pub(crate) mod driver;

pub(crate) mod orphan;
use orphan::{OrphanQueue, OrphanQueueImpl, Wait};

mod reap;
use reap::Reaper;

use crate::io::PollEvented;
use crate::process::kill::Kill;
use crate::process::SpawnedChild;
use crate::signal::unix::driver::Handle as SignalHandle;
use crate::signal::unix::{signal, Signal, SignalKind};

use mio::event::Source;
use mio::unix::SourceFd;
use once_cell::sync::Lazy;
use std::fmt;
use std::fs::File;
use std::future::Future;
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::pin::Pin;
use std::process::{Child as StdChild, ExitStatus, Stdio};
use std::task::Context;
use std::task::Poll;

impl Wait for StdChild {
    fn id(&self) -> u32 {
        self.id()
    }

    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.try_wait()
    }
}

impl Kill for StdChild {
    fn kill(&mut self) -> io::Result<()> {
        self.kill()
    }
}

static ORPHAN_QUEUE: Lazy<OrphanQueueImpl<StdChild>> = Lazy::new(OrphanQueueImpl::new);

pub(crate) struct GlobalOrphanQueue;

impl fmt::Debug for GlobalOrphanQueue {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        ORPHAN_QUEUE.fmt(fmt)
    }
}

impl GlobalOrphanQueue {
    fn reap_orphans(handle: &SignalHandle) {
        ORPHAN_QUEUE.reap_orphans(handle)
    }
}

impl OrphanQueue<StdChild> for GlobalOrphanQueue {
    fn push_orphan(&self, orphan: StdChild) {
        ORPHAN_QUEUE.push_orphan(orphan)
    }
}

#[must_use = "futures do nothing unless polled"]
pub(crate) struct Child {
    inner: Reaper<StdChild, GlobalOrphanQueue, Signal>,
}

impl fmt::Debug for Child {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Child")
            .field("pid", &self.inner.id())
            .finish()
    }
}

pub(crate) fn spawn_child(cmd: &mut std::process::Command) -> io::Result<SpawnedChild> {
    let mut child = cmd.spawn()?;
    let stdin = child.stdin.take().map(stdio).transpose()?;
    let stdout = child.stdout.take().map(stdio).transpose()?;
    let stderr = child.stderr.take().map(stdio).transpose()?;

    let signal = signal(SignalKind::child())?;

    Ok(SpawnedChild {
        child: Child {
            inner: Reaper::new(child, GlobalOrphanQueue, signal),
        },
        stdin,
        stdout,
        stderr,
    })
}

impl Child {
    pub(crate) fn id(&self) -> u32 {
        self.inner.id()
    }

    pub(crate) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.inner.inner_mut().try_wait()
    }
}

impl Kill for Child {
    fn kill(&mut self) -> io::Result<()> {
        self.inner.kill()
    }
}

impl Future for Child {
    type Output = io::Result<ExitStatus>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

#[derive(Debug)]
pub(crate) struct Pipe {
    // Actually a pipe and not a File. However, we are reusing `File` to get
    // close on drop. This is a similar trick as `mio`.
    fd: File,
}

impl<T: IntoRawFd> From<T> for Pipe {
    fn from(fd: T) -> Self {
        let fd = unsafe { File::from_raw_fd(fd.into_raw_fd()) };
        Self { fd }
    }
}

impl<'a> io::Read for &'a Pipe {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        (&self.fd).read(bytes)
    }
}

impl<'a> io::Write for &'a Pipe {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        (&self.fd).write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.fd).flush()
    }
}

impl AsRawFd for Pipe {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

pub(crate) fn convert_to_stdio(io: PollEvented<Pipe>) -> io::Result<Stdio> {
    let mut fd = io.into_inner()?.fd;

    // Ensure that the fd to be inherited is set to *blocking* mode, as this
    // is the default that virtually all programs expect to have. Those
    // programs that know how to work with nonblocking stdio will know how to
    // change it to nonblocking mode.
    set_nonblocking(&mut fd, false)?;

    Ok(Stdio::from(fd))
}

impl Source for Pipe {
    fn register(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        interest: mio::Interest,
    ) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).register(registry, token, interest)
    }

    fn reregister(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        interest: mio::Interest,
    ) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).reregister(registry, token, interest)
    }

    fn deregister(&mut self, registry: &mio::Registry) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).deregister(registry)
    }
}

pub(crate) type ChildStdio = PollEvented<Pipe>;

fn set_nonblocking<T: AsRawFd>(fd: &mut T, nonblocking: bool) -> io::Result<()> {
    unsafe {
        let fd = fd.as_raw_fd();
        let previous = libc::fcntl(fd, libc::F_GETFL);
        if previous == -1 {
            return Err(io::Error::last_os_error());
        }

        let new = if nonblocking {
            previous | libc::O_NONBLOCK
        } else {
            previous & !libc::O_NONBLOCK
        };

        let r = libc::fcntl(fd, libc::F_SETFL, new);
        if r == -1 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(())
}

pub(super) fn stdio<T>(io: T) -> io::Result<PollEvented<Pipe>>
where
    T: IntoRawFd,
{
    // Set the fd to nonblocking before we pass it to the event loop
    let mut pipe = Pipe::from(io);
    set_nonblocking(&mut pipe, true)?;

    PollEvented::new(pipe)
}
