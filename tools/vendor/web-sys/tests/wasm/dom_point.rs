use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use web_sys::{DomPoint, DomPointReadOnly};

#[wasm_bindgen_test]
fn dom_point() {
    let x = DomPoint::new_with_x_and_y_and_z_and_w(1.0, 2.0, 3.0, 4.0).unwrap();
    assert_eq!(x.x(), 1.0);
    x.set_x(1.5);
    assert_eq!(x.x(), 1.5);

    assert_eq!(x.y(), 2.0);
    x.set_y(2.5);
    assert_eq!(x.y(), 2.5);

    assert_eq!(x.z(), 3.0);
    x.set_z(3.5);
    assert_eq!(x.z(), 3.5);

    assert_eq!(x.w(), 4.0);
    x.set_w(4.5);
    assert_eq!(x.w(), 4.5);
}

#[wasm_bindgen_test]
fn dom_point_readonly() {
    let x = DomPoint::new_with_x_and_y_and_z_and_w(1.0, 2.0, 3.0, 4.0).unwrap();
    let x = DomPointReadOnly::from(JsValue::from(x));
    assert_eq!(x.x(), 1.0);
    assert_eq!(x.y(), 2.0);
    assert_eq!(x.z(), 3.0);
    assert_eq!(x.w(), 4.0);
}
