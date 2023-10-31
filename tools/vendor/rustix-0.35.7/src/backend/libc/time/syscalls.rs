//! libc syscalls supporting `rustix::time`.

use super::super::c;
use super::super::conv::ret;
#[cfg(feature = "time")]
#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
use super::super::time::types::LibcItimerspec;
use super::super::time::types::LibcTimespec;
use super::types::Timespec;
#[cfg(not(target_os = "wasi"))]
use super::types::{ClockId, DynamicClockId};
use crate::io;
use core::mem::MaybeUninit;
#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
#[cfg(feature = "time")]
use {
    super::super::conv::{borrowed_fd, ret_owned_fd},
    crate::fd::BorrowedFd,
    crate::io::OwnedFd,
    crate::time::{Itimerspec, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags},
};

#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
weak!(fn __clock_gettime64(c::clockid_t, *mut LibcTimespec) -> c::c_int);
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
weak!(fn __clock_getres64(c::clockid_t, *mut LibcTimespec) -> c::c_int);
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[cfg(feature = "time")]
weak!(fn __timerfd_gettime64(c::c_int, *mut LibcItimerspec) -> c::c_int);
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[cfg(feature = "time")]
weak!(fn __timerfd_settime64(c::c_int, c::c_int, *const LibcItimerspec, *mut LibcItimerspec) -> c::c_int);

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[inline]
#[must_use]
pub(crate) fn clock_getres(id: ClockId) -> Timespec {
    let mut timespec = MaybeUninit::<LibcTimespec>::uninit();

    // 32-bit gnu version: libc has `clock_getres` but it is not y2038 safe by
    // default.
    #[cfg(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    ))]
    unsafe {
        if let Some(libc_clock_getres) = __clock_getres64.get() {
            ret(libc_clock_getres(id as c::clockid_t, timespec.as_mut_ptr())).unwrap();
            timespec.assume_init().into()
        } else {
            clock_getres_old(id)
        }
    }

    // Main version: libc is y2038 safe and has `clock_getres`.
    #[cfg(not(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    )))]
    unsafe {
        let _ = c::clock_getres(id as c::clockid_t, timespec.as_mut_ptr());
        timespec.assume_init()
    }
}

#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[must_use]
unsafe fn clock_getres_old(id: ClockId) -> Timespec {
    let mut old_timespec = MaybeUninit::<c::timespec>::uninit();
    ret(c::clock_getres(
        id as c::clockid_t,
        old_timespec.as_mut_ptr(),
    ))
    .unwrap();
    let old_timespec = old_timespec.assume_init();
    Timespec {
        tv_sec: old_timespec.tv_sec.into(),
        tv_nsec: old_timespec.tv_nsec.into(),
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn clock_gettime(id: ClockId) -> Timespec {
    let mut timespec = MaybeUninit::<LibcTimespec>::uninit();

    #[cfg(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    ))]
    unsafe {
        if let Some(libc_clock_gettime) = __clock_gettime64.get() {
            ret(libc_clock_gettime(
                id as c::clockid_t,
                timespec.as_mut_ptr(),
            ))
            .unwrap();
            timespec.assume_init().into()
        } else {
            clock_gettime_old(id)
        }
    }

    // Use `unwrap()` here because `clock_getres` can fail if the clock itself
    // overflows a number of seconds, but if that happens, the monotonic clocks
    // can't maintain their invariants, or the realtime clocks aren't properly
    // configured.
    #[cfg(not(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    )))]
    unsafe {
        ret(c::clock_gettime(id as c::clockid_t, timespec.as_mut_ptr())).unwrap();
        timespec.assume_init()
    }
}

#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[must_use]
unsafe fn clock_gettime_old(id: ClockId) -> Timespec {
    let mut old_timespec = MaybeUninit::<c::timespec>::uninit();
    ret(c::clock_gettime(
        id as c::clockid_t,
        old_timespec.as_mut_ptr(),
    ))
    .unwrap();
    let old_timespec = old_timespec.assume_init();
    Timespec {
        tv_sec: old_timespec.tv_sec.into(),
        tv_nsec: old_timespec.tv_nsec.into(),
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn clock_gettime_dynamic(id: DynamicClockId<'_>) -> io::Result<Timespec> {
    let mut timespec = MaybeUninit::<LibcTimespec>::uninit();
    unsafe {
        let id: c::clockid_t = match id {
            DynamicClockId::Known(id) => id as c::clockid_t,

            #[cfg(any(target_os = "android", target_os = "linux"))]
            DynamicClockId::Dynamic(fd) => {
                use crate::fd::AsRawFd;
                const CLOCKFD: i32 = 3;
                (!fd.as_raw_fd() << 3) | CLOCKFD
            }

            #[cfg(not(any(target_os = "android", target_os = "linux")))]
            DynamicClockId::Dynamic(_fd) => {
                // Dynamic clocks are not supported on this platform.
                return Err(io::Errno::INVAL);
            }

            #[cfg(any(target_os = "android", target_os = "linux"))]
            DynamicClockId::RealtimeAlarm => c::CLOCK_REALTIME_ALARM,

            #[cfg(any(target_os = "android", target_os = "linux"))]
            DynamicClockId::Tai => c::CLOCK_TAI,

            #[cfg(any(target_os = "android", target_os = "linux"))]
            DynamicClockId::Boottime => c::CLOCK_BOOTTIME,

            #[cfg(any(target_os = "android", target_os = "linux"))]
            DynamicClockId::BoottimeAlarm => c::CLOCK_BOOTTIME_ALARM,
        };

        #[cfg(all(
            any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
            target_env = "gnu",
        ))]
        {
            if let Some(libc_clock_gettime) = __clock_gettime64.get() {
                ret(libc_clock_gettime(
                    id as c::clockid_t,
                    timespec.as_mut_ptr(),
                ))?;

                Ok(timespec.assume_init().into())
            } else {
                clock_gettime_dynamic_old(id)
            }
        }

        #[cfg(not(all(
            any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
            target_env = "gnu",
        )))]
        {
            ret(c::clock_gettime(id as c::clockid_t, timespec.as_mut_ptr()))?;

            Ok(timespec.assume_init())
        }
    }
}

