//! This module roughly corresponds to `mach/exc.h`.

use exception_types::{exception_data_t, exception_type_t};
use kern_return::kern_return_t;
use message::mach_msg_type_number_t;
use port::mach_port_t;
use thread_status::thread_state_t;

pub const exc_MSG_COUNT: ::libc::c_uint = 3;

extern "C" {
    pub fn exception_raise(
        exception_port: mach_port_t,
        thread: mach_port_t,
        task: mach_port_t,
        exception: exception_type_t,
        code: exception_data_t,
        codeCnt: mach_msg_type_number_t,
    ) -> kern_return_t;
    pub fn exception_raise_state(
        exception_port: mach_port_t,
        exception: exception_type_t,
        code: exception_data_t,
        codeCnt: mach_msg_type_number_t,
        flavor: *mut ::libc::c_int,
        old_state: thread_state_t,
        old_stateCnt: mach_msg_type_number_t,
        new_state: thread_state_t,
        new_stateCnt: *mut mach_msg_type_number_t,
    ) -> kern_return_t;
    pub fn exception_raise_state_identity(
        exception_port: mach_port_t,
        thread: mach_port_t,
        task: mach_port_t,
        exception: exception_type_t,
        code: exception_data_t,
        codeCnt: mach_msg_type_number_t,
        flavor: *mut ::libc::c_int,
        old_state: thread_state_t,
        old_stateCnt: mach_msg_type_number_t,
        new_state: thread_state_t,
        new_stateCnt: *mut mach_msg_type_number_t,
    ) -> kern_return_t;
}
