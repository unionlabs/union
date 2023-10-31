//! When generating our web_sys APIs we default to setting slice references that
//! get passed to JS as mutable in case they get mutated in JS.
//!
//! In certain cases we know for sure that the slice will not get mutated - for
//! example when working with the WebGlRenderingContext APIs.
//!
//! These tests ensure that whitelisted methods do indeed accept immutable slices.
//! Especially important since this whitelist is stringly typed and currently
//! maintained by hand.
//!
//! @see https://github.com/rustwasm/wasm-bindgen/issues/1005

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{WebGl2RenderingContext, WebGlRenderingContext, WebSocket};

// Ensure that our whitelisted WebGlRenderingContext methods compile with immutable slices.
fn test_webgl_rendering_context_immutable_slices() {
    let gl = JsValue::null().unchecked_into::<WebGlRenderingContext>();

    gl.vertex_attrib1fv_with_f32_array(0, &[1.]);
    gl.vertex_attrib2fv_with_f32_array(0, &[1.]);
    gl.vertex_attrib3fv_with_f32_array(0, &[1.]);
    gl.vertex_attrib4fv_with_f32_array(0, &[1.]);

    gl.uniform1fv_with_f32_array(None, &[1.]);
    gl.uniform2fv_with_f32_array(None, &[1.]);
    gl.uniform3fv_with_f32_array(None, &[1.]);
    gl.uniform4fv_with_f32_array(None, &[1.]);

    gl.uniform_matrix2fv_with_f32_array(None, false, &[1.]);
    gl.uniform_matrix3fv_with_f32_array(None, false, &[1.]);
    gl.uniform_matrix4fv_with_f32_array(None, false, &[1.]);

    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        Some(&[1]),
    );
    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        Some(&[1]),
    );
    gl.compressed_tex_image_2d_with_u8_array(0, 0, 0, 0, 0, 0, &[1]);
}

// Ensure that our whitelisted WebGl2RenderingContext methods compile with immutable slices.
fn test_webgl2_rendering_context_immutable_slices() {
    let gl = JsValue::null().unchecked_into::<WebGl2RenderingContext>();

    gl.tex_image_3d_with_opt_u8_array(0, 0, 0, 0, 0, 0, 0, 0, 0, Some(&[1]));
    gl.tex_sub_image_3d_with_opt_u8_array(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, Some(&[1]));
    gl.compressed_tex_image_3d_with_u8_array(0, 0, 0, 0, 0, 0, 0, &[1]);
}

// Ensure that our whitelisted WebSocket methods compile with immutable slices.
fn test_websocket_immutable_slices() {
    let ws = JsValue::null().unchecked_into::<WebSocket>();
    ws.send_with_u8_array(&[0]);
}

// TODO:
//#[wasm_bindgen_test]
//fn test_another_types_immutable_slices_here() {
//}
