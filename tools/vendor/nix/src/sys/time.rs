use std::{cmp, fmt, ops};
use std::time::Duration;
use std::convert::From;
use libc::{timespec, timeval};
#[cfg_attr(target_env = "musl", allow(deprecated))] // https://github.com/rust-lang/libc/issues/1848
pub use libc::{time_t, suseconds_t};

pub trait TimeValLike: Sized {
    #[inline]
    fn zero() -> Self {
        Self::seconds(0)
    }

    #[inline]
    fn hours(hours: i64) -> Self {
        let secs = hours.checked_mul(SECS_PER_HOUR)
            .expect("TimeValLike::hours ouf of bounds");
        Self::seconds(secs)
    }

    #[inline]
    fn minutes(minutes: i64) -> Self {
        let secs = minutes.checked_mul(SECS_PER_MINUTE)
            .expect("TimeValLike::minutes out of bounds");
        Self::seconds(secs)
    }

    fn seconds(seconds: i64) -> Self;
    fn milliseconds(milliseconds: i64) -> Self;
    fn microseconds(microseconds: i64) -> Self;
    fn nanoseconds(nanoseconds: i64) -> Self;

    #[inline]
    fn num_hours(&self) -> i64 {
        self.num_seconds() / 3600
    }

    #[inline]
    fn num_minutes(&self) -> i64 {
        self.num_seconds() / 60
    }

    fn num_seconds(&self) -> i64;
    fn num_milliseconds(&self) -> i64;
    fn num_microseconds(&self) -> i64;
    fn num_nanoseconds(&self) -> i64;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TimeSpec(timespec);

const NANOS_PER_SEC: i64 = 1_000_000_000;
const SECS_PER_MINUTE: i64 = 60;
const SECS_PER_HOUR: i64 = 3600;

#[cfg(target_pointer_width = "64")]
const TS_MAX_SECONDS: i64 = (::std::i64::MAX / NANOS_PER_SEC) - 1;

#[cfg(target_pointer_width = "32")]
const TS_MAX_SECONDS: i64 = ::std::isize::MAX as i64;

const TS_MIN_SECONDS: i64 = -TS_MAX_SECONDS;

// x32 compatibility
// See https://sourceware.org/bugzilla/show_bug.cgi?id=16437
#[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
type timespec_tv_nsec_t = i64;
#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
type timespec_tv_nsec_t = libc::c_long;

impl From<timespec> for TimeSpec {
    fn from(ts: timespec) -> Self {
        Self(ts)
    }
}

impl From<Duration> for TimeSpec {
    fn from(duration: Duration) -> Self {
        Self::from_duration(duration)
    }
}

impl From<TimeSpec> for Duration {
    fn from(timespec: TimeSpec) -> Self {
        Duration::new(timespec.0.tv_sec as u64, timespec.0.tv_nsec as u32)
    }
}

impl AsRef<timespec> for TimeSpec {
    fn as_ref(&self) -> &timespec {
        &self.0
    }
}

impl AsMut<timespec> for TimeSpec {
    fn as_mut(&mut self) -> &mut timespec {
        &mut self.0
    }
}

impl Ord for TimeSpec {
    // The implementation of cmp is simplified by assuming that the struct is
    // normalized.  That is, tv_nsec must always be within [0, 1_000_000_000)
    fn cmp(&self, other: &TimeSpec) -> cmp::Ordering {
        if self.tv_sec() == other.tv_sec() {
            self.tv_nsec().cmp(&other.tv_nsec())
        } else {
            self.tv_sec().cmp(&other.tv_sec())
        }
    }
}

impl PartialOrd for TimeSpec {
    fn partial_cmp(&self, other: &TimeSpec) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TimeValLike for TimeSpec {
    #[inline]
    fn seconds(seconds: i64) -> TimeSpec {
        assert!(seconds >= TS_MIN_SECONDS && seconds <= TS_MAX_SECONDS,
                "TimeSpec out of bounds; seconds={}", seconds);
        #[cfg_attr(target_env = "musl", allow(deprecated))] // https://github.com/rust-lang/libc/issues/1848
        TimeSpec(timespec {tv_sec: seconds as time_t, tv_nsec: 0 })
    }

    #[inline]
    fn milliseconds(milliseconds: i64) -> TimeSpec {
        let nanoseconds = milliseconds.checked_mul(1_000_000)
            .expect("TimeSpec::milliseconds out of bounds");

        TimeSpec::nanoseconds(nanoseconds)
    }

    /// Makes a new `TimeSpec` with given number of microseconds.
    #[inline]
    fn microseconds(microseconds: i64) -> TimeSpec {
        let nanoseconds = microseconds.checked_mul(1_000)
            .expect("TimeSpec::milliseconds out of bounds");

        TimeSpec::nanoseconds(nanoseconds)
    }

    /// Makes a new `TimeSpec` with given number of nanoseconds.
    #[inline]
    fn nanoseconds(nanoseconds: i64) -> TimeSpec {
        let (secs, nanos) = div_mod_floor_64(nanoseconds, NANOS_PER_SEC);
        assert!(secs >= TS_MIN_SECONDS && secs <= TS_MAX_SECONDS,
                "TimeSpec out of bounds");
        #[cfg_attr(target_env = "musl", allow(deprecated))] // https://github.com/rust-lang/libc/issues/1848
        TimeSpec(timespec {tv_sec: secs as time_t,
                           tv_nsec: nanos as timespec_tv_nsec_t })
    }

    fn num_seconds(&self) -> i64 {
        if self.tv_sec() < 0 && self.tv_nsec() > 0 {
            (self.tv_sec() + 1) as i64
        } else {
            self.tv_sec() as i64
        }
    }

    fn num_milliseconds(&self) -> i64 {
        self.num_nanoseconds() / 1_000_000
    }

    fn num_microseconds(&self) -> i64 {
        self.num_nanoseconds() / 1_000_000_000
    }

    fn num_nanoseconds(&self) -> i64 {
        let secs = self.num_seconds() * 1_000_000_000;
        let nsec = self.nanos_mod_sec();
        secs + nsec as i64
    }
}

impl TimeSpec {
    fn nanos_mod_sec(&self) -> timespec_tv_nsec_t {
        if self.tv_sec() < 0 && self.tv_nsec() > 0 {
            self.tv_nsec() - NANOS_PER_SEC as timespec_tv_nsec_t
        } else {
            self.tv_nsec()
        }
    }

