// This is a part of Chrono.
// See README.md and LICENSE.txt for details.

use crate::datetime::DateTime;
use crate::oldtime::Duration;
use crate::NaiveDateTime;
use crate::TimeZone;
use crate::Timelike;
use core::cmp::Ordering;
use core::fmt;
use core::marker::Sized;
use core::ops::{Add, Sub};

/// Extension trait for subsecond rounding or truncation to a maximum number
/// of digits. Rounding can be used to decrease the error variance when
/// serializing/persisting to lower precision. Truncation is the default
/// behavior in Chrono display formatting.  Either can be used to guarantee
/// equality (e.g. for testing) when round-tripping through a lower precision
/// format.
pub trait SubsecRound {
    /// Return a copy rounded to the specified number of subsecond digits. With
    /// 9 or more digits, self is returned unmodified. Halfway values are
    /// rounded up (away from zero).
    ///
    /// # Example
    /// ``` rust
    /// # use chrono::{DateTime, SubsecRound, Timelike, TimeZone, Utc, NaiveDate};
    /// let dt = NaiveDate::from_ymd_opt(2018, 1, 11).unwrap().and_hms_milli_opt(12, 0, 0, 154).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(dt.round_subsecs(2).nanosecond(), 150_000_000);
    /// assert_eq!(dt.round_subsecs(1).nanosecond(), 200_000_000);
    /// ```
    fn round_subsecs(self, digits: u16) -> Self;

    /// Return a copy truncated to the specified number of subsecond
    /// digits. With 9 or more digits, self is returned unmodified.
    ///
    /// # Example
    /// ``` rust
    /// # use chrono::{DateTime, SubsecRound, Timelike, TimeZone, Utc, NaiveDate};
    /// let dt = NaiveDate::from_ymd_opt(2018, 1, 11).unwrap().and_hms_milli_opt(12, 0, 0, 154).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(dt.trunc_subsecs(2).nanosecond(), 150_000_000);
    /// assert_eq!(dt.trunc_subsecs(1).nanosecond(), 100_000_000);
    /// ```
    fn trunc_subsecs(self, digits: u16) -> Self;
}

impl<T> SubsecRound for T
where
    T: Timelike + Add<Duration, Output = T> + Sub<Duration, Output = T>,
{
    fn round_subsecs(self, digits: u16) -> T {
        let span = span_for_digits(digits);
        let delta_down = self.nanosecond() % span;
        if delta_down > 0 {
            let delta_up = span - delta_down;
            if delta_up <= delta_down {
                self + Duration::nanoseconds(delta_up.into())
            } else {
                self - Duration::nanoseconds(delta_down.into())
            }
        } else {
            self // unchanged
        }
    }

    fn trunc_subsecs(self, digits: u16) -> T {
        let span = span_for_digits(digits);
        let delta_down = self.nanosecond() % span;
        if delta_down > 0 {
            self - Duration::nanoseconds(delta_down.into())
        } else {
            self // unchanged
        }
    }
}

// Return the maximum span in nanoseconds for the target number of digits.
fn span_for_digits(digits: u16) -> u32 {
    // fast lookup form of: 10^(9-min(9,digits))
    match digits {
        0 => 1_000_000_000,
        1 => 100_000_000,
        2 => 10_000_000,
        3 => 1_000_000,
        4 => 100_000,
        5 => 10_000,
        6 => 1_000,
        7 => 100,
        8 => 10,
        _ => 1,
    }
}

/// Extension trait for rounding or truncating a DateTime by a Duration.
///
/// # Limitations
/// Both rounding and truncating are done via [`Duration::num_nanoseconds`] and
/// [`DateTime::timestamp_nanos`]. This means that they will fail if either the
/// `Duration` or the `DateTime` are too big to represented as nanoseconds. They
/// will also fail if the `Duration` is bigger than the timestamp.
pub trait DurationRound: Sized {
    /// Error that can occur in rounding or truncating
    #[cfg(any(feature = "std", test))]
    type Err: std::error::Error;

