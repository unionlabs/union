use super::NaiveTime;
use crate::oldtime::Duration;
use crate::Timelike;
use std::u32;

#[test]
fn test_time_from_hms_milli() {
    assert_eq!(
        NaiveTime::from_hms_milli_opt(3, 5, 7, 0),
        Some(NaiveTime::from_hms_nano_opt(3, 5, 7, 0).unwrap())
    );
    assert_eq!(
        NaiveTime::from_hms_milli_opt(3, 5, 7, 777),
        Some(NaiveTime::from_hms_nano_opt(3, 5, 7, 777_000_000).unwrap())
    );
    assert_eq!(
        NaiveTime::from_hms_milli_opt(3, 5, 7, 1_999),
        Some(NaiveTime::from_hms_nano_opt(3, 5, 7, 1_999_000_000).unwrap())
    );
    assert_eq!(NaiveTime::from_hms_milli_opt(3, 5, 7, 2_000), None);
    assert_eq!(NaiveTime::from_hms_milli_opt(3, 5, 7, 5_000), None); // overflow check
    assert_eq!(NaiveTime::from_hms_milli_opt(3, 5, 7, u32::MAX), None);
}

#[test]
fn test_time_from_hms_micro() {
    assert_eq!(
        NaiveTime::from_hms_micro_opt(3, 5, 7, 0),
        Some(NaiveTime::from_hms_nano_opt(3, 5, 7, 0).unwrap())
    );
    assert_eq!(
        NaiveTime::from_hms_micro_opt(3, 5, 7, 333),
        Some(NaiveTime::from_hms_nano_opt(3, 5, 7, 333_000).unwrap())
    );
    assert_eq!(
        NaiveTime::from_hms_micro_opt(3, 5, 7, 777_777),
        Some(NaiveTime::from_hms_nano_opt(3, 5, 7, 777_777_000).unwrap())
    );
    assert_eq!(
        NaiveTime::from_hms_micro_opt(3, 5, 7, 1_999_999),
        Some(NaiveTime::from_hms_nano_opt(3, 5, 7, 1_999_999_000).unwrap())
    );
    assert_eq!(NaiveTime::from_hms_micro_opt(3, 5, 7, 2_000_000), None);
    assert_eq!(NaiveTime::from_hms_micro_opt(3, 5, 7, 5_000_000), None); // overflow check
    assert_eq!(NaiveTime::from_hms_micro_opt(3, 5, 7, u32::MAX), None);
}

#[test]
fn test_time_hms() {
    assert_eq!(NaiveTime::from_hms_opt(3, 5, 7).unwrap().hour(), 3);
    assert_eq!(
        NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_hour(0),
        Some(NaiveTime::from_hms_opt(0, 5, 7).unwrap())
    );
    assert_eq!(
        NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_hour(23),
        Some(NaiveTime::from_hms_opt(23, 5, 7).unwrap())
    );
    assert_eq!(NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_hour(24), None);
    assert_eq!(NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_hour(u32::MAX), None);

    assert_eq!(NaiveTime::from_hms_opt(3, 5, 7).unwrap().minute(), 5);
    assert_eq!(
        NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_minute(0),
        Some(NaiveTime::from_hms_opt(3, 0, 7).unwrap())
    );
    assert_eq!(
        NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_minute(59),
        Some(NaiveTime::from_hms_opt(3, 59, 7).unwrap())
    );
    assert_eq!(NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_minute(60), None);
    assert_eq!(NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_minute(u32::MAX), None);

    assert_eq!(NaiveTime::from_hms_opt(3, 5, 7).unwrap().second(), 7);
    assert_eq!(
        NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_second(0),
        Some(NaiveTime::from_hms_opt(3, 5, 0).unwrap())
    );
    assert_eq!(
        NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_second(59),
        Some(NaiveTime::from_hms_opt(3, 5, 59).unwrap())
    );
    assert_eq!(NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_second(60), None);
    assert_eq!(NaiveTime::from_hms_opt(3, 5, 7).unwrap().with_second(u32::MAX), None);
}