    #[cfg_attr(target_env = "musl", allow(deprecated))] // https://github.com/rust-lang/libc/issues/1848
    pub const fn tv_sec(&self) -> time_t {
        self.0.tv_sec
    }

    pub const fn tv_nsec(&self) -> timespec_tv_nsec_t {
        self.0.tv_nsec
    }

    pub const fn from_duration(duration: Duration) -> Self {
        #[cfg_attr(target_env = "musl", allow(deprecated))] // https://github.com/rust-lang/libc/issues/1848
        TimeSpec(timespec {
            tv_sec: duration.as_secs() as time_t,
            tv_nsec: duration.subsec_nanos() as timespec_tv_nsec_t
        })
    }

    pub const fn from_timespec(timespec: timespec) -> Self {
        Self(timespec)
    }
}

impl ops::Neg for TimeSpec {
    type Output = TimeSpec;

    fn neg(self) -> TimeSpec {
        TimeSpec::nanoseconds(-self.num_nanoseconds())
    }
}

impl ops::Add for TimeSpec {
    type Output = TimeSpec;

    fn add(self, rhs: TimeSpec) -> TimeSpec {
        TimeSpec::nanoseconds(
            self.num_nanoseconds() + rhs.num_nanoseconds())
    }
}

impl ops::Sub for TimeSpec {
    type Output = TimeSpec;

    fn sub(self, rhs: TimeSpec) -> TimeSpec {
        TimeSpec::nanoseconds(
            self.num_nanoseconds() - rhs.num_nanoseconds())
    }
}

impl ops::Mul<i32> for TimeSpec {
    type Output = TimeSpec;

    fn mul(self, rhs: i32) -> TimeSpec {
        let usec = self.num_nanoseconds().checked_mul(i64::from(rhs))
            .expect("TimeSpec multiply out of bounds");

        TimeSpec::nanoseconds(usec)
    }
}

impl ops::Div<i32> for TimeSpec {
    type Output = TimeSpec;

