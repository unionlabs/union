//! Windows system calls in the `event` module.

use crate::backend::c;
use crate::backend::conv::ret_c_int;
use crate::event::PollFd;
use crate::io;

pub(crate) fn poll(fds: &mut [PollFd<'_>], timeout: c::c_int) -> io::Result<usize> {
    let nfds = fds
        .len()
        .try_into()
        .map_err(|_convert_err| io::Errno::INVAL)?;

    ret_c_int(unsafe { c::poll(fds.as_mut_ptr().cast(), nfds, timeout) })
        .map(|nready| nready as usize)
}
