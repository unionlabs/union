use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlMeterElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_meter() -> HtmlMeterElement;
}

#[wasm_bindgen_test]
fn test_meter_element() {
    let meter = new_meter();

    meter.set_min(-5.);
    assert_eq!(
        meter.min(),
        -5.,
        "Meter should have the min value we gave it."
    );

    meter.set_max(5.);
    assert_eq!(
        meter.max(),
        5.,
        "Meter should have the max value we gave it."
    );

    meter.set_value(2.);
    assert_eq!(meter.value(), 2., "Meter should have the value we gave it.");

    meter.set_low(-1.);
    assert_eq!(
        meter.low(),
        -1.,
        "Meter should have the low value we gave it."
    );

    meter.set_high(1.);
    assert_eq!(
        meter.high(),
        1.,
        "Meter should have the high value we gave it."
    );

    meter.set_optimum(3.);
    assert_eq!(
        meter.optimum(),
        3.,
        "Meter should have the optimum value we gave it."
    );

    assert!(
        meter.labels().length() == 0,
        "Our meter shouldn't have any labels associated with it."
    );
}
