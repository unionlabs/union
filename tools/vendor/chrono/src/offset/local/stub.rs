// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::time::{SystemTime, UNIX_EPOCH};

use super::{FixedOffset, Local};
use crate::{DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

pub(super) fn now() -> DateTime<Local> {
    tm_to_datetime(Timespec::now().local())
}

/// Converts a local `NaiveDateTime` to the `time::Timespec`.
#[cfg(not(all(
    target_arch = "wasm32",
    feature = "wasmbind",
    not(any(target_os = "emscripten", target_os = "wasi"))
)))]
pub(super) fn naive_to_local(d: &NaiveDateTime, local: bool) -> LocalResult<DateTime<Local>> {
    let tm = Tm {
        tm_sec: d.second() as i32,
        tm_min: d.minute() as i32,
        tm_hour: d.hour() as i32,
        tm_mday: d.day() as i32,
        tm_mon: d.month0() as i32, // yes, C is that strange...
        tm_year: d.year() - 1900,  // this doesn't underflow, we know that d is `NaiveDateTime`.
        tm_wday: 0,                // to_local ignores this
        tm_yday: 0,                // and this
        tm_isdst: -1,
        // This seems pretty fake?
        tm_utcoff: if local { 1 } else { 0 },
        // do not set this, OS APIs are heavily inconsistent in terms of leap second handling
        tm_nsec: 0,
    };

    let spec = Timespec {
        sec: match local {
            false => utc_tm_to_time(&tm),
            true => local_tm_to_time(&tm),
        },
        nsec: tm.tm_nsec,
    };

    // Adjust for leap seconds
    let mut tm = spec.local();
    assert_eq!(tm.tm_nsec, 0);
    tm.tm_nsec = d.nanosecond() as i32;

    LocalResult::Single(tm_to_datetime(tm))
}

/// Converts a `time::Tm` struct into the timezone-aware `DateTime`.
/// This assumes that `time` is working correctly, i.e. any error is fatal.
#[cfg(not(all(
    target_arch = "wasm32",
    feature = "wasmbind",
    not(any(target_os = "emscripten", target_os = "wasi"))
)))]
fn tm_to_datetime(mut tm: Tm) -> DateTime<Local> {
    if tm.tm_sec >= 60 {
        tm.tm_nsec += (tm.tm_sec - 59) * 1_000_000_000;
        tm.tm_sec = 59;
    }

    let date = NaiveDate::from_yo(tm.tm_year + 1900, tm.tm_yday as u32 + 1);
    let time = NaiveTime::from_hms_nano(
        tm.tm_hour as u32,
        tm.tm_min as u32,
        tm.tm_sec as u32,
        tm.tm_nsec as u32,
    );

    let offset = FixedOffset::east_opt(tm.tm_utcoff).unwrap();
    DateTime::from_utc(date.and_time(time) - offset, offset)
}

/// A record specifying a time value in seconds and nanoseconds, where
/// nanoseconds represent the offset from the given second.
///
/// For example a timespec of 1.2 seconds after the beginning of the epoch would
/// be represented as {sec: 1, nsec: 200000000}.
struct Timespec {
    sec: i64,
    nsec: i32,
}

impl Timespec {
    /// Constructs a timespec representing the current time in UTC.
    fn now() -> Timespec {
        let st =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before Unix epoch");
        Timespec { sec: st.as_secs() as i64, nsec: st.subsec_nanos() as i32 }
    }

    /// Converts this timespec into the system's local time.
    fn local(self) -> Tm {
        let mut tm = Tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_utcoff: 0,
            tm_nsec: 0,
        };
        time_to_local_tm(self.sec, &mut tm);
        tm.tm_nsec = self.nsec;
        tm
    }
}

/// Holds a calendar date and time broken down into its components (year, month,
/// day, and so on), also called a broken-down time value.
// FIXME: use c_int instead of i32?
#[repr(C)]
pub(super) struct Tm {
    /// Seconds after the minute - [0, 60]
    tm_sec: i32,

    /// Minutes after the hour - [0, 59]
    tm_min: i32,

    /// Hours after midnight - [0, 23]
    tm_hour: i32,

    /// Day of the month - [1, 31]
    tm_mday: i32,

    /// Months since January - [0, 11]
    tm_mon: i32,

    /// Years since 1900
    tm_year: i32,

    /// Days since Sunday - [0, 6]. 0 = Sunday, 1 = Monday, ..., 6 = Saturday.
    tm_wday: i32,

    /// Days since January 1 - [0, 365]
    tm_yday: i32,

    /// Daylight Saving Time flag.
    ///
    /// This value is positive if Daylight Saving Time is in effect, zero if
    /// Daylight Saving Time is not in effect, and negative if this information
    /// is not available.
    tm_isdst: i32,

    /// Identifies the time zone that was used to compute this broken-down time
    /// value, including any adjustment for Daylight Saving Time. This is the
    /// number of seconds east of UTC. For example, for U.S. Pacific Daylight
    /// Time, the value is `-7*60*60 = -25200`.
    tm_utcoff: i32,

    /// Nanoseconds after the second - [0, 10<sup>9</sup> - 1]
    tm_nsec: i32,
}

fn time_to_tm(ts: i64, tm: &mut Tm) {
    let leapyear = |year| -> bool { year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) };

    static YTAB: [[i64; 12]; 2] = [
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    ];

    let mut year = 1970;

    let dayclock = ts % 86400;
    let mut dayno = ts / 86400;

    tm.tm_sec = (dayclock % 60) as i32;
    tm.tm_min = ((dayclock % 3600) / 60) as i32;
    tm.tm_hour = (dayclock / 3600) as i32;
    tm.tm_wday = ((dayno + 4) % 7) as i32;
    loop {
        let yearsize = if leapyear(year) { 366 } else { 365 };
        if dayno >= yearsize {
            dayno -= yearsize;
            year += 1;
        } else {
            break;
        }
    }
    tm.tm_year = (year - 1900) as i32;
    tm.tm_yday = dayno as i32;
    let mut mon = 0;
    while dayno >= YTAB[if leapyear(year) { 1 } else { 0 }][mon] {
        dayno -= YTAB[if leapyear(year) { 1 } else { 0 }][mon];
        mon += 1;
    }
    tm.tm_mon = mon as i32;
    tm.tm_mday = dayno as i32 + 1;
    tm.tm_isdst = 0;
}

fn tm_to_time(tm: &Tm) -> i64 {
    let mut y = tm.tm_year as i64 + 1900;
    let mut m = tm.tm_mon as i64 + 1;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let d = tm.tm_mday as i64;
    let h = tm.tm_hour as i64;
    let mi = tm.tm_min as i64;
    let s = tm.tm_sec as i64;
    (365 * y + y / 4 - y / 100 + y / 400 + 3 * (m + 1) / 5 + 30 * m + d - 719561) * 86400
        + 3600 * h
        + 60 * mi
        + s
}

pub(super) fn time_to_local_tm(sec: i64, tm: &mut Tm) {
    // FIXME: Add timezone logic
    time_to_tm(sec, tm);
}

pub(super) fn utc_tm_to_time(tm: &Tm) -> i64 {
    tm_to_time(tm)
}

pub(super) fn local_tm_to_time(tm: &Tm) -> i64 {
    // FIXME: Add timezone logic
    tm_to_time(tm)
}
