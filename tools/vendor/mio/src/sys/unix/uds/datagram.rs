use super::{socket_addr, SocketAddr};
use crate::sys::unix::net::new_socket;

use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net;
use std::path::Path;

pub(crate) fn bind(path: &Path) -> io::Result<net::UnixDatagram> {
    let fd = new_socket(libc::AF_UNIX, libc::SOCK_DGRAM)?;
    // Ensure the fd is closed.
    let socket = unsafe { net::UnixDatagram::from_raw_fd(fd) };
    let (sockaddr, socklen) = socket_addr(path)?;
    let sockaddr = &sockaddr as *const libc::sockaddr_un as *const _;
    syscall!(bind(fd, sockaddr, socklen))?;
    Ok(socket)
}

pub(crate) fn unbound() -> io::Result<net::UnixDatagram> {
    new_socket(libc::AF_UNIX, libc::SOCK_DGRAM)
        .map(|socket| unsafe { net::UnixDatagram::from_raw_fd(socket) })
}

pub(crate) fn pair() -> io::Result<(net::UnixDatagram, net::UnixDatagram)> {
    super::pair(libc::SOCK_DGRAM)
}

pub(crate) fn local_addr(socket: &net::UnixDatagram) -> io::Result<SocketAddr> {
    super::local_addr(socket.as_raw_fd())
}

pub(crate) fn peer_addr(socket: &net::UnixDatagram) -> io::Result<SocketAddr> {
    super::peer_addr(socket.as_raw_fd())
}

pub(crate) fn recv_from(
    socket: &net::UnixDatagram,
    dst: &mut [u8],
) -> io::Result<(usize, SocketAddr)> {
    let mut count = 0;
    let socketaddr = SocketAddr::new(|sockaddr, socklen| {
        syscall!(recvfrom(
            socket.as_raw_fd(),
            dst.as_mut_ptr() as *mut _,
            dst.len(),
            0,
            sockaddr,
            socklen,
        ))
        .map(|c| {
            count = c;
            c as libc::c_int
        })
    })?;
    Ok((count as usize, socketaddr))
}
