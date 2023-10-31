use js_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn get_date() {
    let date = Date::new(&"August 19, 1975 23:15:30".into());
    assert_eq!(date.get_date(), 19);
}

#[wasm_bindgen_test]
fn get_day() {
    let date = Date::new(&"August 19, 1975 23:15:30".into());
    assert_eq!(date.get_day(), 2);
}

#[wasm_bindgen_test]
fn get_full_year() {
    let date = Date::new(&"July 20, 1969 00:20:18".into());
    let abbr = Date::new(&"Thu, 06 Sep 12 00:00:00".into());

    assert_eq!(date.get_full_year(), 1969);
    assert_eq!(abbr.get_full_year(), 2012);
}

#[wasm_bindgen_test]
fn get_hours() {
    let date = Date::new(&"March 13, 08 04:20".into());
    assert_eq!(date.get_hours(), 4);
}

#[wasm_bindgen_test]
fn get_milliseconds() {
    let date = Date::new(&"1995-12-17T09:24:00Z".into());
    let ms = Date::new(&"1995-12-17T09:24:00.123Z".into());

    assert_eq!(date.get_milliseconds(), 0);
    assert_eq!(ms.get_milliseconds(), 123);
}

#[wasm_bindgen_test]
fn get_minutes() {
    let date = Date::new(&"March 13, 08 04:20".into());

    assert_eq!(date.get_minutes(), 20);
}

#[wasm_bindgen_test]
fn get_month() {
    let date = Date::new(&"July 20, 69 00:20:18".into());

    assert_eq!(date.get_month(), 6);
}

#[wasm_bindgen_test]
fn get_seconds() {
    let date = Date::new(&"July 20, 69 00:20:18".into());

    assert_eq!(date.get_seconds(), 18);
}

#[wasm_bindgen_test]
fn get_time() {
    let date = Date::new(&"July 20, 69 00:20:18 GMT+00:00".into());

    assert_eq!(date.get_time(), -14254782000.0);
}

#[wasm_bindgen_test]
fn get_timezone_offset() {
    let date1 = Date::new(&"August 19, 1975 23:15:30 GMT+07:00".into());
    let date2 = Date::new(&"August 19, 1975 23:15:30 GMT-02:00".into());
    assert_eq!(date1.get_timezone_offset(), date2.get_timezone_offset());
}

#[wasm_bindgen_test]
fn get_utc_date() {
    let date1 = Date::new(&"August 19, 1975 23:15:30 GMT+11:00".into());
    let date2 = Date::new(&"August 19, 1975 23:15:30 GMT-11:00".into());
    assert_eq!(date1.get_utc_date(), 19);
    assert_eq!(date2.get_utc_date(), 20);
}

#[wasm_bindgen_test]
fn get_utc_day() {
    let date1 = Date::new(&"August 19, 1975 23:15:30 GMT+11:00".into());
    let date2 = Date::new(&"August 19, 1975 23:15:30 GMT-11:00".into());

    assert_eq!(date1.get_utc_day(), 2);
    assert_eq!(date2.get_utc_day(), 3);
}

#[wasm_bindgen_test]
fn get_utc_full_year() {
    let date1 = Date::new(&"December 31, 1975, 23:15:30 GMT+11:00".into());
    let date2 = Date::new(&"December 31, 1975, 23:15:30 GMT-11:00".into());
    assert_eq!(date1.get_utc_full_year(), 1975);
    assert_eq!(date2.get_utc_full_year(), 1976);
}

#[wasm_bindgen_test]
fn get_utc_hours() {
    let date1 = Date::new(&"December 31, 1975, 23:15:30 GMT+11:00".into());
    let date2 = Date::new(&"December 31, 1975, 23:15:30 GMT-11:00".into());

    assert_eq!(date1.get_utc_hours(), 12);
    assert_eq!(date2.get_utc_hours(), 10);
}

#[wasm_bindgen_test]
fn get_utc_milliseconds() {
    let date = Date::new(&"2018-01-02T03:04:05.678Z".into());
    assert_eq!(date.get_utc_milliseconds(), 678);
}

#[wasm_bindgen_test]
fn get_utc_minutes() {
    let date1 = Date::new(&"1 January 2000 03:15:30 GMT+07:00".into());
    let date2 = Date::new(&"1 January 2000 03:15:30 GMT+03:30".into());
    assert_eq!(date1.get_utc_minutes(), 15);
    assert_eq!(date2.get_utc_minutes(), 45);
}

