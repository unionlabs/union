use crate::views::{FilelikeViewType, SocketlikeViewType};
#[cfg(any(unix, target_os = "wasi"))]
use crate::OwnedFd;
#[cfg(windows)]
use crate::{OwnedHandle, OwnedSocket};

#[cfg(any(unix, target_os = "wasi"))]
unsafe impl FilelikeViewType for OwnedFd {}

#[cfg(windows)]
unsafe impl FilelikeViewType for OwnedHandle {}

#[cfg(windows)]
unsafe impl SocketlikeViewType for OwnedSocket {}

unsafe impl FilelikeViewType for std::fs::File {}

unsafe impl SocketlikeViewType for std::net::TcpStream {}

unsafe impl SocketlikeViewType for std::net::TcpListener {}

unsafe impl SocketlikeViewType for std::net::UdpSocket {}

#[cfg(unix)]
unsafe impl SocketlikeViewType for std::os::unix::net::UnixStream {}

#[cfg(unix)]
unsafe impl SocketlikeViewType for std::os::unix::net::UnixListener {}

#[cfg(unix)]
unsafe impl SocketlikeViewType for std::os::unix::net::UnixDatagram {}
