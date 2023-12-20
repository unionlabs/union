//! Get the system's UTC offset on Unix.

use core::mem::MaybeUninit;

use crate::util::local_offset::{self, Soundness};
use crate::{OffsetDateTime, UtcOffset};

/// Whether the operating system has a thread-safe environment. This allows bypassing the check for
/// if the process is multi-threaded.
// This is the same value as `cfg!(target_os = "x")`.
// Use byte-strings to work around current limitations of const eval.
const OS_HAS_THREAD_SAFE_ENVIRONMENT: bool = match std::env::consts::OS.as_bytes() {
    // https://github.com/illumos/illumos-gate/blob/0fb96ba1f1ce26ff8b286f8f928769a6afcb00a6/usr/src/lib/libc/port/gen/getenv.c
    b"illumos"
    // https://github.com/NetBSD/src/blob/f45028636a44111bc4af44d460924958a4460844/lib/libc/stdlib/getenv.c
    // https://github.com/NetBSD/src/blob/f45028636a44111bc4af44d460924958a4460844/lib/libc/stdlib/setenv.c
    | b"netbsd"
    => true,
    _ => false,
};

/// Convert the given Unix timestamp to a `libc::tm`. Returns `None` on any error.
///
/// # Safety
///
/// This method must only be called when the process is single-threaded.
///
/// This method will remain `unsafe` until `std::env::set_var` is deprecated or has its behavior
/// altered. This method is, on its own, safe. It is the presence of a safe, unsound way to set
/// environment variables that makes it unsafe.
unsafe fn timestamp_to_tm(timestamp: i64) -> Option<libc::tm> {
    extern "C" {
        #[cfg_attr(target_os = "netbsd", link_name = "__tzset50")]
        fn tzset();
    }

    // The exact type of `timestamp` beforehand can vary, so this conversion is necessary.
    #[allow(clippy::useless_conversion)]
    let timestamp = timestamp.try_into().ok()?;

    let mut tm = MaybeUninit::uninit();

    // Update timezone information from system. `localtime_r` does not do this for us.
    //
    // Safety: tzset is thread-safe.
    unsafe { tzset() };

    // Safety: We are calling a system API, which mutates the `tm` variable. If a null
    // pointer is returned, an error occurred.
    let tm_ptr = unsafe { libc::localtime_r(&timestamp, tm.as_mut_ptr()) };

    if tm_ptr.is_null() {
        None
    } else {
        // Safety: The value was initialized, as we no longer have a null pointer.
        Some(unsafe { tm.assume_init() })
    }
}

/// Convert a `libc::tm` to a `UtcOffset`. Returns `None` on any error.
// This is available to any target known to have the `tm_gmtoff` extension.
#[cfg(any(
    target_os = "redox",
    target_os = "linux",
    target_os = "l4re",
    target_os = "android",
    target_os = "emscripten",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "haiku",
))]
fn tm_to_offset(_unix_timestamp: i64, tm: libc::tm) -> Option<UtcOffset> {
    let seconds = tm.tm_gmtoff.try_into().ok()?;
    UtcOffset::from_whole_seconds(seconds).ok()
}

/// Convert a `libc::tm` to a `UtcOffset`. Returns `None` on any error.
///
/// This method can return an incorrect value, as it only approximates the `tm_gmtoff` field. The
/// reason for this is that daylight saving time does not start on the same date every year, nor are
/// the rules for daylight saving time the same for every year. This implementation assumes 1970 is
/// equivalent to every other year, which is not always the case.
#[cfg(not(any(
    target_os = "redox",
    target_os = "linux",
    target_os = "l4re",
    target_os = "android",
    target_os = "emscripten",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "haiku",
)))]
fn tm_to_offset(unix_timestamp: i64, tm: libc::tm) -> Option<UtcOffset> {
    use crate::Date;

    let mut tm = tm;
    if tm.tm_sec == 60 {
        // Leap seconds are not currently supported.
        tm.tm_sec = 59;
    }

    let local_timestamp =
        Date::from_ordinal_date(1900 + tm.tm_year, u16::try_from(tm.tm_yday).ok()? + 1)
            .ok()?
            .with_hms(
                tm.tm_hour.try_into().ok()?,
                tm.tm_min.try_into().ok()?,
                tm.tm_sec.try_into().ok()?,
            )
            .ok()?
            .assume_utc()
            .unix_timestamp();

    let diff_secs = (local_timestamp - unix_timestamp).try_into().ok()?;

    UtcOffset::from_whole_seconds(diff_secs).ok()
}

/// Obtain the system's UTC offset.
pub(super) fn local_offset_at(datetime: OffsetDateTime) -> Option<UtcOffset> {
    // Continue to obtaining the UTC offset if and only if the call is sound or the user has
    // explicitly opted out of soundness.
    //
    // Soundness can be guaranteed either by knowledge of the operating system or knowledge that the
    // process is single-threaded. If the process is single-threaded, then the environment cannot
    // be mutated by a different thread in the process while execution of this function is taking
    // place, which can cause a segmentation fault by dereferencing a dangling pointer.
    //
    // If the `num_threads` crate is incapable of determining the number of running threads, then
    // we conservatively return `None` to avoid a soundness bug.

    if OS_HAS_THREAD_SAFE_ENVIRONMENT
        || local_offset::get_soundness() == Soundness::Unsound
        || num_threads::is_single_threaded() == Some(true)
    {
        let unix_timestamp = datetime.unix_timestamp();
        // Safety: We have just confirmed that the process is single-threaded or the user has
        // explicitly opted out of soundness.
        let tm = unsafe { timestamp_to_tm(unix_timestamp) }?;
        tm_to_offset(unix_timestamp, tm)
    } else {
        None
    }
}
