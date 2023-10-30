use std::io;
use std::mem::{self, MaybeUninit};
use std::net::{self, SocketAddr};
use std::os::windows::io::{AsRawSocket, FromRawSocket};
use std::os::windows::raw::SOCKET as StdSocket; // windows-sys uses usize, stdlib uses u32/u64.

use crate::sys::windows::net::{init, new_ip_socket, socket_addr};
use windows_sys::Win32::Networking::WinSock::{
    bind as win_bind, closesocket, getsockopt, IPPROTO_IPV6, IPV6_V6ONLY, SOCKET_ERROR, SOCK_DGRAM,
};

pub fn bind(addr: SocketAddr) -> io::Result<net::UdpSocket> {
    init();
    new_ip_socket(addr, SOCK_DGRAM).and_then(|socket| {
        let (raw_addr, raw_addr_length) = socket_addr(&addr);
        syscall!(
            win_bind(socket, raw_addr.as_ptr(), raw_addr_length,),
            PartialEq::eq,
            SOCKET_ERROR
        )
        .map_err(|err| {
            // Close the socket if we hit an error, ignoring the error
            // from closing since we can't pass back two errors.
            let _ = unsafe { closesocket(socket) };
            err
        })
        .map(|_| unsafe { net::UdpSocket::from_raw_socket(socket as StdSocket) })
    })
}

pub(crate) fn only_v6(socket: &net::UdpSocket) -> io::Result<bool> {
    let mut optval: MaybeUninit<i32> = MaybeUninit::uninit();
    let mut optlen = mem::size_of::<i32>() as i32;

    syscall!(
        getsockopt(
            socket.as_raw_socket() as usize,
            IPPROTO_IPV6 as i32,
            IPV6_V6ONLY as i32,
            optval.as_mut_ptr().cast(),
            &mut optlen,
        ),
        PartialEq::eq,
        SOCKET_ERROR
    )?;

    debug_assert_eq!(optlen as usize, mem::size_of::<i32>());
    // Safety: `getsockopt` initialised `optval` for us.
    let optval = unsafe { optval.assume_init() };
    Ok(optval != 0)
}
