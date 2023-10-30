use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/Temporal.js")]
extern "C" {
    fn is_temporal_supported() -> bool;
}

#[wasm_bindgen_test]
fn now() {
    if !is_temporal_supported() {
        return;
    }
    let x = Temporal::Now::plain_date("iso8086");
    let y: JsValue = x.into();
    assert!(y.is_object());
}

#[wasm_bindgen_test]
fn duration() {
    if !is_temporal_supported() {
        return;
    }
    let x = Temporal::Duration::from(&"PT1.00001S".into());
    assert_eq!(x.years(), 0);
    assert_eq!(x.microseconds(), 10);
}

#[wasm_bindgen_test]
fn zoned_date_time() {
    if !is_temporal_supported() {
        return;
    }
    let zdt = Temporal::ZonedDateTime::from(&"2020-03-09T00:00-07:00[America/Los_Angeles]".into());
    let earlier_day = zdt.subtract(
        &Temporal::Duration::from(&"P1D".into()),
        &JsValue::UNDEFINED,
    );
    let earlier_hours = zdt.subtract(
        &Temporal::Duration::from(&"PT24H".into()),
        &JsValue::UNDEFINED,
    );
    assert_eq!(earlier_day.hour(), 0);
    assert_eq!(earlier_hours.hour(), 23);
}

#[wasm_bindgen_test]
fn plain_time() {
    if !is_temporal_supported() {
        return;
    }
    let time = Temporal::PlainTime::from(&"19:39:09.068346205".into());
    assert_eq!(time.microsecond(), 346);
    assert_eq!(time.round(&"hour".into()).hour(), 20);
}

#[wasm_bindgen_test]
fn plain_date() {
    if !is_temporal_supported() {
        return;
    }
    let x = Temporal::PlainDate::new(2022, 2, 2).with_calendar("persian");
    let y = Temporal::PlainDate::new(2022, 2, 5);
    let z = Temporal::PlainDate::from(&"-002022-02-02[u-ca=gregory]".into());
    assert_eq!(Temporal::PlainDate::compare(&x, &y), -1);
    assert_eq!(x.year(), 1400);
    assert_eq!(x.month(), 11);
    assert_eq!(x.month_code(), "M11");
    assert_eq!(x.day(), 13);
    assert_eq!(x.era(), "ap");
    assert_eq!(y.era(), JsValue::UNDEFINED);
    assert_eq!(z.era_year(), 2023i32);
    assert_eq!(y.day_of_week(), 6);
    assert_eq!(
        Reflect::get(&x.get_iso_fields(), &"isoYear".into()).unwrap(),
        2022i32
    );
    assert_eq!(x.in_leap_year(), false);
}
