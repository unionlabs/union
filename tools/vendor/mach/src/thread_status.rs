//! This module corresponds to `mach/thread_status.h`.

use vm_types::natural_t;

pub type thread_state_t = *mut natural_t;
pub type thread_state_flavor_t = ::libc::c_int;

pub static x86_THREAD_STATE32: thread_state_flavor_t = 1;
pub static x86_FLOAT_STATE32: thread_state_flavor_t = 2;
pub static x86_EXCEPTION_STATE32: thread_state_flavor_t = 3;
pub static x86_THREAD_STATE64: thread_state_flavor_t = 4;
pub static x86_FLOAT_STATE64: thread_state_flavor_t = 5;
pub static x86_EXCEPTION_STATE64: thread_state_flavor_t = 6;
pub static x86_THREAD_STATE: thread_state_flavor_t = 7;
pub static x86_FLOAT_STATE: thread_state_flavor_t = 8;
pub static x86_EXCEPTION_STATE: thread_state_flavor_t = 9;
pub static x86_DEBUG_STATE32: thread_state_flavor_t = 10;
pub static x86_DEBUG_STATE64: thread_state_flavor_t = 11;
pub static x86_DEBUG_STATE: thread_state_flavor_t = 12;
pub static THREAD_STATE_NONE: thread_state_flavor_t = 13;
pub static x86_AVX_STATE32: thread_state_flavor_t = 16;
pub static x86_AVX_STATE64: thread_state_flavor_t = 17;
pub static x86_AVX_STATE: thread_state_flavor_t = 18;
