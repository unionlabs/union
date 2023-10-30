#![allow(deprecated)] // Don't warn on `IntoFd` and `FromFd` impls.

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
use crate::{AsFd, FromFd, IntoFd};
#[cfg(windows)]
use crate::{AsHandle, AsSocket, FromHandle, FromSocket, IntoHandle, IntoSocket};
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
use crate::{BorrowedFd, OwnedFd};
#[cfg(windows)]
use crate::{BorrowedHandle, BorrowedSocket, HandleOrInvalid, OwnedHandle, OwnedSocket};
#[cfg(target_os = "hermit")]
use std::os::hermit::io::{AsRawFd, FromRawFd, IntoRawFd};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, FromRawFd, IntoRawFd};
#[cfg(windows)]
use std::os::windows::io::{
    AsRawHandle, AsRawSocket, FromRawHandle, FromRawSocket, IntoRawHandle, IntoRawSocket,
};

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsFd for BorrowedFd<'_> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for BorrowedHandle<'_> {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(windows)]
impl AsSocket for BorrowedSocket<'_> {
    #[inline]
    fn as_socket(&self) -> BorrowedSocket<'_> {
        unsafe { BorrowedSocket::borrow_raw(self.as_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsFd for OwnedFd {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for OwnedHandle {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(windows)]
impl AsSocket for OwnedSocket {
    #[inline]
    fn as_socket(&self) -> BorrowedSocket<'_> {
        unsafe { BorrowedSocket::borrow_raw(self.as_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl IntoFd for OwnedFd {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { Self::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoHandle for OwnedHandle {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { Self::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl IntoSocket for OwnedSocket {
    #[inline]
    fn into_socket(self) -> OwnedSocket {
        unsafe { Self::from_raw_socket(self.into_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl FromFd for OwnedFd {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl FromHandle for OwnedHandle {
    #[inline]
    fn from_handle(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl FromSocket for OwnedSocket {
    #[inline]
    fn from_socket(owned: OwnedSocket) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(windows)]
impl FromHandle for HandleOrInvalid {
    #[inline]
    fn from_handle(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<OwnedHandle> for HandleOrInvalid {
    #[inline]
    fn from(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsFd for std::fs::File {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for std::fs::File {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl IntoFd for std::fs::File {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl From<std::fs::File> for OwnedFd {
    #[inline]
    fn from(owned: std::fs::File) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoHandle for std::fs::File {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { OwnedHandle::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<std::fs::File> for OwnedHandle {
    #[inline]
    fn from(owned: std::fs::File) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl FromFd for std::fs::File {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl From<OwnedFd> for std::fs::File {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl FromHandle for std::fs::File {
    #[inline]
    fn from_handle(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<OwnedHandle> for std::fs::File {
    #[inline]
    fn from(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsFd for std::net::TcpStream {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsSocket for std::net::TcpStream {
    #[inline]
    fn as_socket(&self) -> BorrowedSocket<'_> {
        unsafe { BorrowedSocket::borrow_raw(self.as_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl IntoFd for std::net::TcpStream {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl From<std::net::TcpStream> for OwnedFd {
    #[inline]
    fn from(owned: std::net::TcpStream) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoSocket for std::net::TcpStream {
    #[inline]
    fn into_socket(self) -> OwnedSocket {
        unsafe { OwnedSocket::from_raw_socket(self.into_raw_socket()) }
    }
}

#[cfg(windows)]
impl From<std::net::TcpStream> for OwnedSocket {
    #[inline]
    fn from(owned: std::net::TcpStream) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl FromFd for std::net::TcpStream {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl From<OwnedFd> for std::net::TcpStream {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl FromSocket for std::net::TcpStream {
    #[inline]
    fn from_socket(owned: OwnedSocket) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(windows)]
impl From<OwnedSocket> for std::net::TcpStream {
    #[inline]
    fn from(owned: OwnedSocket) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsFd for std::net::TcpListener {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsSocket for std::net::TcpListener {
    #[inline]
    fn as_socket(&self) -> BorrowedSocket<'_> {
        unsafe { BorrowedSocket::borrow_raw(self.as_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl IntoFd for std::net::TcpListener {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl From<std::net::TcpListener> for OwnedFd {
    #[inline]
    fn from(owned: std::net::TcpListener) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoSocket for std::net::TcpListener {
    #[inline]
    fn into_socket(self) -> OwnedSocket {
        unsafe { OwnedSocket::from_raw_socket(self.into_raw_socket()) }
    }
}

#[cfg(windows)]
impl From<std::net::TcpListener> for OwnedSocket {
    #[inline]
    fn from(owned: std::net::TcpListener) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl FromFd for std::net::TcpListener {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl From<OwnedFd> for std::net::TcpListener {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl FromSocket for std::net::TcpListener {
    #[inline]
    fn from_socket(owned: OwnedSocket) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(windows)]
impl From<OwnedSocket> for std::net::TcpListener {
    #[inline]
    fn from(owned: OwnedSocket) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsFd for std::net::UdpSocket {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsSocket for std::net::UdpSocket {
    #[inline]
    fn as_socket(&self) -> BorrowedSocket {
        unsafe { BorrowedSocket::borrow_raw(self.as_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl IntoFd for std::net::UdpSocket {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl From<std::net::UdpSocket> for OwnedFd {
    #[inline]
    fn from(owned: std::net::UdpSocket) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoSocket for std::net::UdpSocket {
    #[inline]
    fn into_socket(self) -> OwnedSocket {
        unsafe { OwnedSocket::from_raw_socket(self.into_raw_socket()) }
    }
}

#[cfg(windows)]
impl From<std::net::UdpSocket> for OwnedSocket {
    #[inline]
    fn from(owned: std::net::UdpSocket) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl FromFd for std::net::UdpSocket {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl From<OwnedFd> for std::net::UdpSocket {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl FromSocket for std::net::UdpSocket {
    #[inline]
    fn from_socket(owned: OwnedSocket) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(windows)]
impl From<OwnedSocket> for std::net::UdpSocket {
    #[inline]
    fn from(owned: OwnedSocket) -> Self {
        unsafe { Self::from_raw_socket(owned.into_raw_socket()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsFd for std::io::Stdin {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for std::io::Stdin {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl<'a> AsFd for std::io::StdinLock<'a> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl<'a> AsHandle for std::io::StdinLock<'a> {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsFd for std::io::Stdout {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for std::io::Stdout {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl<'a> AsFd for std::io::StdoutLock<'a> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl<'a> AsHandle for std::io::StdoutLock<'a> {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl AsFd for std::io::Stderr {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for std::io::Stderr {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl<'a> AsFd for std::io::StderrLock<'a> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl<'a> AsHandle for std::io::StderrLock<'a> {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(unix)]
impl AsFd for std::process::ChildStdin {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for std::process::ChildStdin {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(unix)]
impl IntoFd for std::process::ChildStdin {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<std::process::ChildStdin> for OwnedFd {
    #[inline]
    fn from(owned: std::process::ChildStdin) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoHandle for std::process::ChildStdin {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { OwnedHandle::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<std::process::ChildStdin> for OwnedHandle {
    #[inline]
    fn from(owned: std::process::ChildStdin) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(unix)]
impl AsFd for std::process::ChildStdout {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for std::process::ChildStdout {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(unix)]
impl IntoFd for std::process::ChildStdout {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<std::process::ChildStdout> for OwnedFd {
    #[inline]
    fn from(owned: std::process::ChildStdout) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoHandle for std::process::ChildStdout {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { OwnedHandle::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<std::process::ChildStdout> for OwnedHandle {
    #[inline]
    fn from(owned: std::process::ChildStdout) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(unix)]
impl AsFd for std::process::ChildStderr {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for std::process::ChildStderr {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(unix)]
impl IntoFd for std::process::ChildStderr {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<std::process::ChildStderr> for OwnedFd {
    #[inline]
    fn from(owned: std::process::ChildStderr) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoHandle for std::process::ChildStderr {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { OwnedHandle::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<std::process::ChildStderr> for OwnedHandle {
    #[inline]
    fn from(owned: std::process::ChildStderr) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(unix)]
impl FromFd for std::process::Stdio {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<OwnedFd> for std::process::Stdio {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl FromHandle for std::process::Stdio {
    #[inline]
    fn from_handle(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<OwnedHandle> for std::process::Stdio {
    #[inline]
    fn from(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl AsHandle for std::process::Child {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(windows)]
impl IntoHandle for std::process::Child {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { OwnedHandle::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<std::process::Child> for OwnedHandle {
    #[inline]
    fn from(owned: std::process::Child) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(unix)]
impl AsFd for std::os::unix::net::UnixStream {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(unix)]
impl IntoFd for std::os::unix::net::UnixStream {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<std::os::unix::net::UnixStream> for OwnedFd {
    #[inline]
    fn from(owned: std::os::unix::net::UnixStream) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl FromFd for std::os::unix::net::UnixStream {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<OwnedFd> for std::os::unix::net::UnixStream {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl AsFd for std::os::unix::net::UnixListener {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(unix)]
impl IntoFd for std::os::unix::net::UnixListener {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<std::os::unix::net::UnixListener> for OwnedFd {
    #[inline]
    fn from(owned: std::os::unix::net::UnixListener) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl FromFd for std::os::unix::net::UnixListener {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<OwnedFd> for std::os::unix::net::UnixListener {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl AsFd for std::os::unix::net::UnixDatagram {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(unix)]
impl IntoFd for std::os::unix::net::UnixDatagram {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<std::os::unix::net::UnixDatagram> for OwnedFd {
    #[inline]
    fn from(owned: std::os::unix::net::UnixDatagram) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl FromFd for std::os::unix::net::UnixDatagram {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(unix)]
impl From<OwnedFd> for std::os::unix::net::UnixDatagram {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl<T> AsHandle for std::thread::JoinHandle<T> {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(windows)]
impl<T> IntoHandle for std::thread::JoinHandle<T> {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { OwnedHandle::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl<T> From<std::thread::JoinHandle<T>> for OwnedHandle {
    #[inline]
    fn from(owned: std::thread::JoinHandle<T>) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}