    /// Error that can occur in rounding or truncating
    #[cfg(not(any(feature = "std", test)))]
    type Err: fmt::Debug + fmt::Display;

    /// Return a copy rounded by Duration.
    ///
    /// # Example
    /// ``` rust
    /// # use chrono::{DateTime, DurationRound, Duration, TimeZone, Utc, NaiveDate};
    /// let dt = NaiveDate::from_ymd_opt(2018, 1, 11).unwrap().and_hms_milli_opt(12, 0, 0, 154).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(
    ///     dt.duration_round(Duration::milliseconds(10)).unwrap().to_string(),
    ///     "2018-01-11 12:00:00.150 UTC"
    /// );
    /// assert_eq!(
    ///     dt.duration_round(Duration::days(1)).unwrap().to_string(),
    ///     "2018-01-12 00:00:00 UTC"
    /// );
    /// ```
    fn duration_round(self, duration: Duration) -> Result<Self, Self::Err>;

    /// Return a copy truncated by Duration.
    ///
    /// # Example
    /// ``` rust
    /// # use chrono::{DateTime, DurationRound, Duration, TimeZone, Utc, NaiveDate};
    /// let dt = NaiveDate::from_ymd_opt(2018, 1, 11).unwrap().and_hms_milli_opt(12, 0, 0, 154).unwrap().and_local_timezone(Utc).unwrap();
    /// assert_eq!(
    ///     dt.duration_trunc(Duration::milliseconds(10)).unwrap().to_string(),
    ///     "2018-01-11 12:00:00.150 UTC"
    /// );
    /// assert_eq!(
    ///     dt.duration_trunc(Duration::days(1)).unwrap().to_string(),
    ///     "2018-01-11 00:00:00 UTC"
    /// );
    /// ```
    fn duration_trunc(self, duration: Duration) -> Result<Self, Self::Err>;
}

/// The maximum number of seconds a DateTime can be to be represented as nanoseconds
const MAX_SECONDS_TIMESTAMP_FOR_NANOS: i64 = 9_223_372_036;

impl<Tz: TimeZone> DurationRound for DateTime<Tz> {
    type Err = RoundingError;

    fn duration_round(self, duration: Duration) -> Result<Self, Self::Err> {
        duration_round(self.naive_local(), self, duration)
    }

    fn duration_trunc(self, duration: Duration) -> Result<Self, Self::Err> {
        duration_trunc(self.naive_local(), self, duration)
    }
}

impl DurationRound for NaiveDateTime {
    type Err = RoundingError;

    fn duration_round(self, duration: Duration) -> Result<Self, Self::Err> {
        duration_round(self, self, duration)
    }

    fn duration_trunc(self, duration: Duration) -> Result<Self, Self::Err> {
        duration_trunc(self, self, duration)
    }
}

fn duration_round<T>(
    naive: NaiveDateTime,
    original: T,
    duration: Duration,
) -> Result<T, RoundingError>
where
    T: Timelike + Add<Duration, Output = T> + Sub<Duration, Output = T>,
{
    if let Some(span) = duration.num_nanoseconds() {
        if naive.timestamp().abs() > MAX_SECONDS_TIMESTAMP_FOR_NANOS {
            return Err(RoundingError::TimestampExceedsLimit);
        }
        let stamp = naive.timestamp_nanos();
        if span > stamp.abs() {
            return Err(RoundingError::DurationExceedsTimestamp);
        }
        if span == 0 {
            return Ok(original);
        }
        let delta_down = stamp % span;
        if delta_down == 0 {
            Ok(original)
        } else {
            let (delta_up, delta_down) = if delta_down < 0 {
                (delta_down.abs(), span - delta_down.abs())
            } else {
                (span - delta_down, delta_down)
            };
            if delta_up <= delta_down {
                Ok(original + Duration::nanoseconds(delta_up))
            } else {
                Ok(original - Duration::nanoseconds(delta_down))
            }
        }
    } else {
        Err(RoundingError::DurationExceedsLimit)
    }
}