    fn div(self, rhs: i32) -> TimeSpec {
        let usec = self.num_nanoseconds() / i64::from(rhs);
        TimeSpec::nanoseconds(usec)
    }
}

impl fmt::Display for TimeSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (abs, sign) = if self.tv_sec() < 0 {
            (-*self, "-")
        } else {
            (*self, "")
        };

        let sec = abs.tv_sec();

        write!(f, "{}", sign)?;

        if abs.tv_nsec() == 0 {
            if abs.tv_sec() == 1 {
                write!(f, "{} second", sec)?;
            } else {
                write!(f, "{} seconds", sec)?;
            }
        } else if abs.tv_nsec() % 1_000_000 == 0 {
            write!(f, "{}.{:03} seconds", sec, abs.tv_nsec() / 1_000_000)?;
        } else if abs.tv_nsec() % 1_000 == 0 {
            write!(f, "{}.{:06} seconds", sec, abs.tv_nsec() / 1_000)?;
        } else {
            write!(f, "{}.{:09} seconds", sec, abs.tv_nsec())?;
        }

        Ok(())
    }
}



#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TimeVal(timeval);

const MICROS_PER_SEC: i64 = 1_000_000;

#[cfg(target_pointer_width = "64")]
const TV_MAX_SECONDS: i64 = (::std::i64::MAX / MICROS_PER_SEC) - 1;

#[cfg(target_pointer_width = "32")]
const TV_MAX_SECONDS: i64 = ::std::isize::MAX as i64;

const TV_MIN_SECONDS: i64 = -TV_MAX_SECONDS;

impl AsRef<timeval> for TimeVal {
    fn as_ref(&self) -> &timeval {
        &self.0
    }
}

impl AsMut<timeval> for TimeVal {
    fn as_mut(&mut self) -> &mut timeval {
        &mut self.0
    }
}

impl Ord for TimeVal {
    // The implementation of cmp is simplified by assuming that the struct is
    // normalized.  That is, tv_usec must always be within [0, 1_000_000)
    fn cmp(&self, other: &TimeVal) -> cmp::Ordering {
        if self.tv_sec() == other.tv_sec() {
            self.tv_usec().cmp(&other.tv_usec())
        } else {
            self.tv_sec().cmp(&other.tv_sec())
        }
    }
}

impl PartialOrd for TimeVal {
    fn partial_cmp(&self, other: &TimeVal) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TimeValLike for TimeVal {
    #[inline]
    fn seconds(seconds: i64) -> TimeVal {
        assert!(seconds >= TV_MIN_SECONDS && seconds <= TV_MAX_SECONDS,
                "TimeVal out of bounds; seconds={}", seconds);
        #[cfg_attr(target_env = "musl", allow(deprecated))] // https://github.com/rust-lang/libc/issues/1848
        TimeVal(timeval {tv_sec: seconds as time_t, tv_usec: 0 })
    }

    #[inline]
    fn milliseconds(milliseconds: i64) -> TimeVal {
        let microseconds = milliseconds.checked_mul(1_000)
            .expect("TimeVal::milliseconds out of bounds");

        TimeVal::microseconds(microseconds)
    }

    /// Makes a new `TimeVal` with given number of microseconds.
    #[inline]
    fn microseconds(microseconds: i64) -> TimeVal {
        let (secs, micros) = div_mod_floor_64(microseconds, MICROS_PER_SEC);
        assert!(secs >= TV_MIN_SECONDS && secs <= TV_MAX_SECONDS,
                "TimeVal out of bounds");
        #[cfg_attr(target_env = "musl", allow(deprecated))] // https://github.com/rust-lang/libc/issues/1848
        TimeVal(timeval {tv_sec: secs as time_t,
                           tv_usec: micros as suseconds_t })
    }

    /// Makes a new `TimeVal` with given number of nanoseconds.  Some precision
    /// will be lost
    #[inline]
    fn nanoseconds(nanoseconds: i64) -> TimeVal {
        let microseconds = nanoseconds / 1000;
        let (secs, micros) = div_mod_floor_64(microseconds, MICROS_PER_SEC);
        assert!(secs >= TV_MIN_SECONDS && secs <= TV_MAX_SECONDS,
                "TimeVal out of bounds");
        #[cfg_attr(target_env = "musl", allow(deprecated))] // https://github.com/rust-lang/libc/issues/1848
        TimeVal(timeval {tv_sec: secs as time_t,
                           tv_usec: micros as suseconds_t })
    }

    fn num_seconds(&self) -> i64 {
        if self.tv_sec() < 0 && self.tv_usec() > 0 {
            (self.tv_sec() + 1) as i64
        } else {
            self.tv_sec() as i64
        }
    }

    fn num_milliseconds(&self) -> i64 {
        self.num_microseconds() / 1_000
    }

    fn num_microseconds(&self) -> i64 {
        let secs = self.num_seconds() * 1_000_000;
        let usec = self.micros_mod_sec();
        secs + usec as i64
    }

    fn num_nanoseconds(&self) -> i64 {
        self.num_microseconds() * 1_000
    }
}

impl TimeVal {
    fn micros_mod_sec(&self) -> suseconds_t {
        if self.tv_sec() < 0 && self.tv_usec() > 0 {
            self.tv_usec() - MICROS_PER_SEC as suseconds_t
        } else {
            self.tv_usec()
        }
    }

