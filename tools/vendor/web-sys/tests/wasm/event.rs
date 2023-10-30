use js_sys::{Object, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::Event;

#[wasm_bindgen(module = "/tests/wasm/event.js")]
extern "C" {
    fn new_event() -> Promise;
}

#[wasm_bindgen_test]
async fn event() {
    let result = JsFuture::from(new_event()).await.unwrap();
    let event = Event::from(result);
    // All DOM interfaces should inherit from `Object`.
    assert!(event.is_instance_of::<Object>());
    let _: &Object = event.as_ref();

    // These should match `new Event`.
    assert!(event.bubbles());
    assert!(event.cancelable());
    assert!(event.composed());

    // The default behavior not initially prevented, but after
    // we call `prevent_default` it better be.
    assert!(!event.default_prevented());
    event.prevent_default();
    assert!(event.default_prevented());
}
