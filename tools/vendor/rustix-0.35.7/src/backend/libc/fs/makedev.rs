#[cfg(not(all(target_os = "android", target_pointer_width = "32")))]
use super::super::c;
use crate::fs::Dev;

#[cfg(not(any(target_os = "android", target_os = "emscripten")))]
#[inline]
pub(crate) fn makedev(maj: u32, min: u32) -> Dev {
    unsafe { c::makedev(maj, min) }
}

#[cfg(all(target_os = "android", not(target_pointer_width = "32")))]
#[inline]
pub(crate) fn makedev(maj: u32, min: u32) -> Dev {
    // Android's `makedev` oddly has signed argument types.
    unsafe { c::makedev(maj as i32, min as i32) }
}

#[cfg(all(target_os = "android", target_pointer_width = "32"))]
#[inline]
pub(crate) fn makedev(maj: u32, min: u32) -> Dev {
    // 32-bit Android's `dev_t` is 32-bit, but its `st_dev` is 64-bit,
    // so we do it ourselves.
    ((u64::from(maj) & 0xffff_f000_u64) << 32)
        | ((u64::from(maj) & 0x0000_0fff_u64) << 8)
        | ((u64::from(min) & 0xffff_ff00_u64) << 12)
        | (u64::from(min) & 0x0000_00ff_u64)
}

#[cfg(target_os = "emscripten")]
#[inline]
pub(crate) fn makedev(maj: u32, min: u32) -> Dev {
    // Emscripten's `makedev` has a 32-bit return value.
    Dev::from(unsafe { c::makedev(maj, min) })
}

#[cfg(not(any(target_os = "android", target_os = "emscripten")))]
#[inline]
pub(crate) fn major(dev: Dev) -> u32 {
    unsafe { c::major(dev) }
}

#[cfg(all(target_os = "android", not(target_pointer_width = "32")))]
#[inline]
pub(crate) fn major(dev: Dev) -> u32 {
    // Android's `major` oddly has signed return types.
    (unsafe { c::major(dev) }) as u32
}

#[cfg(all(target_os = "android", target_pointer_width = "32"))]
#[inline]
pub(crate) fn major(dev: Dev) -> u32 {
    // 32-bit Android's `dev_t` is 32-bit, but its `st_dev` is 64-bit,
    // so we do it ourselves.
    (((dev >> 31 >> 1) & 0xffff_f000) | ((dev >> 8) & 0x0000_0fff)) as u32
}

#[cfg(target_os = "emscripten")]
#[inline]
pub(crate) fn major(dev: Dev) -> u32 {
    // Emscripten's `major` has a 32-bit argument value.
    unsafe { c::major(dev as u32) }
}

#[cfg(not(any(target_os = "android", target_os = "emscripten")))]
#[inline]
pub(crate) fn minor(dev: Dev) -> u32 {
    unsafe { c::minor(dev) }
}

#[cfg(all(target_os = "android", not(target_pointer_width = "32")))]
#[inline]
pub(crate) fn minor(dev: Dev) -> u32 {
    // Android's `minor` oddly has signed return types.
    (unsafe { c::minor(dev) }) as u32
}

#[cfg(all(target_os = "android", target_pointer_width = "32"))]
#[inline]
pub(crate) fn minor(dev: Dev) -> u32 {
    // 32-bit Android's `dev_t` is 32-bit, but its `st_dev` is 64-bit,
    // so we do it ourselves.
    (((dev >> 12) & 0xffff_ff00) | (dev & 0x0000_00ff)) as u32
}

#[cfg(target_os = "emscripten")]
#[inline]
pub(crate) fn minor(dev: Dev) -> u32 {
    // Emscripten's `minor` has a 32-bit argument value.
    unsafe { c::minor(dev as u32) }
}
