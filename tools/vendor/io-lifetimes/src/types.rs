use std::fmt;
use std::marker::PhantomData;
use std::mem::forget;
#[cfg(target_os = "hermit")]
use std::os::hermit::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
#[cfg(windows)]
use std::{
    convert::TryFrom,
    os::windows::io::{
        AsRawHandle, AsRawSocket, FromRawHandle, FromRawSocket, IntoRawHandle, IntoRawSocket,
        RawHandle, RawSocket,
    },
};
#[cfg(all(windows, feature = "close"))]
use {
    windows_sys::Win32::Foundation::{
        CloseHandle, DuplicateHandle, SetHandleInformation, BOOL, DUPLICATE_HANDLE_OPTIONS,
        DUPLICATE_SAME_ACCESS, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
    },
    windows_sys::Win32::Networking::WinSock::{
        closesocket, WSADuplicateSocketW, WSAGetLastError, WSASocketW, INVALID_SOCKET, SOCKET,
        SOCKET_ERROR, WSAEINVAL, WSAEPROTOTYPE, WSAPROTOCOL_INFOW, WSA_FLAG_NO_HANDLE_INHERIT,
        WSA_FLAG_OVERLAPPED,
    },
    windows_sys::Win32::System::Threading::{GetCurrentProcess, GetCurrentProcessId},
};

#[cfg(all(windows, not(feature = "close")))]
type HANDLE = isize;
#[cfg(all(windows, not(feature = "close")))]
const INVALID_HANDLE_VALUE: HANDLE = !0 as _;
#[cfg(all(windows, not(feature = "close")))]
const INVALID_SOCKET: usize = !0 as _;

/// A borrowed file descriptor.
///
/// This has a lifetime parameter to tie it to the lifetime of something that
/// owns the file descriptor.
///
/// This uses `repr(transparent)` and has the representation of a host file
/// descriptor, so it can be used in FFI in places where a file descriptor is
/// passed as an argument, it is not captured or consumed, and it never has the
/// value `-1`.
///
/// This type's `.to_owned()` implementation returns another `BorrowedFd`
/// rather than an `OwnedFd`. It just makes a trivial copy of the raw file
/// descriptor, which is then borrowed under the same lifetime.
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct BorrowedFd<'fd> {
    fd: RawFd,
    _phantom: PhantomData<&'fd OwnedFd>,
}

/// A borrowed handle.
///
/// This has a lifetime parameter to tie it to the lifetime of something that
/// owns the handle.
///
/// This uses `repr(transparent)` and has the representation of a host handle,
/// so it can be used in FFI in places where a handle is passed as an argument,
/// it is not captured or consumed, and it is never null.
///
/// Note that it *may* have the value `-1`, which in `BorrowedHandle` always
/// represents a valid handle value, such as [the current process handle], and
/// not `INVALID_HANDLE_VALUE`, despite the two having the same value. See
/// [here] for the full story.
///
/// This type's `.to_owned()` implementation returns another `BorrowedHandle`
/// rather than an `OwnedHandle`. It just makes a trivial copy of the raw
/// handle, which is then borrowed under the same lifetime.
///
/// [here]: https://devblogs.microsoft.com/oldnewthing/20040302-00/?p=40443
/// [the current process handle]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentprocess#remarks
#[cfg(windows)]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct BorrowedHandle<'handle> {
    handle: RawHandle,
    _phantom: PhantomData<&'handle OwnedHandle>,
}

/// A borrowed socket.
///
/// This has a lifetime parameter to tie it to the lifetime of something that
/// owns the socket.
///
/// This uses `repr(transparent)` and has the representation of a host socket,
/// so it can be used in FFI in places where a socket is passed as an argument,
/// it is not captured or consumed, and it never has the value
/// [`INVALID_SOCKET`].
///
/// This type's `.to_owned()` implementation returns another `BorrowedSocket`
/// rather than an `OwnedSocket`. It just makes a trivial copy of the raw
/// socket, which is then borrowed under the same lifetime.
#[cfg(windows)]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct BorrowedSocket<'socket> {
    socket: RawSocket,
    _phantom: PhantomData<&'socket OwnedSocket>,
}

