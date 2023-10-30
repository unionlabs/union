//! Rust implementation of the `CPU_*` macro API.

#![allow(non_snake_case)]

use super::types::{RawCpuSet, CPU_SETSIZE};
use crate::backend::c;

#[inline]
pub(crate) fn CPU_SET(cpu: usize, cpuset: &mut RawCpuSet) {
    assert!(
        cpu < CPU_SETSIZE,
        "cpu out of bounds: the cpu max is {} but the cpu is {}",
        CPU_SETSIZE,
        cpu
    );
    unsafe { c::CPU_SET(cpu, cpuset) }
}

#[inline]
pub(crate) fn CPU_ZERO(cpuset: &mut RawCpuSet) {
    unsafe { c::CPU_ZERO(cpuset) }
}

#[inline]
pub(crate) fn CPU_CLR(cpu: usize, cpuset: &mut RawCpuSet) {
    assert!(
        cpu < CPU_SETSIZE,
        "cpu out of bounds: the cpu max is {} but the cpu is {}",
        CPU_SETSIZE,
        cpu
    );
    unsafe { c::CPU_CLR(cpu, cpuset) }
}

#[inline]
pub(crate) fn CPU_ISSET(cpu: usize, cpuset: &RawCpuSet) -> bool {
    assert!(
        cpu < CPU_SETSIZE,
        "cpu out of bounds: the cpu max is {} but the cpu is {}",
        CPU_SETSIZE,
        cpu
    );
    unsafe { c::CPU_ISSET(cpu, cpuset) }
}

#[cfg(linux_kernel)]
#[inline]
pub(crate) fn CPU_COUNT(cpuset: &RawCpuSet) -> u32 {
    unsafe { c::CPU_COUNT(cpuset).try_into().unwrap() }
}