fn duration_trunc<T>(
    naive: NaiveDateTime,
    original: T,
    duration: Duration,
) -> Result<T, RoundingError>
where
    T: Timelike + Add<Duration, Output = T> + Sub<Duration, Output = T>,
{
    if let Some(span) = duration.num_nanoseconds() {
        if naive.timestamp().abs() > MAX_SECONDS_TIMESTAMP_FOR_NANOS {
            return Err(RoundingError::TimestampExceedsLimit);
        }
        let stamp = naive.timestamp_nanos();
        if span > stamp.abs() {
            return Err(RoundingError::DurationExceedsTimestamp);
        }
        let delta_down = stamp % span;
        match delta_down.cmp(&0) {
            Ordering::Equal => Ok(original),
            Ordering::Greater => Ok(original - Duration::nanoseconds(delta_down)),
            Ordering::Less => Ok(original - Duration::nanoseconds(span - delta_down.abs())),
        }
    } else {
        Err(RoundingError::DurationExceedsLimit)
    }
}

/// An error from rounding by `Duration`
///
/// See: [`DurationRound`]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum RoundingError {
    /// Error when the Duration exceeds the Duration from or until the Unix epoch.
    ///
    /// ``` rust
    /// # use chrono::{DateTime, DurationRound, Duration, RoundingError, TimeZone, Utc};
    /// let dt = Utc.with_ymd_and_hms(1970, 12, 12, 0, 0, 0).unwrap();
    ///
    /// assert_eq!(
    ///     dt.duration_round(Duration::days(365)),
    ///     Err(RoundingError::DurationExceedsTimestamp),
    /// );
    /// ```
    DurationExceedsTimestamp,

    /// Error when `Duration.num_nanoseconds` exceeds the limit.
    ///
    /// ``` rust
    /// # use chrono::{DateTime, DurationRound, Duration, RoundingError, TimeZone, Utc, NaiveDate};
    /// let dt = NaiveDate::from_ymd_opt(2260, 12, 31).unwrap().and_hms_nano_opt(23, 59, 59, 1_75_500_000).unwrap().and_local_timezone(Utc).unwrap();
    ///
    /// assert_eq!(
    ///     dt.duration_round(Duration::days(300 * 365)),
    ///     Err(RoundingError::DurationExceedsLimit)
    /// );
    /// ```
    DurationExceedsLimit,

    /// Error when `DateTime.timestamp_nanos` exceeds the limit.
    ///
    /// ``` rust
    /// # use chrono::{DateTime, DurationRound, Duration, RoundingError, TimeZone, Utc};
    /// let dt = Utc.with_ymd_and_hms(2300, 12, 12, 0, 0, 0).unwrap();
    ///
    /// assert_eq!(dt.duration_round(Duration::days(1)), Err(RoundingError::TimestampExceedsLimit),);
    /// ```
    TimestampExceedsLimit,
}

impl fmt::Display for RoundingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RoundingError::DurationExceedsTimestamp => {
                write!(f, "duration in nanoseconds exceeds timestamp")
            }
            RoundingError::DurationExceedsLimit => {
                write!(f, "duration exceeds num_nanoseconds limit")
            }
            RoundingError::TimestampExceedsLimit => {
                write!(f, "timestamp exceeds num_nanoseconds limit")
            }
        }
    }
}

#[cfg(any(feature = "std", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for RoundingError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "error from rounding or truncating with DurationRound"
    }
}

#[cfg(test)]
mod tests {
    use super::{Duration, DurationRound, SubsecRound};
    use crate::offset::{FixedOffset, TimeZone, Utc};
    use crate::NaiveDate;
    use crate::Timelike;

