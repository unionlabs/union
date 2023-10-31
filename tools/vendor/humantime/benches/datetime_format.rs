#![feature(test)]
extern crate test;

use std::io::Write;
use std::time::{Duration, UNIX_EPOCH};

use humantime::format_rfc3339;

#[bench]
fn rfc3339_humantime_seconds(b: &mut test::Bencher) {
    let time = UNIX_EPOCH + Duration::new(1_483_228_799, 0);
    let mut buf = Vec::with_capacity(100);
    b.iter(|| {
        buf.truncate(0);
        write!(&mut buf, "{}", format_rfc3339(time)).unwrap()
    });
}

#[bench]
fn rfc3339_chrono(b: &mut test::Bencher) {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use chrono::format::Item;
    use chrono::format::Item::*;
    use chrono::format::Numeric::*;
    use chrono::format::Fixed::*;
    use chrono::format::Pad::*;

    let time = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(1_483_228_799, 0), Utc);
    let mut buf = Vec::with_capacity(100);

    // formatting code from env_logger
    const ITEMS: &[Item<'static>] = {
        &[
            Numeric(Year, Zero),
            Literal("-"),
            Numeric(Month, Zero),
            Literal("-"),
            Numeric(Day, Zero),
            Literal("T"),
            Numeric(Hour, Zero),
            Literal(":"),
            Numeric(Minute, Zero),
            Literal(":"),
            Numeric(Second, Zero),
            Fixed(TimezoneOffsetZ),
        ]
    };


    b.iter(|| {
        buf.truncate(0);
        write!(&mut buf, "{}", time.format_with_items(ITEMS.iter().cloned()))
            .unwrap()
    });
}
