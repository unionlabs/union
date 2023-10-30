//! Tests for [`rustix::net`].

#![cfg(feature = "net")]
#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg(not(any(target_os = "redox", target_os = "wasi")))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
#![cfg_attr(core_c_str, feature(core_c_str))]

mod addr;
mod connect_bind_send;
mod poll;
mod sockopt;
#[cfg(unix)]
mod unix;
mod v4;
mod v6;

/// Windows requires us to call a setup function before using any of the
/// socket APIs.
#[cfg(windows)]
#[ctor::ctor]
fn windows_startup() {
    let _ = rustix::net::wsa_startup().unwrap();
}

/// Windows requires us to call a cleanup function after using any of the
/// socket APIs.
#[cfg(windows)]
#[ctor::dtor]
fn windows_shutdown() {
    rustix::net::wsa_cleanup().unwrap();
}