#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[inline]
unsafe fn clock_gettime_dynamic_old(id: c::clockid_t) -> io::Result<Timespec> {
    let mut old_timespec = MaybeUninit::<c::timespec>::uninit();

    ret(c::clock_gettime(
        id as c::clockid_t,
        old_timespec.as_mut_ptr(),
    ))?;

    let old_timespec = old_timespec.assume_init();
    Ok(Timespec {
        tv_sec: old_timespec.tv_sec.into(),
        tv_nsec: old_timespec.tv_nsec.into(),
    })
}

#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
#[cfg(feature = "time")]
pub(crate) fn timerfd_create(id: TimerfdClockId, flags: TimerfdFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(c::timerfd_create(id as c::clockid_t, flags.bits())) }
}

#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
#[cfg(feature = "time")]
pub(crate) fn timerfd_settime(
    fd: BorrowedFd<'_>,
    flags: TimerfdTimerFlags,
    new_value: &Itimerspec,
) -> io::Result<Itimerspec> {
    let mut result = MaybeUninit::<LibcItimerspec>::uninit();

    #[cfg(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    ))]
    unsafe {
        if let Some(libc_timerfd_settime) = __timerfd_settime64.get() {
            ret(libc_timerfd_settime(
                borrowed_fd(fd),
                flags.bits(),
                &new_value.clone().into(),
                result.as_mut_ptr(),
            ))
            .map(|()| result.assume_init().into())
        } else {
            timerfd_settime_old(fd, flags, new_value)
        }
    }

    #[cfg(not(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    )))]
    unsafe {
        ret(c::timerfd_settime(
            borrowed_fd(fd),
            flags.bits(),
            new_value,
            result.as_mut_ptr(),
        ))
        .map(|()| result.assume_init())
    }
}

#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[cfg(feature = "time")]
unsafe fn timerfd_settime_old(
    fd: BorrowedFd<'_>,
    flags: TimerfdTimerFlags,
    new_value: &Itimerspec,
) -> io::Result<Itimerspec> {
    use core::convert::TryInto;

    let mut old_result = MaybeUninit::<c::itimerspec>::uninit();

    // Convert `new_value` to the old `itimerspec` format.
    let old_new_value = c::itimerspec {
        it_interval: c::timespec {
            tv_sec: new_value
                .it_interval
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: new_value
                .it_interval
                .tv_nsec
                .try_into()
                .map_err(|_| io::Errno::INVAL)?,
        },
        it_value: c::timespec {
            tv_sec: new_value
                .it_value
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: new_value
                .it_value
                .tv_nsec
                .try_into()
                .map_err(|_| io::Errno::INVAL)?,
        },
    };

    ret(c::timerfd_settime(
        borrowed_fd(fd),
        flags.bits(),
        &old_new_value,
        old_result.as_mut_ptr(),
    ))?;

    let old_result = old_result.assume_init();
    Ok(Itimerspec {
        it_interval: Timespec {
            tv_sec: old_result
                .it_interval
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: old_result.it_interval.tv_nsec as _,
        },
        it_value: Timespec {
            tv_sec: old_result
                .it_interval
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: old_result.it_interval.tv_nsec as _,
        },
    })
}

#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
#[cfg(feature = "time")]
pub(crate) fn timerfd_gettime(fd: BorrowedFd<'_>) -> io::Result<Itimerspec> {
    let mut result = MaybeUninit::<LibcItimerspec>::uninit();

    #[cfg(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    ))]
    unsafe {
        if let Some(libc_timerfd_gettime) = __timerfd_gettime64.get() {
            ret(libc_timerfd_gettime(borrowed_fd(fd), result.as_mut_ptr()))
                .map(|()| result.assume_init().into())
        } else {
            timerfd_gettime_old(fd)
        }
    }

    #[cfg(not(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    )))]
    unsafe {
        ret(c::timerfd_gettime(borrowed_fd(fd), result.as_mut_ptr())).map(|()| result.assume_init())
    }
}

#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[cfg(feature = "time")]
unsafe fn timerfd_gettime_old(fd: BorrowedFd<'_>) -> io::Result<Itimerspec> {
    use core::convert::TryInto;

    let mut old_result = MaybeUninit::<c::itimerspec>::uninit();

    ret(c::timerfd_gettime(borrowed_fd(fd), old_result.as_mut_ptr()))?;

    let old_result = old_result.assume_init();
    Ok(Itimerspec {
        it_interval: Timespec {
            tv_sec: old_result
                .it_interval
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: old_result.it_interval.tv_nsec as _,
        },
        it_value: Timespec {
            tv_sec: old_result
                .it_interval
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: old_result.it_interval.tv_nsec as _,
        },
    })
}