    #[test]
    fn test_round_subsecs() {
        let pst = FixedOffset::east_opt(8 * 60 * 60).unwrap();
        let dt = pst
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2018, 1, 11)
                    .unwrap()
                    .and_hms_nano_opt(10, 5, 13, 84_660_684)
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(dt.round_subsecs(10), dt);
        assert_eq!(dt.round_subsecs(9), dt);
        assert_eq!(dt.round_subsecs(8).nanosecond(), 84_660_680);
        assert_eq!(dt.round_subsecs(7).nanosecond(), 84_660_700);
        assert_eq!(dt.round_subsecs(6).nanosecond(), 84_661_000);
        assert_eq!(dt.round_subsecs(5).nanosecond(), 84_660_000);
        assert_eq!(dt.round_subsecs(4).nanosecond(), 84_700_000);
        assert_eq!(dt.round_subsecs(3).nanosecond(), 85_000_000);
        assert_eq!(dt.round_subsecs(2).nanosecond(), 80_000_000);
        assert_eq!(dt.round_subsecs(1).nanosecond(), 100_000_000);

        assert_eq!(dt.round_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.round_subsecs(0).second(), 13);

        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2018, 1, 11)
                    .unwrap()
                    .and_hms_nano_opt(10, 5, 27, 750_500_000)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(dt.round_subsecs(9), dt);
        assert_eq!(dt.round_subsecs(4), dt);
        assert_eq!(dt.round_subsecs(3).nanosecond(), 751_000_000);
        assert_eq!(dt.round_subsecs(2).nanosecond(), 750_000_000);
        assert_eq!(dt.round_subsecs(1).nanosecond(), 800_000_000);

        assert_eq!(dt.round_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.round_subsecs(0).second(), 28);
    }

    #[test]
    fn test_round_leap_nanos() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 1_750_500_000)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(dt.round_subsecs(9), dt);
        assert_eq!(dt.round_subsecs(4), dt);
        assert_eq!(dt.round_subsecs(2).nanosecond(), 1_750_000_000);
        assert_eq!(dt.round_subsecs(1).nanosecond(), 1_800_000_000);
        assert_eq!(dt.round_subsecs(1).second(), 59);

        assert_eq!(dt.round_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.round_subsecs(0).second(), 0);
    }

    #[test]
    fn test_trunc_subsecs() {
        let pst = FixedOffset::east_opt(8 * 60 * 60).unwrap();
        let dt = pst
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2018, 1, 11)
                    .unwrap()
                    .and_hms_nano_opt(10, 5, 13, 84_660_684)
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(dt.trunc_subsecs(10), dt);
        assert_eq!(dt.trunc_subsecs(9), dt);
        assert_eq!(dt.trunc_subsecs(8).nanosecond(), 84_660_680);
        assert_eq!(dt.trunc_subsecs(7).nanosecond(), 84_660_600);
        assert_eq!(dt.trunc_subsecs(6).nanosecond(), 84_660_000);
        assert_eq!(dt.trunc_subsecs(5).nanosecond(), 84_660_000);
        assert_eq!(dt.trunc_subsecs(4).nanosecond(), 84_600_000);
        assert_eq!(dt.trunc_subsecs(3).nanosecond(), 84_000_000);
        assert_eq!(dt.trunc_subsecs(2).nanosecond(), 80_000_000);
        assert_eq!(dt.trunc_subsecs(1).nanosecond(), 0);

        assert_eq!(dt.trunc_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.trunc_subsecs(0).second(), 13);

        let dt = pst
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2018, 1, 11)
                    .unwrap()
                    .and_hms_nano_opt(10, 5, 27, 750_500_000)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(dt.trunc_subsecs(9), dt);
        assert_eq!(dt.trunc_subsecs(4), dt);
        assert_eq!(dt.trunc_subsecs(3).nanosecond(), 750_000_000);
        assert_eq!(dt.trunc_subsecs(2).nanosecond(), 750_000_000);
        assert_eq!(dt.trunc_subsecs(1).nanosecond(), 700_000_000);

        assert_eq!(dt.trunc_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.trunc_subsecs(0).second(), 27);
    }

    #[test]
    fn test_trunc_leap_nanos() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 1_750_500_000)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(dt.trunc_subsecs(9), dt);
        assert_eq!(dt.trunc_subsecs(4), dt);
        assert_eq!(dt.trunc_subsecs(2).nanosecond(), 1_750_000_000);
        assert_eq!(dt.trunc_subsecs(1).nanosecond(), 1_700_000_000);
        assert_eq!(dt.trunc_subsecs(1).second(), 59);

        assert_eq!(dt.trunc_subsecs(0).nanosecond(), 1_000_000_000);
        assert_eq!(dt.trunc_subsecs(0).second(), 59);
    }

    #[test]
    fn test_duration_round() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 175_500_000)
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(
            dt.duration_round(Duration::zero()).unwrap().to_string(),
            "2016-12-31 23:59:59.175500 UTC"
        );

        assert_eq!(
            dt.duration_round(Duration::milliseconds(10)).unwrap().to_string(),
            "2016-12-31 23:59:59.180 UTC"
        );

        // round up
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 30, 0)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            dt.duration_round(Duration::minutes(5)).unwrap().to_string(),
            "2012-12-12 18:25:00 UTC"
        );
        // round down
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 29, 999)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            dt.duration_round(Duration::minutes(5)).unwrap().to_string(),
            "2012-12-12 18:20:00 UTC"
        );

        assert_eq!(
            dt.duration_round(Duration::minutes(10)).unwrap().to_string(),
            "2012-12-12 18:20:00 UTC"
        );
        assert_eq!(
            dt.duration_round(Duration::minutes(30)).unwrap().to_string(),
            "2012-12-12 18:30:00 UTC"
        );
        assert_eq!(
            dt.duration_round(Duration::hours(1)).unwrap().to_string(),
            "2012-12-12 18:00:00 UTC"
        );
        assert_eq!(
            dt.duration_round(Duration::days(1)).unwrap().to_string(),
            "2012-12-13 00:00:00 UTC"
        );

        // timezone east
        let dt =
            FixedOffset::east_opt(3600).unwrap().with_ymd_and_hms(2020, 10, 27, 15, 0, 0).unwrap();
        assert_eq!(
            dt.duration_round(Duration::days(1)).unwrap().to_string(),
            "2020-10-28 00:00:00 +01:00"
        );
        assert_eq!(
            dt.duration_round(Duration::weeks(1)).unwrap().to_string(),
            "2020-10-29 00:00:00 +01:00"
        );

        // timezone west
        let dt =
            FixedOffset::west_opt(3600).unwrap().with_ymd_and_hms(2020, 10, 27, 15, 0, 0).unwrap();
        assert_eq!(
            dt.duration_round(Duration::days(1)).unwrap().to_string(),
            "2020-10-28 00:00:00 -01:00"
        );
        assert_eq!(
            dt.duration_round(Duration::weeks(1)).unwrap().to_string(),
            "2020-10-29 00:00:00 -01:00"
        );
    }

    #[test]
    fn test_duration_round_naive() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 175_500_000)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();

        assert_eq!(
            dt.duration_round(Duration::zero()).unwrap().to_string(),
            "2016-12-31 23:59:59.175500"
        );

        assert_eq!(
            dt.duration_round(Duration::milliseconds(10)).unwrap().to_string(),
            "2016-12-31 23:59:59.180"
        );

        // round up
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 30, 0)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();
        assert_eq!(
            dt.duration_round(Duration::minutes(5)).unwrap().to_string(),
            "2012-12-12 18:25:00"
        );
        // round down
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 29, 999)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();
        assert_eq!(
            dt.duration_round(Duration::minutes(5)).unwrap().to_string(),
            "2012-12-12 18:20:00"
        );

        assert_eq!(
            dt.duration_round(Duration::minutes(10)).unwrap().to_string(),
            "2012-12-12 18:20:00"
        );
        assert_eq!(
            dt.duration_round(Duration::minutes(30)).unwrap().to_string(),
            "2012-12-12 18:30:00"
        );
        assert_eq!(
            dt.duration_round(Duration::hours(1)).unwrap().to_string(),
            "2012-12-12 18:00:00"
        );
        assert_eq!(
            dt.duration_round(Duration::days(1)).unwrap().to_string(),
            "2012-12-13 00:00:00"
        );
    }

    #[test]
    fn test_duration_round_pre_epoch() {
        let dt = Utc.with_ymd_and_hms(1969, 12, 12, 12, 12, 12).unwrap();
        assert_eq!(
            dt.duration_round(Duration::minutes(10)).unwrap().to_string(),
            "1969-12-12 12:10:00 UTC"
        );
    }

    #[test]
    fn test_duration_trunc() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 175_500_000)
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(
            dt.duration_trunc(Duration::milliseconds(10)).unwrap().to_string(),
            "2016-12-31 23:59:59.170 UTC"
        );

        // would round up
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 30, 0)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            dt.duration_trunc(Duration::minutes(5)).unwrap().to_string(),
            "2012-12-12 18:20:00 UTC"
        );
        // would round down
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 29, 999)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            dt.duration_trunc(Duration::minutes(5)).unwrap().to_string(),
            "2012-12-12 18:20:00 UTC"
        );
        assert_eq!(
            dt.duration_trunc(Duration::minutes(10)).unwrap().to_string(),
            "2012-12-12 18:20:00 UTC"
        );
        assert_eq!(
            dt.duration_trunc(Duration::minutes(30)).unwrap().to_string(),
            "2012-12-12 18:00:00 UTC"
        );
        assert_eq!(
            dt.duration_trunc(Duration::hours(1)).unwrap().to_string(),
            "2012-12-12 18:00:00 UTC"
        );
        assert_eq!(
            dt.duration_trunc(Duration::days(1)).unwrap().to_string(),
            "2012-12-12 00:00:00 UTC"
        );

        // timezone east
        let dt =
            FixedOffset::east_opt(3600).unwrap().with_ymd_and_hms(2020, 10, 27, 15, 0, 0).unwrap();
        assert_eq!(
            dt.duration_trunc(Duration::days(1)).unwrap().to_string(),
            "2020-10-27 00:00:00 +01:00"
        );
        assert_eq!(
            dt.duration_trunc(Duration::weeks(1)).unwrap().to_string(),
            "2020-10-22 00:00:00 +01:00"
        );

        // timezone west
        let dt =
            FixedOffset::west_opt(3600).unwrap().with_ymd_and_hms(2020, 10, 27, 15, 0, 0).unwrap();
        assert_eq!(
            dt.duration_trunc(Duration::days(1)).unwrap().to_string(),
            "2020-10-27 00:00:00 -01:00"
        );
        assert_eq!(
            dt.duration_trunc(Duration::weeks(1)).unwrap().to_string(),
            "2020-10-22 00:00:00 -01:00"
        );
    }

    #[test]
    fn test_duration_trunc_naive() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 175_500_000)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();

        assert_eq!(
            dt.duration_trunc(Duration::milliseconds(10)).unwrap().to_string(),
            "2016-12-31 23:59:59.170"
        );

        // would round up
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 30, 0)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();
        assert_eq!(
            dt.duration_trunc(Duration::minutes(5)).unwrap().to_string(),
            "2012-12-12 18:20:00"
        );
        // would round down
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 29, 999)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();
        assert_eq!(
            dt.duration_trunc(Duration::minutes(5)).unwrap().to_string(),
            "2012-12-12 18:20:00"
        );
        assert_eq!(
            dt.duration_trunc(Duration::minutes(10)).unwrap().to_string(),
            "2012-12-12 18:20:00"
        );
        assert_eq!(
            dt.duration_trunc(Duration::minutes(30)).unwrap().to_string(),
            "2012-12-12 18:00:00"
        );
        assert_eq!(
            dt.duration_trunc(Duration::hours(1)).unwrap().to_string(),
            "2012-12-12 18:00:00"
        );
        assert_eq!(
            dt.duration_trunc(Duration::days(1)).unwrap().to_string(),
            "2012-12-12 00:00:00"
        );
    }

    #[test]
    fn test_duration_trunc_pre_epoch() {
        let dt = Utc.with_ymd_and_hms(1969, 12, 12, 12, 12, 12).unwrap();
        assert_eq!(
            dt.duration_trunc(Duration::minutes(10)).unwrap().to_string(),
            "1969-12-12 12:10:00 UTC"
        );
    }
}
