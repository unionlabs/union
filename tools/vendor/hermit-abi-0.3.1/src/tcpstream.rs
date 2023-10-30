//! `tcpstream` provide an interface to establish tcp socket client.

use crate::{Handle, IpAddress};

extern "Rust" {
	fn sys_tcp_stream_connect(ip: &[u8], port: u16, timeout: Option<u64>) -> Result<Handle, ()>;
	fn sys_tcp_stream_close(handle: Handle) -> Result<(), ()>;
	fn sys_tcp_stream_read(handle: Handle, buffer: &mut [u8]) -> Result<usize, ()>;
	fn sys_tcp_stream_write(handle: Handle, buffer: &[u8]) -> Result<usize, ()>;
	fn sys_tcp_stream_set_read_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()>;
	fn sys_tcp_stream_get_read_timeout(handle: Handle) -> Result<Option<u64>, ()>;
	fn sys_tcp_stream_set_write_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()>;
	fn sys_tcp_stream_get_write_timeout(handle: Handle) -> Result<Option<u64>, ()>;
	fn sys_tcp_stream_peek(handle: Handle, buf: &mut [u8]) -> Result<usize, ()>;
	fn sys_tcp_stream_set_nonblocking(handle: Handle, mode: bool) -> Result<(), ()>;
	fn sys_tcp_stream_set_tll(handle: Handle, ttl: u32) -> Result<(), ()>;
	fn sys_tcp_stream_get_tll(handle: Handle) -> Result<u32, ()>;
	fn sys_tcp_stream_shutdown(handle: Handle, how: i32) -> Result<(), ()>;
	fn sys_tcp_stream_peer_addr(handle: Handle) -> Result<(IpAddress, u16), ()>;
}

/// Opens a TCP connection to a remote host.
#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn connect(ip: &[u8], port: u16, timeout: Option<u64>) -> Result<Handle, ()> {
	unsafe { sys_tcp_stream_connect(ip, port, timeout) }
}

/// Close a TCP connection
#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn close(handle: Handle) -> Result<(), ()> {
	unsafe { sys_tcp_stream_close(handle) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn peek(handle: Handle, buf: &mut [u8]) -> Result<usize, ()> {
	unsafe { sys_tcp_stream_peek(handle, buf) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn peer_addr(handle: Handle) -> Result<(IpAddress, u16), ()> {
	unsafe { sys_tcp_stream_peer_addr(handle) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn read(handle: Handle, buffer: &mut [u8]) -> Result<usize, ()> {
	unsafe { sys_tcp_stream_read(handle, buffer) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn write(handle: Handle, buffer: &[u8]) -> Result<usize, ()> {
	unsafe { sys_tcp_stream_write(handle, buffer) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn set_read_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
	unsafe { sys_tcp_stream_set_read_timeout(handle, timeout) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn set_write_timeout(handle: Handle, timeout: Option<u64>) -> Result<(), ()> {
	unsafe { sys_tcp_stream_set_write_timeout(handle, timeout) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn get_read_timeout(handle: Handle) -> Result<Option<u64>, ()> {
	unsafe { sys_tcp_stream_get_read_timeout(handle) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn get_write_timeout(handle: Handle) -> Result<Option<u64>, ()> {
	unsafe { sys_tcp_stream_get_write_timeout(handle) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn set_nodelay(_: Handle, mode: bool) -> Result<(), ()> {
	// smoltcp does not support Nagle's algorithm
	// => to enable Nagle's algorithm isn't possible
	if mode {
		Ok(())
	} else {
		Err(())
	}
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn nodelay(_: Handle) -> Result<bool, ()> {
	// smoltcp does not support Nagle's algorithm
	// => return always true
	Ok(true)
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn set_nonblocking(handle: Handle, mode: bool) -> Result<(), ()> {
	unsafe { sys_tcp_stream_set_nonblocking(handle, mode) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn set_tll(handle: Handle, ttl: u32) -> Result<(), ()> {
	unsafe { sys_tcp_stream_set_tll(handle, ttl) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn get_tll(handle: Handle) -> Result<u32, ()> {
	unsafe { sys_tcp_stream_get_tll(handle) }
}

#[deprecated(since = "0.3.0", note = "please use new BSD socket interface")]
pub fn shutdown(handle: Handle, how: i32) -> Result<(), ()> {
	unsafe { sys_tcp_stream_shutdown(handle, how) }
}
