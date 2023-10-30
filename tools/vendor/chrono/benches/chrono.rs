//! Benchmarks for chrono that just depend on std
#![cfg(feature = "__internal_bench")]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use chrono::prelude::*;
use chrono::{DateTime, FixedOffset, Local, Utc, __BenchYearFlags};

fn bench_datetime_parse_from_rfc2822(c: &mut Criterion) {
    c.bench_function("bench_datetime_parse_from_rfc2822", |b| {
        b.iter(|| {
            let str = black_box("Wed, 18 Feb 2015 23:16:09 +0000");
            DateTime::parse_from_rfc2822(str).unwrap()
        })
    });
}

fn bench_datetime_parse_from_rfc3339(c: &mut Criterion) {
    c.bench_function("bench_datetime_parse_from_rfc3339", |b| {
        b.iter(|| {
            let str = black_box("2015-02-18T23:59:60.234567+05:00");
            DateTime::parse_from_rfc3339(str).unwrap()
        })
    });
}

fn bench_datetime_from_str(c: &mut Criterion) {
    c.bench_function("bench_datetime_from_str", |b| {
        b.iter(|| {
            use std::str::FromStr;
            let str = black_box("2019-03-30T18:46:57.193Z");
            DateTime::<Utc>::from_str(str).unwrap()
        })
    });
}

fn bench_datetime_to_rfc2822(c: &mut Criterion) {
    let pst = FixedOffset::east_opt(8 * 60 * 60).unwrap();
    let dt = pst
        .from_local_datetime(
            &NaiveDate::from_ymd_opt(2018, 1, 11)
                .unwrap()
                .and_hms_nano_opt(10, 5, 13, 84_660_000)
                .unwrap(),
        )
        .unwrap();
    c.bench_function("bench_datetime_to_rfc2822", |b| b.iter(|| black_box(dt).to_rfc2822()));
}

fn bench_datetime_to_rfc3339(c: &mut Criterion) {
    let pst = FixedOffset::east_opt(8 * 60 * 60).unwrap();
    let dt = pst
        .from_local_datetime(
            &NaiveDate::from_ymd_opt(2018, 1, 11)
                .unwrap()
                .and_hms_nano_opt(10, 5, 13, 84_660_000)
                .unwrap(),
        )
        .unwrap();
    c.bench_function("bench_datetime_to_rfc3339", |b| b.iter(|| black_box(dt).to_rfc3339()));
}

fn bench_year_flags_from_year(c: &mut Criterion) {
    c.bench_function("bench_year_flags_from_year", |b| {
        b.iter(|| {
            for year in -999i32..1000 {
                __BenchYearFlags::from_year(year);
            }
        })
    });
}

fn bench_get_local_time(c: &mut Criterion) {
    c.bench_function("bench_get_local_time", |b| {
        b.iter(|| {
            let _ = Local::now();
        })
    });
}

/// Returns the number of multiples of `div` in the range `start..end`.
///
/// If the range `start..end` is back-to-front, i.e. `start` is greater than `end`, the
/// behaviour is defined by the following equation:
/// `in_between(start, end, div) == - in_between(end, start, div)`.
///
/// When `div` is 1, this is equivalent to `end - start`, i.e. the length of `start..end`.
///
/// # Panics
///
/// Panics if `div` is not positive.
fn in_between(start: i32, end: i32, div: i32) -> i32 {
    assert!(div > 0, "in_between: nonpositive div = {}", div);
    let start = (start.div_euclid(div), start.rem_euclid(div));
    let end = (end.div_euclid(div), end.rem_euclid(div));
    // The lowest multiple of `div` greater than or equal to `start`, divided.
    let start = start.0 + (start.1 != 0) as i32;
    // The lowest multiple of `div` greater than or equal to   `end`, divided.
    let end = end.0 + (end.1 != 0) as i32;
    end - start
}

/// Alternative implementation to `Datelike::num_days_from_ce`
fn num_days_from_ce_alt<Date: Datelike>(date: &Date) -> i32 {
    let year = date.year();
    let diff = move |div| in_between(1, year, div);
    // 365 days a year, one more in leap years. In the gregorian calendar, leap years are all
    // the multiples of 4 except multiples of 100 but including multiples of 400.
    date.ordinal() as i32 + 365 * diff(1) + diff(4) - diff(100) + diff(400)
}

fn bench_num_days_from_ce(c: &mut Criterion) {
    let mut group = c.benchmark_group("num_days_from_ce");
    for year in &[1, 500, 2000, 2019] {
        let d = NaiveDate::from_ymd_opt(*year, 1, 1).unwrap();
        group.bench_with_input(BenchmarkId::new("new", year), &d, |b, y| {
            b.iter(|| num_days_from_ce_alt(y))
        });
        group.bench_with_input(BenchmarkId::new("classic", year), &d, |b, y| {
            b.iter(|| y.num_days_from_ce())
        });
    }
}

criterion_group!(
    benches,
    bench_datetime_parse_from_rfc2822,
    bench_datetime_parse_from_rfc3339,
    bench_datetime_from_str,
    bench_datetime_to_rfc2822,
    bench_datetime_to_rfc3339,
    bench_year_flags_from_year,
    bench_num_days_from_ce,
    bench_get_local_time,
);

criterion_main!(benches);
