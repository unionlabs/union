extern crate proc_macro;

use std::mem;

#[rustversion::attr(before(1.32), ignore)]
#[test]
fn test_proc_macro_span_size() {
    assert_eq!(mem::size_of::<proc_macro::Span>(), 4);
    assert_eq!(mem::size_of::<Option<proc_macro::Span>>(), 4);
}

#[cfg_attr(not(all(not(wrap_proc_macro), not(span_locations))), ignore)]
#[test]
fn test_proc_macro2_fallback_span_size_without_locations() {
    assert_eq!(mem::size_of::<proc_macro2::Span>(), 0);
    assert_eq!(mem::size_of::<Option<proc_macro2::Span>>(), 1);
}

#[cfg_attr(not(all(not(wrap_proc_macro), span_locations)), ignore)]
#[test]
fn test_proc_macro2_fallback_span_size_with_locations() {
    assert_eq!(mem::size_of::<proc_macro2::Span>(), 8);
    assert_eq!(mem::size_of::<Option<proc_macro2::Span>>(), 12);
}

#[rustversion::attr(before(1.32), ignore)]
#[rustversion::attr(
    since(1.32),
    cfg_attr(not(all(wrap_proc_macro, not(span_locations))), ignore)
)]
#[test]
fn test_proc_macro2_wrapper_span_size_without_locations() {
    assert_eq!(mem::size_of::<proc_macro2::Span>(), 4);
    assert_eq!(mem::size_of::<Option<proc_macro2::Span>>(), 8);
}

#[cfg_attr(not(all(wrap_proc_macro, span_locations)), ignore)]
#[test]
fn test_proc_macro2_wrapper_span_size_with_locations() {
    assert_eq!(mem::size_of::<proc_macro2::Span>(), 12);
    assert_eq!(mem::size_of::<Option<proc_macro2::Span>>(), 12);
}
