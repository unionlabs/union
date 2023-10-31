#![feature(test)]
extern crate test;

use chrono::{DateTime};
use humantime::parse_rfc3339;

#[bench]
fn rfc3339_humantime_seconds(b: &mut test::Bencher) {
    b.iter(|| {
        parse_rfc3339("2018-02-13T23:08:32Z").unwrap()
    });
}

#[bench]
fn datetime_utc_parse_seconds(b: &mut test::Bencher) {
    b.iter(|| {
        DateTime::parse_from_rfc3339("2018-02-13T23:08:32Z").unwrap()
    });
}

#[bench]
fn rfc3339_humantime_millis(b: &mut test::Bencher) {
    b.iter(|| {
        parse_rfc3339("2018-02-13T23:08:32.123Z").unwrap()
    });
}

#[bench]
fn datetime_utc_parse_millis(b: &mut test::Bencher) {
    b.iter(|| {
        DateTime::parse_from_rfc3339("2018-02-13T23:08:32.123Z").unwrap()
    });
}

#[bench]
fn rfc3339_humantime_nanos(b: &mut test::Bencher) {
    b.iter(|| {
        parse_rfc3339("2018-02-13T23:08:32.123456983Z").unwrap()
    });
}

#[bench]
fn datetime_utc_parse_nanos(b: &mut test::Bencher) {
    b.iter(|| {
        DateTime::parse_from_rfc3339("2018-02-13T23:08:32.123456983Z").unwrap()
    });
}
