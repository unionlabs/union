#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

pub mod anchor_element;
pub mod blob;
pub mod body_element;
pub mod br_element;
pub mod button_element;
pub mod console;
pub mod div_element;
pub mod element;
pub mod event;
pub mod head_element;
pub mod headers;
pub mod heading_element;
pub mod history;
pub mod hr_element;
pub mod html_element;
pub mod html_html_element;
pub mod image_data;
pub mod input_element;
//TODO: Both menu-related tests completely break in Chrome, but run fine in Firefox.
//pub mod menu_element;
//pub mod menu_item_element;
pub mod dom_point;
pub mod indexeddb;
pub mod location;
pub mod meta_element;
pub mod meter_element;
pub mod mod_elements;
pub mod olist_element;
pub mod optgroup_element;
pub mod option_element;
pub mod options_collection;
pub mod output_element;
pub mod paragraph_element;
pub mod param_element;
pub mod performance;
pub mod pre_element;
pub mod progress_element;
pub mod quote_element;
pub mod response;
pub mod rtc_rtp_transceiver_direction;
pub mod script_element;
pub mod select_element;
pub mod slot_element;
pub mod span_element;
pub mod style_element;
pub mod table_element;
pub mod title_element;
pub mod whitelisted_immutable_slices;
pub mod xpath_result;

#[wasm_bindgen_test]
fn deref_works() {
    fn _check(a: &web_sys::XmlHttpRequestUpload) {
        let _x: &web_sys::XmlHttpRequestEventTarget = a;
        let _x: &web_sys::EventTarget = a;
        let _x: &js_sys::Object = a;
        let _x: &wasm_bindgen::JsValue = a;
    }
}