#[test]
fn test_time_add() {
    macro_rules! check {
        ($lhs:expr, $rhs:expr, $sum:expr) => {{
            assert_eq!($lhs + $rhs, $sum);
            //assert_eq!($rhs + $lhs, $sum);
        }};
    }

    let hmsm = |h, m, s, ms| NaiveTime::from_hms_milli_opt(h, m, s, ms).unwrap();

    check!(hmsm(3, 5, 7, 900), Duration::zero(), hmsm(3, 5, 7, 900));
    check!(hmsm(3, 5, 7, 900), Duration::milliseconds(100), hmsm(3, 5, 8, 0));
    check!(hmsm(3, 5, 7, 1_300), Duration::milliseconds(-1800), hmsm(3, 5, 6, 500));
    check!(hmsm(3, 5, 7, 1_300), Duration::milliseconds(-800), hmsm(3, 5, 7, 500));
    check!(hmsm(3, 5, 7, 1_300), Duration::milliseconds(-100), hmsm(3, 5, 7, 1_200));
    check!(hmsm(3, 5, 7, 1_300), Duration::milliseconds(100), hmsm(3, 5, 7, 1_400));
    check!(hmsm(3, 5, 7, 1_300), Duration::milliseconds(800), hmsm(3, 5, 8, 100));
    check!(hmsm(3, 5, 7, 1_300), Duration::milliseconds(1800), hmsm(3, 5, 9, 100));
    check!(hmsm(3, 5, 7, 900), Duration::seconds(86399), hmsm(3, 5, 6, 900)); // overwrap
    check!(hmsm(3, 5, 7, 900), Duration::seconds(-86399), hmsm(3, 5, 8, 900));
    check!(hmsm(3, 5, 7, 900), Duration::days(12345), hmsm(3, 5, 7, 900));
    check!(hmsm(3, 5, 7, 1_300), Duration::days(1), hmsm(3, 5, 7, 300));
    check!(hmsm(3, 5, 7, 1_300), Duration::days(-1), hmsm(3, 5, 8, 300));

    // regression tests for #37
    check!(hmsm(0, 0, 0, 0), Duration::milliseconds(-990), hmsm(23, 59, 59, 10));
    check!(hmsm(0, 0, 0, 0), Duration::milliseconds(-9990), hmsm(23, 59, 50, 10));
}

#[test]
fn test_time_overflowing_add() {
    let hmsm = |h, m, s, ms| NaiveTime::from_hms_milli_opt(h, m, s, ms).unwrap();

    assert_eq!(
        hmsm(3, 4, 5, 678).overflowing_add_signed(Duration::hours(11)),
        (hmsm(14, 4, 5, 678), 0)
    );
    assert_eq!(
        hmsm(3, 4, 5, 678).overflowing_add_signed(Duration::hours(23)),
        (hmsm(2, 4, 5, 678), 86_400)
    );
    assert_eq!(
        hmsm(3, 4, 5, 678).overflowing_add_signed(Duration::hours(-7)),
        (hmsm(20, 4, 5, 678), -86_400)
    );

    // overflowing_add_signed with leap seconds may be counter-intuitive
    assert_eq!(
        hmsm(3, 4, 5, 1_678).overflowing_add_signed(Duration::days(1)),
        (hmsm(3, 4, 5, 678), 86_400)
    );
    assert_eq!(
        hmsm(3, 4, 5, 1_678).overflowing_add_signed(Duration::days(-1)),
        (hmsm(3, 4, 6, 678), -86_400)
    );
}

#[test]
fn test_time_addassignment() {
    let hms = |h, m, s| NaiveTime::from_hms_opt(h, m, s).unwrap();
    let mut time = hms(12, 12, 12);
    time += Duration::hours(10);
    assert_eq!(time, hms(22, 12, 12));
    time += Duration::hours(10);
    assert_eq!(time, hms(8, 12, 12));
}