#[wasm_bindgen_test]
fn get_utc_month() {
    let date1 = Date::new(&"December 31, 1975, 23:15:30 GMT+11:00".into());
    let date2 = Date::new(&"December 31, 1975, 23:15:30 GMT-11:00".into());

    assert_eq!(date1.get_utc_month(), 11);
    assert_eq!(date2.get_utc_month(), 0);
}

#[wasm_bindgen_test]
fn get_utc_seconds() {
    let date = Date::new(&"July 20, 1969, 20:18:04 UTC".into());

    assert_eq!(date.get_utc_seconds(), 4);
}

#[wasm_bindgen_test]
fn new() {
    assert!(JsValue::from(Date::new(&JsValue::undefined())).is_object());
}

#[wasm_bindgen_test]
fn new_with_year_month() {
    let date1 = Date::new_with_year_month(1975, 7);

    assert_eq!(date1.get_full_year(), 1975);
    assert_eq!(date1.get_month(), 7);
}

#[wasm_bindgen_test]
fn new_with_year_month_day() {
    let date1 = Date::new_with_year_month_day(1975, 7, 8);

    assert_eq!(date1.get_full_year(), 1975);
    assert_eq!(date1.get_month(), 7);
    assert_eq!(date1.get_date(), 8);
}

#[wasm_bindgen_test]
fn new_with_year_month_day_hr() {
    let date1 = Date::new_with_year_month_day_hr(1975, 7, 8, 4);

    assert_eq!(date1.get_full_year(), 1975);
    assert_eq!(date1.get_month(), 7);
    assert_eq!(date1.get_date(), 8);
    assert_eq!(date1.get_hours(), 4);
}

#[wasm_bindgen_test]
fn new_with_year_month_day_hr_min() {
    let date1 = Date::new_with_year_month_day_hr_min(1975, 7, 8, 4, 35);

    assert_eq!(date1.get_full_year(), 1975);
    assert_eq!(date1.get_month(), 7);
    assert_eq!(date1.get_date(), 8);
    assert_eq!(date1.get_hours(), 4);
    assert_eq!(date1.get_minutes(), 35);
}

#[wasm_bindgen_test]
fn new_with_year_month_day_hr_min_sec() {
    let date1 = Date::new_with_year_month_day_hr_min_sec(1975, 7, 8, 4, 35, 25);

    assert_eq!(date1.get_full_year(), 1975);
    assert_eq!(date1.get_month(), 7);
    assert_eq!(date1.get_date(), 8);
    assert_eq!(date1.get_hours(), 4);
    assert_eq!(date1.get_minutes(), 35);
    assert_eq!(date1.get_seconds(), 25);
}

#[wasm_bindgen_test]
fn new_with_year_month_day_hr_min_sec_milli() {
    let date1 = Date::new_with_year_month_day_hr_min_sec_milli(1975, 7, 8, 4, 35, 25, 300);

    assert_eq!(date1.get_full_year(), 1975);
    assert_eq!(date1.get_month(), 7);
    assert_eq!(date1.get_date(), 8);
    assert_eq!(date1.get_hours(), 4);
    assert_eq!(date1.get_minutes(), 35);
    assert_eq!(date1.get_seconds(), 25);
    assert_eq!(date1.get_milliseconds(), 300);
}

#[wasm_bindgen_test]
fn now() {
    assert!(Date::now() > 0.);
}

#[wasm_bindgen_test]
fn parse() {
    let date = Date::parse("04 Dec 1995 00:12:00 GMT");
    let zero = Date::parse("01 Jan 1970 00:00:00 GMT");

    assert_eq!(date, 818035920000.0);
    assert_eq!(zero, 0.0);
}

#[wasm_bindgen_test]
fn set_date() {
    let event1 = Date::new(&"August 19, 1975 23:15:30".into());
    let event2 = Date::new(&"August 24, 1975 23:15:30".into());

    let ms = event1.set_date(24);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_date(), 24);
}

#[wasm_bindgen_test]
fn set_full_year() {
    let event1 = Date::new(&"August 19, 1975 23:15:30".into());
    let event2 = Date::new(&"August 19, 1976 23:15:30".into());

    let ms = event1.set_full_year(1976);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_full_year(), 1976);
}