/// An owned file descriptor.
///
/// This closes the file descriptor on drop.
///
/// This uses `repr(transparent)` and has the representation of a host file
/// descriptor, so it can be used in FFI in places where a file descriptor is
/// passed as a consumed argument or returned as an owned value, and it never
/// has the value `-1`.
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
#[repr(transparent)]
pub struct OwnedFd {
    fd: RawFd,
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl OwnedFd {
    /// Creates a new `OwnedFd` instance that shares the same underlying file
    /// description as the existing `OwnedFd` instance.
    pub fn try_clone(&self) -> std::io::Result<Self> {
        crate::AsFd::as_fd(self).try_clone_to_owned()
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl BorrowedFd<'_> {
    /// Creates a new `OwnedFd` instance that shares the same underlying file
    /// description as the existing `BorrowedFd` instance.
    pub fn try_clone_to_owned(&self) -> std::io::Result<OwnedFd> {
        #[cfg(feature = "close")]
        {
            #[cfg(unix)]
            {
                // We want to atomically duplicate this file descriptor and set the
                // CLOEXEC flag, and currently that's done via F_DUPFD_CLOEXEC. This
                // is a POSIX flag that was added to Linux in 2.6.24.
                #[cfg(not(target_os = "espidf"))]
                let cmd = libc::F_DUPFD_CLOEXEC;

                // For ESP-IDF, F_DUPFD is used instead, because the CLOEXEC semantics
                // will never be supported, as this is a bare metal framework with
                // no capabilities for multi-process execution.  While F_DUPFD is also
                // not supported yet, it might be (currently it returns ENOSYS).
                #[cfg(target_os = "espidf")]
                let cmd = libc::F_DUPFD;

                let fd = match unsafe { libc::fcntl(self.as_raw_fd(), cmd, 0) } {
                    -1 => return Err(std::io::Error::last_os_error()),
                    fd => fd,
                };

                Ok(unsafe { OwnedFd::from_raw_fd(fd) })
            }

            #[cfg(any(target_os = "wasi", target_os = "hermit"))]
            {
                unreachable!("try_clone is not yet suppported on wasi");
            }
        }

        // If the `close` feature is disabled, we expect users to avoid cloning
        // `OwnedFd` instances, so that we don't have to call `fcntl`.
        #[cfg(not(feature = "close"))]
        {
            unreachable!("try_clone_to_owned called without the \"close\" feature in io-lifetimes");
        }
    }
}

/// An owned handle.
///
/// This closes the handle on drop.
///
/// Note that it *may* have the value `-1`, which in `OwnedHandle` always
/// represents a valid handle value, such as [the current process handle], and
/// not `INVALID_HANDLE_VALUE`, despite the two having the same value. See
/// [here] for the full story.
///
/// And, it *may* have the value `NULL` (0), which can occur when consoles are
/// detached from processes, or when `windows_subsystem` is used.
///
/// `OwnedHandle` uses [`CloseHandle`] to close its handle on drop. As such,
/// it must not be used with handles to open registry keys which need to be
/// closed with [`RegCloseKey`] instead.
///
/// [`CloseHandle`]: https://docs.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle
/// [`RegCloseKey`]: https://docs.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regclosekey
///
/// [here]: https://devblogs.microsoft.com/oldnewthing/20040302-00/?p=40443
/// [the current process handle]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentprocess#remarks
#[cfg(windows)]
#[repr(transparent)]
pub struct OwnedHandle {
    handle: RawHandle,
}

#[cfg(windows)]
impl OwnedHandle {
    /// Creates a new `OwnedHandle` instance that shares the same underlying
    /// object as the existing `OwnedHandle` instance.
    pub fn try_clone(&self) -> std::io::Result<Self> {
        crate::AsHandle::as_handle(self).try_clone_to_owned()
    }
}

#[cfg(windows)]
impl BorrowedHandle<'_> {
    /// Creates a new `OwnedHandle` instance that shares the same underlying
    /// object as the existing `BorrowedHandle` instance.
    pub fn try_clone_to_owned(&self) -> std::io::Result<OwnedHandle> {
        #[cfg(feature = "close")]
        {
            self.duplicate(0, false, DUPLICATE_SAME_ACCESS)
        }

        // If the `close` feature is disabled, we expect users to avoid cloning
        // `OwnedHandle` instances, so that we don't have to call `fcntl`.
        #[cfg(not(feature = "close"))]
        {
            unreachable!("try_clone_to_owned called without the \"close\" feature in io-lifetimes");
        }
    }

    #[cfg(feature = "close")]
    pub(crate) fn duplicate(
        &self,
        access: u32,
        inherit: bool,
        options: DUPLICATE_HANDLE_OPTIONS,
    ) -> std::io::Result<OwnedHandle> {
        let mut ret = 0 as HANDLE;
        match unsafe {
            let cur_proc = GetCurrentProcess();
            DuplicateHandle(
                cur_proc,
                self.as_raw_handle() as HANDLE,
                cur_proc,
                &mut ret,
                access,
                inherit as BOOL,
                options,
            )
        } {
            0 => return Err(std::io::Error::last_os_error()),
            _ => (),
        }
        unsafe { Ok(OwnedHandle::from_raw_handle(ret as RawHandle)) }
    }
}

/// An owned socket.
///
/// This closes the socket on drop.
///
/// This uses `repr(transparent)` and has the representation of a host socket,
/// so it can be used in FFI in places where a socket is passed as a consumed
/// argument or returned as an owned value, and it never has the value
/// [`INVALID_SOCKET`].
#[cfg(windows)]
#[repr(transparent)]
pub struct OwnedSocket {
    socket: RawSocket,
}

#[cfg(windows)]
impl OwnedSocket {
    /// Creates a new `OwnedSocket` instance that shares the same underlying
    /// object as the existing `OwnedSocket` instance.
    pub fn try_clone(&self) -> std::io::Result<Self> {
        crate::AsSocket::as_socket(self).try_clone_to_owned()
    }