#[test]
fn test_time_subassignment() {
    let hms = |h, m, s| NaiveTime::from_hms_opt(h, m, s).unwrap();
    let mut time = hms(12, 12, 12);
    time -= Duration::hours(10);
    assert_eq!(time, hms(2, 12, 12));
    time -= Duration::hours(10);
    assert_eq!(time, hms(16, 12, 12));
}

#[test]
fn test_time_sub() {
    macro_rules! check {
        ($lhs:expr, $rhs:expr, $diff:expr) => {{
            // `time1 - time2 = duration` is equivalent to `time2 - time1 = -duration`
            assert_eq!($lhs.signed_duration_since($rhs), $diff);
            assert_eq!($rhs.signed_duration_since($lhs), -$diff);
        }};
    }

    let hmsm = |h, m, s, ms| NaiveTime::from_hms_milli_opt(h, m, s, ms).unwrap();

    check!(hmsm(3, 5, 7, 900), hmsm(3, 5, 7, 900), Duration::zero());
    check!(hmsm(3, 5, 7, 900), hmsm(3, 5, 7, 600), Duration::milliseconds(300));
    check!(hmsm(3, 5, 7, 200), hmsm(2, 4, 6, 200), Duration::seconds(3600 + 60 + 1));
    check!(
        hmsm(3, 5, 7, 200),
        hmsm(2, 4, 6, 300),
        Duration::seconds(3600 + 60) + Duration::milliseconds(900)
    );

    // treats the leap second as if it coincides with the prior non-leap second,
    // as required by `time1 - time2 = duration` and `time2 - time1 = -duration` equivalence.
    check!(hmsm(3, 5, 7, 200), hmsm(3, 5, 6, 1_800), Duration::milliseconds(400));
    check!(hmsm(3, 5, 7, 1_200), hmsm(3, 5, 6, 1_800), Duration::milliseconds(1400));
    check!(hmsm(3, 5, 7, 1_200), hmsm(3, 5, 6, 800), Duration::milliseconds(1400));

    // additional equality: `time1 + duration = time2` is equivalent to
    // `time2 - time1 = duration` IF AND ONLY IF `time2` represents a non-leap second.
    assert_eq!(hmsm(3, 5, 6, 800) + Duration::milliseconds(400), hmsm(3, 5, 7, 200));
    assert_eq!(hmsm(3, 5, 6, 1_800) + Duration::milliseconds(400), hmsm(3, 5, 7, 200));
}

#[test]
fn test_time_fmt() {
    assert_eq!(
        format!("{}", NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap()),
        "23:59:59.999"
    );
    assert_eq!(
        format!("{}", NaiveTime::from_hms_milli_opt(23, 59, 59, 1_000).unwrap()),
        "23:59:60"
    );
    assert_eq!(
        format!("{}", NaiveTime::from_hms_milli_opt(23, 59, 59, 1_001).unwrap()),
        "23:59:60.001"
    );
    assert_eq!(
        format!("{}", NaiveTime::from_hms_micro_opt(0, 0, 0, 43210).unwrap()),
        "00:00:00.043210"
    );
    assert_eq!(
        format!("{}", NaiveTime::from_hms_nano_opt(0, 0, 0, 6543210).unwrap()),
        "00:00:00.006543210"
    );

    // the format specifier should have no effect on `NaiveTime`
    assert_eq!(
        format!("{:30}", NaiveTime::from_hms_milli_opt(3, 5, 7, 9).unwrap()),
        "03:05:07.009"
    );
}