#[wasm_bindgen_test]
fn set_full_year_with_month() {
    let event1 = Date::new(&"August 19, 1976 23:15:30".into());

    event1.set_full_year_with_month(1979, 4);

    assert_eq!(event1.get_full_year(), 1979);
    assert_eq!(event1.get_month(), 4);
}

#[wasm_bindgen_test]
fn set_full_year_with_month_date() {
    let event1 = Date::new(&"August 19, 1976 23:15:30".into());

    event1.set_full_year_with_month_date(1979, -1, 25);

    assert_eq!(event1.get_full_year(), 1978);
    assert_eq!(event1.get_month(), 11);
    assert_eq!(event1.get_date(), 25);
}

#[wasm_bindgen_test]
fn set_hours() {
    let event1 = Date::new(&"August 19, 1975 23:15:30".into());
    let event2 = Date::new(&"August 19, 1975 20:15:30".into());

    let ms = event1.set_hours(20);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_hours(), 20);
}

#[wasm_bindgen_test]
fn set_milliseconds() {
    let event = Date::new(&"August 19, 1975 23:15:30".into());

    let ms = event.set_milliseconds(456);

    assert_eq!(ms, event.get_time());
    assert_eq!(event.get_milliseconds(), 456);
}

#[wasm_bindgen_test]
fn set_minutes() {
    let event1 = Date::new(&"August 19, 1975 23:15:30".into());
    let event2 = Date::new(&"August 19, 1975 23:45:30".into());

    let ms = event1.set_minutes(45);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_minutes(), 45);
}

#[wasm_bindgen_test]
fn set_month() {
    let event1 = Date::new(&"August 19, 1975 23:15:30".into());
    let event2 = Date::new(&"April 19, 1975 23:15:30".into());

    let ms = event1.set_month(3);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_month(), 3);
}

#[wasm_bindgen_test]
fn set_seconds() {
    let event1 = Date::new(&"August 19, 1975 23:15:30".into());
    let event2 = Date::new(&"August 19, 1975 23:15:42".into());

    let ms = event1.set_seconds(42);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_seconds(), 42);
}

#[wasm_bindgen_test]
fn set_time() {
    let event1 = Date::new(&"July 1, 1999".into());
    let event2 = Date::new(&JsValue::undefined());

    let ms = event2.set_time(event1.get_time());

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
}

#[wasm_bindgen_test]
fn set_utc_date() {
    let event1 = Date::new(&"August 19, 1975 23:15:30 GMT-3:00".into());
    let event2 = Date::new(&"August 19, 1975 02:15:30 GMT".into());

    let ms = event1.set_utc_date(19);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_utc_date(), 19);
}

#[wasm_bindgen_test]
fn set_utc_full_year() {
    let event1 = Date::new(&"December 31, 1975 23:15:30 GMT-3:00".into());
    let event2 = Date::new(&"January 01, 1975 02:15:30 GMT".into());

    let ms = event1.set_utc_full_year(1975);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_utc_full_year(), 1975);
}

#[wasm_bindgen_test]
fn set_utc_full_year_with_month() {
    let event1 = Date::new(&"December 31, 1975 23:15:30 GMT-3:00".into());

    event1.set_utc_full_year_with_month(1975, 6);

    assert_eq!(event1.get_utc_full_year(), 1975);
    assert_eq!(event1.get_utc_month(), 6);
}

#[wasm_bindgen_test]
fn set_utc_full_year_with_month_date() {
    let event1 = Date::new(&"December 31, 1975 23:15:30 GMT-3:00".into());

    event1.set_utc_full_year_with_month_date(1975, -2, 21);

    assert_eq!(event1.get_utc_full_year(), 1974);
    assert_eq!(event1.get_utc_month(), 10);
    assert_eq!(event1.get_utc_date(), 21);
}

#[wasm_bindgen_test]
fn set_utc_hours() {
    let event1 = Date::new(&"August 19, 1975 23:15:30 GMT-3:00".into());
    let event2 = Date::new(&"August 20, 1975 23:15:30 GMT".into());

    let ms = event1.set_utc_hours(23);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_utc_hours(), 23);
}

#[wasm_bindgen_test]
fn set_utc_milliseconds() {
    let event1 = Date::new(&"1995-12-17T09:24:00Z".into());
    let event2 = Date::new(&"1995-12-17T09:24:00.420Z".into());

    let ms = event1.set_utc_milliseconds(420);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_utc_milliseconds(), 420);
}