    #[cfg(feature = "close")]
    #[cfg(not(target_vendor = "uwp"))]
    fn set_no_inherit(&self) -> std::io::Result<()> {
        match unsafe {
            SetHandleInformation(self.as_raw_socket() as HANDLE, HANDLE_FLAG_INHERIT, 0)
        } {
            0 => return Err(std::io::Error::last_os_error()),
            _ => Ok(()),
        }
    }

    #[cfg(feature = "close")]
    #[cfg(target_vendor = "uwp")]
    fn set_no_inherit(&self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unavailable on UWP",
        ))
    }
}

#[cfg(windows)]
impl BorrowedSocket<'_> {
    /// Creates a new `OwnedSocket` instance that shares the same underlying
    /// object as the existing `BorrowedSocket` instance.
    pub fn try_clone_to_owned(&self) -> std::io::Result<OwnedSocket> {
        #[cfg(feature = "close")]
        {
            let mut info = unsafe { std::mem::zeroed::<WSAPROTOCOL_INFOW>() };
            let result = unsafe {
                WSADuplicateSocketW(self.as_raw_socket() as _, GetCurrentProcessId(), &mut info)
            };
            match result {
                SOCKET_ERROR => return Err(std::io::Error::last_os_error()),
                0 => (),
                _ => panic!(),
            }
            let socket = unsafe {
                WSASocketW(
                    info.iAddressFamily,
                    info.iSocketType,
                    info.iProtocol,
                    &mut info,
                    0,
                    WSA_FLAG_OVERLAPPED | WSA_FLAG_NO_HANDLE_INHERIT,
                )
            };

            if socket != INVALID_SOCKET {
                unsafe { Ok(OwnedSocket::from_raw_socket(socket as _)) }
            } else {
                let error = unsafe { WSAGetLastError() };

                if error != WSAEPROTOTYPE && error != WSAEINVAL {
                    return Err(std::io::Error::from_raw_os_error(error));
                }

                let socket = unsafe {
                    WSASocketW(
                        info.iAddressFamily,
                        info.iSocketType,
                        info.iProtocol,
                        &mut info,
                        0,
                        WSA_FLAG_OVERLAPPED,
                    )
                };

                if socket == INVALID_SOCKET {
                    return Err(std::io::Error::last_os_error());
                }

                unsafe {
                    let socket = OwnedSocket::from_raw_socket(socket as _);
                    socket.set_no_inherit()?;
                    Ok(socket)
                }
            }
        }

        // If the `close` feature is disabled, we expect users to avoid cloning
        // `OwnedSocket` instances, so that we don't have to call `fcntl`.
        #[cfg(not(feature = "close"))]
        {
            unreachable!("try_clone_to_owned called without the \"close\" feature in io-lifetimes");
        }
    }
}