    #[cfg_attr(target_env = "musl", allow(deprecated))] // https://github.com/rust-lang/libc/issues/1848
    pub const fn tv_sec(&self) -> time_t {
        self.0.tv_sec
    }

    pub const fn tv_usec(&self) -> suseconds_t {
        self.0.tv_usec
    }
}

impl ops::Neg for TimeVal {
    type Output = TimeVal;

    fn neg(self) -> TimeVal {
        TimeVal::microseconds(-self.num_microseconds())
    }
}

impl ops::Add for TimeVal {
    type Output = TimeVal;

    fn add(self, rhs: TimeVal) -> TimeVal {
        TimeVal::microseconds(
            self.num_microseconds() + rhs.num_microseconds())
    }
}

impl ops::Sub for TimeVal {
    type Output = TimeVal;

    fn sub(self, rhs: TimeVal) -> TimeVal {
        TimeVal::microseconds(
            self.num_microseconds() - rhs.num_microseconds())
    }
}

impl ops::Mul<i32> for TimeVal {
    type Output = TimeVal;

    fn mul(self, rhs: i32) -> TimeVal {
        let usec = self.num_microseconds().checked_mul(i64::from(rhs))
            .expect("TimeVal multiply out of bounds");

        TimeVal::microseconds(usec)
    }
}

impl ops::Div<i32> for TimeVal {
    type Output = TimeVal;

    fn div(self, rhs: i32) -> TimeVal {
        let usec = self.num_microseconds() / i64::from(rhs);
        TimeVal::microseconds(usec)
    }
}

impl fmt::Display for TimeVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (abs, sign) = if self.tv_sec() < 0 {
            (-*self, "-")
        } else {
            (*self, "")
        };

        let sec = abs.tv_sec();

        write!(f, "{}", sign)?;

        if abs.tv_usec() == 0 {
            if abs.tv_sec() == 1 {
                write!(f, "{} second", sec)?;
            } else {
                write!(f, "{} seconds", sec)?;
            }
        } else if abs.tv_usec() % 1000 == 0 {
            write!(f, "{}.{:03} seconds", sec, abs.tv_usec() / 1000)?;
        } else {
            write!(f, "{}.{:06} seconds", sec, abs.tv_usec())?;
        }

        Ok(())
    }
}

impl From<timeval> for TimeVal {
    fn from(tv: timeval) -> Self {
        TimeVal(tv)
    }
}

#[inline]
fn div_mod_floor_64(this: i64, other: i64) -> (i64, i64) {
    (div_floor_64(this, other), mod_floor_64(this, other))
}

#[inline]
fn div_floor_64(this: i64, other: i64) -> i64 {
    match div_rem_64(this, other) {
        (d, r) if (r > 0 && other < 0)
               || (r < 0 && other > 0) => d - 1,
        (d, _)                         => d,
    }
}

#[inline]
fn mod_floor_64(this: i64, other: i64) -> i64 {
    match this % other {
        r if (r > 0 && other < 0)
          || (r < 0 && other > 0) => r + other,
        r                         => r,
    }
}

