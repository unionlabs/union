use chrono::offset::TimeZone;
use chrono::Local;
use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};

use std::{path, process};

#[cfg(unix)]
fn verify_against_date_command_local(path: &'static str, dt: NaiveDateTime) {
    let output = process::Command::new(path)
        .arg("-d")
        .arg(format!("{}-{:02}-{:02} {:02}:05:01", dt.year(), dt.month(), dt.day(), dt.hour()))
        .arg("+%Y-%m-%d %H:%M:%S %:z")
        .output()
        .unwrap();

    let date_command_str = String::from_utf8(output.stdout).unwrap();

    // The below would be preferred. At this stage neither earliest() or latest()
    // seems to be consistent with the output of the `date` command, so we simply
    // compare both.
    // let local = Local
    //     .with_ymd_and_hms(year, month, day, hour, 5, 1)
    //     // looks like the "date" command always returns a given time when it is ambiguous
    //     .earliest();

    // if let Some(local) = local {
    //     assert_eq!(format!("{}\n", local), date_command_str);
    // } else {
    //     // we are in a "Spring forward gap" due to DST, and so date also returns ""
    //     assert_eq!("", date_command_str);
    // }

    // This is used while a decision is made wheter the `date` output needs to
    // be exactly matched, or whether LocalResult::Ambigious should be handled
    // differently

    let date = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day()).unwrap();
    match Local.from_local_datetime(&date.and_hms_opt(dt.hour(), 5, 1).unwrap()) {
        chrono::LocalResult::Ambiguous(a, b) => assert!(
            format!("{}\n", a) == date_command_str || format!("{}\n", b) == date_command_str
        ),
        chrono::LocalResult::Single(a) => {
            assert_eq!(format!("{}\n", a), date_command_str);
        }
        chrono::LocalResult::None => {
            assert_eq!("", date_command_str);
        }
    }
}

#[test]
#[cfg(unix)]
fn try_verify_against_date_command() {
    let date_path = "/usr/bin/date";

    if !path::Path::new(date_path).exists() {
        // date command not found, skipping
        // avoid running this on macOS, which has path /bin/date
        // as the required CLI arguments are not present in the
        // macOS build.
        return;
    }

    let mut date = NaiveDate::from_ymd_opt(1975, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();

    while date.year() < 2078 {
        if (1975..=1977).contains(&date.year())
            || (2020..=2022).contains(&date.year())
            || (2073..=2077).contains(&date.year())
        {
            verify_against_date_command_local(date_path, date);
        }

        date += chrono::Duration::hours(1);
    }
}