/// FFI type for handles in return values or out parameters, where `INVALID_HANDLE_VALUE` is used
/// as a sentry value to indicate errors, such as in the return value of `CreateFileW`. This uses
/// `repr(transparent)` and has the representation of a host handle, so that it can be used in such
/// FFI declarations.
///
/// The only thing you can usefully do with a `HandleOrInvalid` is to convert it into an
/// `OwnedHandle` using its [`TryFrom`] implementation; this conversion takes care of the check for
/// `INVALID_HANDLE_VALUE`. This ensures that such FFI calls cannot start using the handle without
/// checking for `INVALID_HANDLE_VALUE` first.
///
/// This type may hold any handle value that [`OwnedHandle`] may hold, except that when it holds
/// `-1`, that value is interpreted to mean `INVALID_HANDLE_VALUE`.
///
/// If holds a handle other than `INVALID_HANDLE_VALUE`, it will close the handle on drop.
#[cfg(windows)]
#[repr(transparent)]
#[derive(Debug)]
pub struct HandleOrInvalid(RawHandle);

/// FFI type for handles in return values or out parameters, where `NULL` is used
/// as a sentry value to indicate errors, such as in the return value of `CreateThread`. This uses
/// `repr(transparent)` and has the representation of a host handle, so that it can be used in such
/// FFI declarations.
///
/// The only thing you can usefully do with a `HandleOrNull` is to convert it into an
/// `OwnedHandle` using its [`TryFrom`] implementation; this conversion takes care of the check for
/// `NULL`. This ensures that such FFI calls cannot start using the handle without
/// checking for `NULL` first.
///
/// This type may hold any handle value that [`OwnedHandle`] may hold. As with `OwnedHandle`, when
/// it holds `-1`, that value is interpreted as a valid handle value, such as
/// [the current process handle], and not `INVALID_HANDLE_VALUE`.
///
/// If this holds a non-null handle, it will close the handle on drop.
///
/// [the current process handle]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentprocess#remarks
#[cfg(windows)]
#[repr(transparent)]
#[derive(Debug)]
pub struct HandleOrNull(RawHandle);

// The Windows [`HANDLE`] type may be transferred across and shared between
// thread boundaries (despite containing a `*mut void`, which in general isn't
// `Send` or `Sync`).
//
// [`HANDLE`]: std::os::windows::raw::HANDLE
#[cfg(windows)]
unsafe impl Send for OwnedHandle {}
#[cfg(windows)]
unsafe impl Send for HandleOrInvalid {}
#[cfg(windows)]
unsafe impl Send for HandleOrNull {}
#[cfg(windows)]
unsafe impl Send for BorrowedHandle<'_> {}
#[cfg(windows)]
unsafe impl Sync for OwnedHandle {}
#[cfg(windows)]
unsafe impl Sync for HandleOrInvalid {}
#[cfg(windows)]
unsafe impl Sync for HandleOrNull {}
#[cfg(windows)]
unsafe impl Sync for BorrowedHandle<'_> {}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl BorrowedFd<'_> {
    /// Return a `BorrowedFd` holding the given raw file descriptor.
    ///
    /// # Safety
    ///
    /// The resource pointed to by `raw` must remain open for the duration of
    /// the returned `BorrowedFd`, and it must not have the value `-1`.
    #[inline]
    pub const unsafe fn borrow_raw(fd: RawFd) -> Self {
        #[cfg(panic_in_const_fn)]
        debug_assert!(fd != -1_i32 as RawFd);

        Self {
            fd,
            _phantom: PhantomData,
        }
    }
}