#[inline]
fn div_rem_64(this: i64, other: i64) -> (i64, i64) {
    (this / other, this % other)
}

#[cfg(test)]
mod test {
    use super::{TimeSpec, TimeVal, TimeValLike};
    use std::time::Duration;

    #[test]
    pub fn test_timespec() {
        assert!(TimeSpec::seconds(1) != TimeSpec::zero());
        assert_eq!(TimeSpec::seconds(1) + TimeSpec::seconds(2),
                   TimeSpec::seconds(3));
        assert_eq!(TimeSpec::minutes(3) + TimeSpec::seconds(2),
                   TimeSpec::seconds(182));
    }

    #[test]
    pub fn test_timespec_from() {
        let duration = Duration::new(123, 123_456_789);
        let timespec = TimeSpec::nanoseconds(123_123_456_789);

        assert_eq!(TimeSpec::from(duration), timespec);
        assert_eq!(Duration::from(timespec), duration);
    }

    #[test]
    pub fn test_timespec_neg() {
        let a = TimeSpec::seconds(1) + TimeSpec::nanoseconds(123);
        let b = TimeSpec::seconds(-1) + TimeSpec::nanoseconds(-123);

        assert_eq!(a, -b);
    }

    #[test]
    pub fn test_timespec_ord() {
        assert!(TimeSpec::seconds(1) == TimeSpec::nanoseconds(1_000_000_000));
        assert!(TimeSpec::seconds(1) < TimeSpec::nanoseconds(1_000_000_001));
        assert!(TimeSpec::seconds(1) > TimeSpec::nanoseconds(999_999_999));
        assert!(TimeSpec::seconds(-1) < TimeSpec::nanoseconds(-999_999_999));
        assert!(TimeSpec::seconds(-1) > TimeSpec::nanoseconds(-1_000_000_001));
    }

    #[test]
    pub fn test_timespec_fmt() {
        assert_eq!(TimeSpec::zero().to_string(), "0 seconds");
        assert_eq!(TimeSpec::seconds(42).to_string(), "42 seconds");
        assert_eq!(TimeSpec::milliseconds(42).to_string(), "0.042 seconds");
        assert_eq!(TimeSpec::microseconds(42).to_string(), "0.000042 seconds");
        assert_eq!(TimeSpec::nanoseconds(42).to_string(), "0.000000042 seconds");
        assert_eq!(TimeSpec::seconds(-86401).to_string(), "-86401 seconds");
    }

    #[test]
    pub fn test_timeval() {
        assert!(TimeVal::seconds(1) != TimeVal::zero());
        assert_eq!(TimeVal::seconds(1) + TimeVal::seconds(2),
                   TimeVal::seconds(3));
        assert_eq!(TimeVal::minutes(3) + TimeVal::seconds(2),
                   TimeVal::seconds(182));
    }

    #[test]
    pub fn test_timeval_ord() {
        assert!(TimeVal::seconds(1) == TimeVal::microseconds(1_000_000));
        assert!(TimeVal::seconds(1) < TimeVal::microseconds(1_000_001));
        assert!(TimeVal::seconds(1) > TimeVal::microseconds(999_999));
        assert!(TimeVal::seconds(-1) < TimeVal::microseconds(-999_999));
        assert!(TimeVal::seconds(-1) > TimeVal::microseconds(-1_000_001));
    }

    #[test]
    pub fn test_timeval_neg() {
        let a = TimeVal::seconds(1) + TimeVal::microseconds(123);
        let b = TimeVal::seconds(-1) + TimeVal::microseconds(-123);

        assert_eq!(a, -b);
    }

    #[test]
    pub fn test_timeval_fmt() {
        assert_eq!(TimeVal::zero().to_string(), "0 seconds");
        assert_eq!(TimeVal::seconds(42).to_string(), "42 seconds");
        assert_eq!(TimeVal::milliseconds(42).to_string(), "0.042 seconds");
        assert_eq!(TimeVal::microseconds(42).to_string(), "0.000042 seconds");
        assert_eq!(TimeVal::nanoseconds(1402).to_string(), "0.000001 seconds");
        assert_eq!(TimeVal::seconds(-86401).to_string(), "-86401 seconds");
    }
}
