use ring::{signature, test};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn signature_impl_test() {
    test::compile_time_assert_clone::<signature::Signature>();
    test::compile_time_assert_copy::<signature::Signature>();
    test::compile_time_assert_send::<signature::Signature>();
    test::compile_time_assert_sync::<signature::Signature>();
}