#[cfg(windows)]
impl BorrowedHandle<'_> {
    /// Return a `BorrowedHandle` holding the given raw handle.
    ///
    /// # Safety
    ///
    /// The resource pointed to by `handle` must be a valid open handle, it
    /// must remain open for the duration of the returned `BorrowedHandle`.
    ///
    /// Note that it *may* have the value `INVALID_HANDLE_VALUE` (-1), which is
    /// sometimes a valid handle value. See [here] for the full story.
    ///
    /// And, it *may* have the value `NULL` (0), which can occur when consoles are
    /// detached from processes, or when `windows_subsystem` is used.
    ///
    /// [here]: https://devblogs.microsoft.com/oldnewthing/20040302-00/?p=40443
    #[inline]
    pub const unsafe fn borrow_raw(handle: RawHandle) -> Self {
        Self {
            handle,
            _phantom: PhantomData,
        }
    }
}

#[cfg(windows)]
impl BorrowedSocket<'_> {
    /// Return a `BorrowedSocket` holding the given raw socket.
    ///
    /// # Safety
    ///
    /// The resource pointed to by `raw` must remain open for the duration of
    /// the returned `BorrowedSocket`, and it must not have the value
    /// [`INVALID_SOCKET`].
    #[inline]
    pub const unsafe fn borrow_raw(socket: RawSocket) -> Self {
        #[cfg(panic_in_const_fn)]
        debug_assert!(socket != INVALID_SOCKET as RawSocket);
        Self {
            socket,
            _phantom: PhantomData,
        }
    }
}

#[cfg(windows)]
impl TryFrom<HandleOrInvalid> for OwnedHandle {
    type Error = InvalidHandleError;

    #[inline]
    fn try_from(handle_or_invalid: HandleOrInvalid) -> Result<Self, InvalidHandleError> {
        let raw = handle_or_invalid.0;
        if raw as HANDLE == INVALID_HANDLE_VALUE {
            // Don't call `CloseHandle`; it'd be harmless, except that it could
            // overwrite the `GetLastError` error.
            forget(handle_or_invalid);

            Err(InvalidHandleError(()))
        } else {
            Ok(OwnedHandle { handle: raw })
        }
    }
}

#[cfg(windows)]
impl TryFrom<HandleOrNull> for OwnedHandle {
    type Error = NullHandleError;

    #[inline]
    fn try_from(handle_or_null: HandleOrNull) -> Result<Self, NullHandleError> {
        let raw = handle_or_null.0;
        if raw.is_null() {
            // Don't call `CloseHandle`; it'd be harmless, except that it could
            // overwrite the `GetLastError` error.
            forget(handle_or_null);

            Err(NullHandleError(()))
        } else {
            Ok(OwnedHandle { handle: raw })
        }
    }
}

/// This is the error type used by [`HandleOrNull`] when attempting to convert
/// into a handle, to indicate that the value is null.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullHandleError(());

impl fmt::Display for NullHandleError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        "A HandleOrNull could not be converted to a handle because it was null".fmt(fmt)
    }
}

impl std::error::Error for NullHandleError {}

/// This is the error type used by [`HandleOrInvalid`] when attempting to
/// convert into a handle, to indicate that the value is
/// `INVALID_HANDLE_VALUE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidHandleError(());