#[test]
fn test_date_from_str() {
    // valid cases
    let valid = [
        "0:0:0",
        "0:0:0.0000000",
        "0:0:0.0000003",
        " 4 : 3 : 2.1 ",
        " 09:08:07 ",
        " 9:8:07 ",
        "23:59:60.373929310237",
    ];
    for &s in &valid {
        let d = match s.parse::<NaiveTime>() {
            Ok(d) => d,
            Err(e) => panic!("parsing `{}` has failed: {}", s, e),
        };
        let s_ = format!("{:?}", d);
        // `s` and `s_` may differ, but `s.parse()` and `s_.parse()` must be same
        let d_ = match s_.parse::<NaiveTime>() {
            Ok(d) => d,
            Err(e) => {
                panic!("`{}` is parsed into `{:?}`, but reparsing that has failed: {}", s, d, e)
            }
        };
        assert!(
            d == d_,
            "`{}` is parsed into `{:?}`, but reparsed result \
                              `{:?}` does not match",
            s,
            d,
            d_
        );
    }

    // some invalid cases
    // since `ParseErrorKind` is private, all we can do is to check if there was an error
    assert!("".parse::<NaiveTime>().is_err());
    assert!("x".parse::<NaiveTime>().is_err());
    assert!("15".parse::<NaiveTime>().is_err());
    assert!("15:8".parse::<NaiveTime>().is_err());
    assert!("15:8:x".parse::<NaiveTime>().is_err());
    assert!("15:8:9x".parse::<NaiveTime>().is_err());
    assert!("23:59:61".parse::<NaiveTime>().is_err());
    assert!("12:34:56.x".parse::<NaiveTime>().is_err());
    assert!("12:34:56. 0".parse::<NaiveTime>().is_err());
}

#[test]
fn test_time_parse_from_str() {
    let hms = |h, m, s| NaiveTime::from_hms_opt(h, m, s).unwrap();
    assert_eq!(
        NaiveTime::parse_from_str("2014-5-7T12:34:56+09:30", "%Y-%m-%dT%H:%M:%S%z"),
        Ok(hms(12, 34, 56))
    ); // ignore date and offset
    assert_eq!(NaiveTime::parse_from_str("PM 12:59", "%P %H:%M"), Ok(hms(12, 59, 0)));
    assert!(NaiveTime::parse_from_str("12:3456", "%H:%M:%S").is_err());
}

#[test]
fn test_time_format() {
    let t = NaiveTime::from_hms_nano_opt(3, 5, 7, 98765432).unwrap();
    assert_eq!(t.format("%H,%k,%I,%l,%P,%p").to_string(), "03, 3,03, 3,am,AM");
    assert_eq!(t.format("%M").to_string(), "05");
    assert_eq!(t.format("%S,%f,%.f").to_string(), "07,098765432,.098765432");
    assert_eq!(t.format("%.3f,%.6f,%.9f").to_string(), ".098,.098765,.098765432");
    assert_eq!(t.format("%R").to_string(), "03:05");
    assert_eq!(t.format("%T,%X").to_string(), "03:05:07,03:05:07");
    assert_eq!(t.format("%r").to_string(), "03:05:07 AM");
    assert_eq!(t.format("%t%n%%%n%t").to_string(), "\t\n%\n\t");

    let t = NaiveTime::from_hms_micro_opt(3, 5, 7, 432100).unwrap();
    assert_eq!(t.format("%S,%f,%.f").to_string(), "07,432100000,.432100");
    assert_eq!(t.format("%.3f,%.6f,%.9f").to_string(), ".432,.432100,.432100000");

    let t = NaiveTime::from_hms_milli_opt(3, 5, 7, 210).unwrap();
    assert_eq!(t.format("%S,%f,%.f").to_string(), "07,210000000,.210");
    assert_eq!(t.format("%.3f,%.6f,%.9f").to_string(), ".210,.210000,.210000000");

    let t = NaiveTime::from_hms_opt(3, 5, 7).unwrap();
    assert_eq!(t.format("%S,%f,%.f").to_string(), "07,000000000,");
    assert_eq!(t.format("%.3f,%.6f,%.9f").to_string(), ".000,.000000,.000000000");

    // corner cases
    assert_eq!(NaiveTime::from_hms_opt(13, 57, 9).unwrap().format("%r").to_string(), "01:57:09 PM");
    assert_eq!(
        NaiveTime::from_hms_milli_opt(23, 59, 59, 1_000).unwrap().format("%X").to_string(),
        "23:59:60"
    );
}
