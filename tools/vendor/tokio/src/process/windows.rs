//! Windows asynchronous process handling.
//!
//! Like with Unix we don't actually have a way of registering a process with an
//! IOCP object. As a result we similarly need another mechanism for getting a
//! signal when a process has exited. For now this is implemented with the
//! `RegisterWaitForSingleObject` function in the kernel32.dll.
//!
//! This strategy is the same that libuv takes and essentially just queues up a
//! wait for the process in a kernel32-specific thread pool. Once the object is
//! notified (e.g. the process exits) then we have a callback that basically
//! just completes a `Oneshot`.
//!
//! The `poll_exit` implementation will attempt to wait for the process in a
//! nonblocking fashion, but failing that it'll fire off a
//! `RegisterWaitForSingleObject` and then wait on the other end of the oneshot
//! from then on out.

use crate::io::PollEvented;
use crate::process::kill::Kill;
use crate::process::SpawnedChild;
use crate::sync::oneshot;

use mio::windows::NamedPipe;
use std::fmt;
use std::future::Future;
use std::io;
use std::os::windows::prelude::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle};
use std::pin::Pin;
use std::process::Stdio;
use std::process::{Child as StdChild, Command as StdCommand, ExitStatus};
use std::ptr;
use std::task::Context;
use std::task::Poll;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::um::handleapi::{DuplicateHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::threadpoollegacyapiset::UnregisterWaitEx;
use winapi::um::winbase::{RegisterWaitForSingleObject, INFINITE};
use winapi::um::winnt::{
    BOOLEAN, DUPLICATE_SAME_ACCESS, HANDLE, PVOID, WT_EXECUTEINWAITTHREAD, WT_EXECUTEONLYONCE,
};

#[must_use = "futures do nothing unless polled"]
pub(crate) struct Child {
    child: StdChild,
    waiting: Option<Waiting>,
}

impl fmt::Debug for Child {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Child")
            .field("pid", &self.id())
            .field("child", &self.child)
            .field("waiting", &"..")
            .finish()
    }
}

struct Waiting {
    rx: oneshot::Receiver<()>,
    wait_object: HANDLE,
    tx: *mut Option<oneshot::Sender<()>>,
}

unsafe impl Sync for Waiting {}
unsafe impl Send for Waiting {}

pub(crate) fn spawn_child(cmd: &mut StdCommand) -> io::Result<SpawnedChild> {
    let mut child = cmd.spawn()?;
    let stdin = child.stdin.take().map(stdio).transpose()?;
    let stdout = child.stdout.take().map(stdio).transpose()?;
    let stderr = child.stderr.take().map(stdio).transpose()?;

    Ok(SpawnedChild {
        child: Child {
            child,
            waiting: None,
        },
        stdin,
        stdout,
        stderr,
    })
}

impl Child {
    pub(crate) fn id(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }
}

impl Kill for Child {
    fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }
}

impl Future for Child {
    type Output = io::Result<ExitStatus>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = Pin::get_mut(self);
        loop {
            if let Some(ref mut w) = inner.waiting {
                match Pin::new(&mut w.rx).poll(cx) {
                    Poll::Ready(Ok(())) => {}
                    Poll::Ready(Err(_)) => panic!("should not be canceled"),
                    Poll::Pending => return Poll::Pending,
                }
                let status = inner.try_wait()?.expect("not ready yet");
                return Poll::Ready(Ok(status));
            }

            if let Some(e) = inner.try_wait()? {
                return Poll::Ready(Ok(e));
            }
            let (tx, rx) = oneshot::channel();
            let ptr = Box::into_raw(Box::new(Some(tx)));
            let mut wait_object = ptr::null_mut();
            let rc = unsafe {
                RegisterWaitForSingleObject(
                    &mut wait_object,
                    inner.child.as_raw_handle(),
                    Some(callback),
                    ptr as *mut _,
                    INFINITE,
                    WT_EXECUTEINWAITTHREAD | WT_EXECUTEONLYONCE,
                )
            };
            if rc == 0 {
                let err = io::Error::last_os_error();
                drop(unsafe { Box::from_raw(ptr) });
                return Poll::Ready(Err(err));
            }
            inner.waiting = Some(Waiting {
                rx,
                wait_object,
                tx: ptr,
            });
        }
    }
}

impl AsRawHandle for Child {
    fn as_raw_handle(&self) -> RawHandle {
        self.child.as_raw_handle()
    }
}

impl Drop for Waiting {
    fn drop(&mut self) {
        unsafe {
            let rc = UnregisterWaitEx(self.wait_object, INVALID_HANDLE_VALUE);
            if rc == 0 {
                panic!("failed to unregister: {}", io::Error::last_os_error());
            }
            drop(Box::from_raw(self.tx));
        }
    }
}

unsafe extern "system" fn callback(ptr: PVOID, _timer_fired: BOOLEAN) {
    let complete = &mut *(ptr as *mut Option<oneshot::Sender<()>>);
    let _ = complete.take().unwrap().send(());
}

pub(crate) type ChildStdio = PollEvented<NamedPipe>;

pub(super) fn stdio<T>(io: T) -> io::Result<PollEvented<NamedPipe>>
where
    T: IntoRawHandle,
{
    let pipe = unsafe { NamedPipe::from_raw_handle(io.into_raw_handle()) };
    PollEvented::new(pipe)
}

pub(crate) fn convert_to_stdio(io: PollEvented<NamedPipe>) -> io::Result<Stdio> {
    let named_pipe = io.into_inner()?;

    // Mio does not implement `IntoRawHandle` for `NamedPipe`, so we'll manually
    // duplicate the handle here...
    unsafe {
        let mut dup_handle = INVALID_HANDLE_VALUE;
        let cur_proc = GetCurrentProcess();

        let status = DuplicateHandle(
            cur_proc,
            named_pipe.as_raw_handle(),
            cur_proc,
            &mut dup_handle,
            0 as DWORD,
            FALSE,
            DUPLICATE_SAME_ACCESS,
        );

        if status == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Stdio::from_raw_handle(dup_handle))
    }
}
