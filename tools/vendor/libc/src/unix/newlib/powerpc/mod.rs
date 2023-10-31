pub type clock_t = ::c_ulong;
pub type c_char = u8;
pub type wchar_t = ::c_int;

pub type c_long = i32;
pub type c_ulong = u32;

pub use crate::unix::newlib::generic::{sigset_t, stat};

// the newlib shipped with devkitPPC does not support the following components:
// - sockaddr
// - AF_INET6
// - FIONBIO
// - POLL*
// - SOL_SOCKET
// - MSG_*