#[wasm_bindgen_test]
fn set_utc_minutes() {
    let event1 = Date::new(&"December 31, 1975, 23:15:30 GMT-3:00".into());
    let event2 = Date::new(&"January 01, 1976 02:25:30 GMT".into());

    let ms = event1.set_utc_minutes(25);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_utc_minutes(), 25);
}

#[wasm_bindgen_test]
fn set_utc_month() {
    let event1 = Date::new(&"December 31, 1975 23:15:30 GMT-3:00".into());
    let event2 = Date::new(&"December 01, 1976 02:15:30 GMT".into());

    let ms = event1.set_utc_month(11);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_utc_month(), 11);
}

#[wasm_bindgen_test]
fn set_utc_seconds() {
    let event1 = Date::new(&"December 31, 1975 23:15:30 GMT-3:00".into());
    let event2 = Date::new(&"January 01, 1976 02:15:39 GMT".into());

    let ms = event1.set_utc_seconds(39);

    assert_eq!(ms, event2.get_time());
    assert_eq!(event1.get_time(), event2.get_time());
    assert_eq!(event1.get_utc_seconds(), 39);
}

#[wasm_bindgen_test]
fn to_date_string() {
    // Create the date from date components rather than a string because this
    // constructor interprets it as a date in the local time zone, which is how
    // `toDateString` outputs it. This makes sure that this test will work in any
    // time zone.
    // October is 9 rather than 10 because the months count from 0.
    let date = Date::new_with_year_month_day_hr_min(2011, 9, 5, 14, 48);
    assert_eq!(JsValue::from(date.to_date_string()), "Wed Oct 05 2011");
}

#[wasm_bindgen_test]
fn to_iso_string() {
    let date = Date::new(&"05 October 2011 14:48 UTC".into());
    assert_eq!(
        JsValue::from(date.to_iso_string()),
        "2011-10-05T14:48:00.000Z"
    );
}

#[wasm_bindgen_test]
fn to_json() {
    let date = Date::new(&"August 19, 1975 23:15:30 UTC".into());

    assert_eq!(JsValue::from(date.to_json()), "1975-08-19T23:15:30.000Z");
}

#[wasm_bindgen_test]
fn to_locale_date_string() {
    let date = Date::new(&"August 19, 1975 23:15:30 UTC".into());
    let s = date.to_locale_date_string("de-DE", &JsValue::undefined());
    assert!(s.length() > 0);
}

#[wasm_bindgen_test]
fn to_locale_string() {
    let date = Date::new(&"August 19, 1975 23:15:30 UTC".into());
    let s = date.to_locale_string("de-DE", &JsValue::undefined());
    assert!(s.length() > 0);
}

#[wasm_bindgen_test]
fn to_locale_time_string() {
    let date = Date::new(&"August 19, 1975 23:15:30".into());
    assert_eq!(
        JsValue::from(date.to_locale_time_string("en-US")),
        "11:15:30 PM",
    );
}

#[wasm_bindgen_test]
fn to_string() {
    let date = Date::new(&"August 19, 1975 23:15:30".into());
    let s = JsValue::from(date.to_string()).as_string().unwrap();
    assert_eq!(&s[0..15], "Tue Aug 19 1975");
}

#[wasm_bindgen_test]
fn to_time_string() {
    let date = Date::new(&"August 19, 1975 23:15:30".into());
    let s = JsValue::from(date.to_time_string()).as_string().unwrap();
    assert_eq!(&s[0..8], "23:15:30");
}

#[wasm_bindgen_test]
fn to_utc_string() {
    let date = Date::new(&"14 Jun 2017 00:00:00 PDT".into());
    let s = JsValue::from(date.to_utc_string()).as_string().unwrap();
    assert_eq!(s, "Wed, 14 Jun 2017 07:00:00 GMT");
}

#[wasm_bindgen_test]
fn utc() {
    assert_eq!(Date::utc(2018f64, 6f64), 1530403200000.0);
}

#[wasm_bindgen_test]
fn value_of() {
    let date = Date::new(&Date::utc(2018f64, 6f64).into());
    assert_eq!(date.value_of(), 1530403200000.0);
}

#[wasm_bindgen_test]
fn date_inheritance() {
    let date = Date::new(&"August 19, 1975 23:15:30".into());
    assert!(date.is_instance_of::<Date>());
    assert!(date.is_instance_of::<Object>());
    let _: &Object = date.as_ref();
}