impl fmt::Display for InvalidHandleError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        "A HandleOrInvalid could not be converted to a handle because it was INVALID_HANDLE_VALUE"
            .fmt(fmt)
    }
}

impl std::error::Error for InvalidHandleError {}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsRawFd for BorrowedFd<'_> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

#[cfg(windows)]
impl AsRawHandle for BorrowedHandle<'_> {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.handle
    }
}

#[cfg(windows)]
impl AsRawSocket for BorrowedSocket<'_> {
    #[inline]
    fn as_raw_socket(&self) -> RawSocket {
        self.socket
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsRawFd for OwnedFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

#[cfg(windows)]
impl AsRawHandle for OwnedHandle {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.handle
    }
}

#[cfg(windows)]
impl AsRawSocket for OwnedSocket {
    #[inline]
    fn as_raw_socket(&self) -> RawSocket {
        self.socket
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl IntoRawFd for OwnedFd {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        let fd = self.fd;
        forget(self);
        fd
    }
}

#[cfg(windows)]
impl IntoRawHandle for OwnedHandle {
    #[inline]
    fn into_raw_handle(self) -> RawHandle {
        let handle = self.handle;
        forget(self);
        handle
    }
}

#[cfg(windows)]
impl IntoRawSocket for OwnedSocket {
    #[inline]
    fn into_raw_socket(self) -> RawSocket {
        let socket = self.socket;
        forget(self);
        socket
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl FromRawFd for OwnedFd {
    /// Constructs a new instance of `Self` from the given raw file descriptor.
    ///
    /// # Safety
    ///
    /// The resource pointed to by `raw` must be open and suitable for assuming
    /// ownership.
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        debug_assert_ne!(fd, -1_i32 as RawFd);
        Self { fd }
    }
}

#[cfg(windows)]
impl FromRawHandle for OwnedHandle {
    #[inline]
    unsafe fn from_raw_handle(handle: RawHandle) -> Self {
        Self { handle }
    }
}

#[cfg(windows)]
impl FromRawSocket for OwnedSocket {
    #[inline]
    unsafe fn from_raw_socket(socket: RawSocket) -> Self {
        debug_assert_ne!(socket, INVALID_SOCKET as RawSocket);
        Self { socket }
    }
}

#[cfg(windows)]
impl HandleOrInvalid {
    /// Constructs a new instance of `Self` from the given `RawHandle` returned
    /// from a Windows API that uses `INVALID_HANDLE_VALUE` to indicate
    /// failure, such as `CreateFileW`.
    ///
    /// Use `HandleOrNull` instead of `HandleOrInvalid` for APIs that
    /// use null to indicate failure.
    ///
    /// # Safety
    ///
    /// The passed `handle` value must either satisfy the safety requirements
    /// of [`FromRawHandle::from_raw_handle`], or be
    /// `INVALID_HANDLE_VALUE` (-1). Note that not all Windows APIs use
    /// `INVALID_HANDLE_VALUE` for errors; see [here] for the full story.
    ///
    /// [here]: https://devblogs.microsoft.com/oldnewthing/20040302-00/?p=40443
    #[inline]
    pub unsafe fn from_raw_handle(handle: RawHandle) -> Self {
        Self(handle)
    }
}

#[cfg(windows)]
impl HandleOrNull {
    /// Constructs a new instance of `Self` from the given `RawHandle` returned
    /// from a Windows API that uses null to indicate failure, such as
    /// `CreateThread`.
    ///
    /// Use `HandleOrInvalid` instead of `HandleOrNull` for APIs that
    /// use `INVALID_HANDLE_VALUE` to indicate failure.
    ///
    /// # Safety
    ///
    /// The passed `handle` value must either satisfy the safety requirements
    /// of [`FromRawHandle::from_raw_handle`], or be null. Note that not all
    /// Windows APIs use null for errors; see [here] for the full story.
    ///
    /// [here]: https://devblogs.microsoft.com/oldnewthing/20040302-00/?p=40443
    #[inline]
    pub unsafe fn from_raw_handle(handle: RawHandle) -> Self {
        Self(handle)
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl Drop for OwnedFd {
    #[inline]
    fn drop(&mut self) {
        #[cfg(feature = "close")]
        unsafe {
            let _ = libc::close(self.fd as std::os::raw::c_int);
        }

        // If the `close` feature is disabled, we expect users to avoid letting
        // `OwnedFd` instances drop, so that we don't have to call `close`.
        #[cfg(not(feature = "close"))]
        {
            unreachable!("drop called without the \"close\" feature in io-lifetimes");
        }
    }
}

#[cfg(target_os = "hermit")]
impl Drop for OwnedFd {
    #[inline]
    fn drop(&mut self) {
        #[cfg(feature = "close")]
        unsafe {
            let _ = hermit_abi::close(self.fd);
        }

        // If the `close` feature is disabled, we expect users to avoid letting
        // `OwnedFd` instances drop, so that we don't have to call `close`.
        #[cfg(not(feature = "close"))]
        {
            unreachable!("drop called without the \"close\" feature in io-lifetimes");
        }
    }
}

#[cfg(windows)]
impl Drop for OwnedHandle {
    #[inline]
    fn drop(&mut self) {
        #[cfg(feature = "close")]
        unsafe {
            let _ = CloseHandle(self.handle as HANDLE);
        }

        // If the `close` feature is disabled, we expect users to avoid letting
        // `OwnedHandle` instances drop, so that we don't have to call `close`.
        #[cfg(not(feature = "close"))]
        {
            unreachable!("drop called without the \"close\" feature in io-lifetimes");
        }
    }
}

#[cfg(windows)]
impl Drop for HandleOrInvalid {
    #[inline]
    fn drop(&mut self) {
        #[cfg(feature = "close")]
        unsafe {
            let _ = CloseHandle(self.0 as HANDLE);
        }

        // If the `close` feature is disabled, we expect users to avoid letting
        // `HandleOrInvalid` instances drop, so that we don't have to call `close`.
        #[cfg(not(feature = "close"))]
        {
            unreachable!("drop called without the \"close\" feature in io-lifetimes");
        }
    }
}

#[cfg(windows)]
impl Drop for HandleOrNull {
    #[inline]
    fn drop(&mut self) {
        #[cfg(feature = "close")]
        unsafe {
            let _ = CloseHandle(self.0 as HANDLE);
        }

        // If the `close` feature is disabled, we expect users to avoid letting
        // `HandleOrNull` instances drop, so that we don't have to call `close`.
        #[cfg(not(feature = "close"))]
        {
            unreachable!("drop called without the \"close\" feature in io-lifetimes");
        }
    }
}

#[cfg(windows)]
impl Drop for OwnedSocket {
    #[inline]
    fn drop(&mut self) {
        #[cfg(feature = "close")]
        unsafe {
            let _ = closesocket(self.socket as SOCKET);
        }

        // If the `close` feature is disabled, we expect users to avoid letting
        // `OwnedSocket` instances drop, so that we don't have to call `close`.
        #[cfg(not(feature = "close"))]
        {
            unreachable!("drop called without the \"close\" feature in io-lifetimes");
        }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl fmt::Debug for BorrowedFd<'_> {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowedFd").field("fd", &self.fd).finish()
    }
}

#[cfg(windows)]
impl fmt::Debug for BorrowedHandle<'_> {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowedHandle")
            .field("handle", &self.handle)
            .finish()
    }
}

#[cfg(windows)]
impl fmt::Debug for BorrowedSocket<'_> {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowedSocket")
            .field("socket", &self.socket)
            .finish()
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl fmt::Debug for OwnedFd {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedFd").field("fd", &self.fd).finish()
    }
}

#[cfg(windows)]
impl fmt::Debug for OwnedHandle {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedHandle")
            .field("handle", &self.handle)
            .finish()
    }
}

#[cfg(windows)]
impl fmt::Debug for OwnedSocket {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedSocket")
            .field("socket", &self.socket)
            .finish()
    }
}
