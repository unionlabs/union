extern crate autocfg;

use std::env;

fn main() {
    let ac = autocfg::new();

    // If the "i128" feature is explicity requested, don't bother probing for it.
    // It will still cause a build error if that was set improperly.
    if env::var_os("CARGO_FEATURE_I128").is_some() || ac.probe_type("i128") {
        autocfg::emit("has_i128");
    }

    ac.emit_expression_cfg(
        "unsafe { 1f64.to_int_unchecked::<i32>() }",
        "has_to_int_unchecked",
    );

    ac.emit_expression_cfg("1u32.reverse_bits()", "has_reverse_bits");
    ac.emit_expression_cfg("1u32.trailing_ones()", "has_leading_trailing_ones");
    ac.emit_expression_cfg("{ let mut x = 1; x += &2; }", "has_int_assignop_ref");
    ac.emit_expression_cfg("1u32.div_euclid(1u32)", "has_div_euclid");

    if env::var_os("CARGO_FEATURE_STD").is_some() {
        ac.emit_expression_cfg("1f64.copysign(-1f64)", "has_copysign");
    }

    autocfg::rerun_path("build.rs");
}
