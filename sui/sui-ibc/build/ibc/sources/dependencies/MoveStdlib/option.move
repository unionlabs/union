// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// This module defines the Option type and its methods to represent and handle an optional value.
module std::option;

/// Abstraction of a value that may or may not be present. Implemented with a vector of size
/// zero or one because Move bytecode does not have ADTs.
public struct Option<Element> has copy, drop, store {
    vec: vector<Element>,
}

/// The `Option` is in an invalid state for the operation attempted.
/// The `Option` is `Some` while it should be `None`.
const EOPTION_IS_SET: u64 = 0x40000;
/// The `Option` is in an invalid state for the operation attempted.
/// The `Option` is `None` while it should be `Some`.
const EOPTION_NOT_SET: u64 = 0x40001;

/// Return an empty `Option`
public fun none<Element>(): Option<Element> {
    Option { vec: vector::empty() }
}

/// Return an `Option` containing `e`
public fun some<Element>(e: Element): Option<Element> {
    Option { vec: vector::singleton(e) }
}

/// Return true if `t` does not hold a value
public fun is_none<Element>(t: &Option<Element>): bool {
    t.vec.is_empty()
}

/// Return true if `t` holds a value
public fun is_some<Element>(t: &Option<Element>): bool {
    !t.vec.is_empty()
}

/// Return true if the value in `t` is equal to `e_ref`
/// Always returns `false` if `t` does not hold a value
public fun contains<Element>(t: &Option<Element>, e_ref: &Element): bool {
    t.vec.contains(e_ref)
}

/// Return an immutable reference to the value inside `t`
/// Aborts if `t` does not hold a value
public fun borrow<Element>(t: &Option<Element>): &Element {
    assert!(t.is_some(), EOPTION_NOT_SET);
    &t.vec[0]
}

/// Return a reference to the value inside `t` if it holds one
/// Return `default_ref` if `t` does not hold a value
public fun borrow_with_default<Element>(t: &Option<Element>, default_ref: &Element): &Element {
    let vec_ref = &t.vec;
    if (vec_ref.is_empty()) default_ref
    else &vec_ref[0]
}

/// Return the value inside `t` if it holds one
/// Return `default` if `t` does not hold a value
public fun get_with_default<Element: copy + drop>(t: &Option<Element>, default: Element): Element {
    let vec_ref = &t.vec;
    if (vec_ref.is_empty()) default
    else vec_ref[0]
}

/// Convert the none option `t` to a some option by adding `e`.
/// Aborts if `t` already holds a value
public fun fill<Element>(t: &mut Option<Element>, e: Element) {
    let vec_ref = &mut t.vec;
    if (vec_ref.is_empty()) vec_ref.push_back(e)
    else abort EOPTION_IS_SET
}

/// Convert a `some` option to a `none` by removing and returning the value stored inside `t`
/// Aborts if `t` does not hold a value
public fun extract<Element>(t: &mut Option<Element>): Element {
    assert!(t.is_some(), EOPTION_NOT_SET);
    t.vec.pop_back()
}

/// Return a mutable reference to the value inside `t`
/// Aborts if `t` does not hold a value
public fun borrow_mut<Element>(t: &mut Option<Element>): &mut Element {
    assert!(t.is_some(), EOPTION_NOT_SET);
    &mut t.vec[0]
}

/// Swap the old value inside `t` with `e` and return the old value
/// Aborts if `t` does not hold a value
public fun swap<Element>(t: &mut Option<Element>, e: Element): Element {
    assert!(t.is_some(), EOPTION_NOT_SET);
    let vec_ref = &mut t.vec;
    let old_value = vec_ref.pop_back();
    vec_ref.push_back(e);
    old_value
}

/// Swap the old value inside `t` with `e` and return the old value;
/// or if there is no old value, fill it with `e`.
/// Different from swap(), swap_or_fill() allows for `t` not holding a value.
public fun swap_or_fill<Element>(t: &mut Option<Element>, e: Element): Option<Element> {
    let vec_ref = &mut t.vec;
    let old_value = if (vec_ref.is_empty()) none()
    else some(vec_ref.pop_back());
    vec_ref.push_back(e);
    old_value
}

/// Destroys `t.` If `t` holds a value, return it. Returns `default` otherwise
public fun destroy_with_default<Element: drop>(t: Option<Element>, default: Element): Element {
    let Option { mut vec } = t;
    if (vec.is_empty()) default
    else vec.pop_back()
}

/// Unpack `t` and return its contents
/// Aborts if `t` does not hold a value
public fun destroy_some<Element>(t: Option<Element>): Element {
    assert!(t.is_some(), EOPTION_NOT_SET);
    let Option { mut vec } = t;
    let elem = vec.pop_back();
    vec.destroy_empty();
    elem
}

/// Unpack `t`
/// Aborts if `t` holds a value
public fun destroy_none<Element>(t: Option<Element>) {
    assert!(t.is_none(), EOPTION_IS_SET);
    let Option { vec } = t;
    vec.destroy_empty()
}

/// Convert `t` into a vector of length 1 if it is `Some`,
/// and an empty vector otherwise
public fun to_vec<Element>(t: Option<Element>): vector<Element> {
    let Option { vec } = t;
    vec
}

// === Macro Functions ===

/// Destroy `Option<T>` and call the closure `f` on the value inside if it holds one.
public macro fun destroy<$T>($o: Option<$T>, $f: |$T|) {
    let o = $o;
    o.do!($f);
}

/// Destroy `Option<T>` and call the closure `f` on the value inside if it holds one.
public macro fun do<$T>($o: Option<$T>, $f: |$T|) {
    let o = $o;
    if (o.is_some()) $f(o.destroy_some())
    else o.destroy_none()
}

/// Execute a closure on the value inside `t` if it holds one.
public macro fun do_ref<$T>($o: &Option<$T>, $f: |&$T|) {
    let o = $o;
    if (o.is_some()) $f(o.borrow());
}

/// Execute a closure on the mutable reference to the value inside `t` if it holds one.
public macro fun do_mut<$T>($o: &mut Option<$T>, $f: |&mut $T|) {
    let o = $o;
    if (o.is_some()) $f(o.borrow_mut());
}

/// Select the first `Some` value from the two options, or `None` if both are `None`.
/// Equivalent to Rust's `a.or(b)`.
public macro fun or<$T>($o: Option<$T>, $default: Option<$T>): Option<$T> {
    let o = $o;
    if (o.is_some()) {
        o
    } else {
        o.destroy_none();
        $default
    }
}

/// If the value is `Some`, call the closure `f` on it. Otherwise, return `None`.
/// Equivalent to Rust's `t.and_then(f)`.
public macro fun and<$T, $U>($o: Option<$T>, $f: |$T| -> Option<$U>): Option<$U> {
    let o = $o;
    if (o.is_some()) {
        $f(o.destroy_some())
    } else {
        o.destroy_none();
        none()
    }
}

/// If the value is `Some`, call the closure `f` on it. Otherwise, return `None`.
/// Equivalent to Rust's `t.and_then(f)`.
public macro fun and_ref<$T, $U>($o: &Option<$T>, $f: |&$T| -> Option<$U>): Option<$U> {
    let o = $o;
    if (o.is_some()) $f(o.borrow())
    else none()
}

/// Map an `Option<T>` to `Option<U>` by applying a function to a contained value.
/// Equivalent to Rust's `t.map(f)`.
public macro fun map<$T, $U>($o: Option<$T>, $f: |$T| -> $U): Option<$U> {
    let o = $o;
    if (o.is_some()) {
        some($f(o.destroy_some()))
    } else {
        o.destroy_none();
        none()
    }
}

/// Map an `Option<T>` value to `Option<U>` by applying a function to a contained value by reference.
/// Original `Option<T>` is preserved.
/// Equivalent to Rust's `t.map(f)`.
public macro fun map_ref<$T, $U>($o: &Option<$T>, $f: |&$T| -> $U): Option<$U> {
    let o = $o;
    if (o.is_some()) some($f(o.borrow()))
    else none()
}

/// Return `None` if the value is `None`, otherwise return `Option<T>` if the predicate `f` returns true.
public macro fun filter<$T: drop>($o: Option<$T>, $f: |&$T| -> bool): Option<$T> {
    let o = $o;
    if (o.is_some() && $f(o.borrow())) o
    else none()
}

/// Return `false` if the value is `None`, otherwise return the result of the predicate `f`.
public macro fun is_some_and<$T>($o: &Option<$T>, $f: |&$T| -> bool): bool {
    let o = $o;
    o.is_some() && $f(o.borrow())
}

/// Destroy `Option<T>` and return the value inside if it holds one, or `default` otherwise.
/// Equivalent to Rust's `t.unwrap_or(default)`.
///
/// Note: this function is a more efficient version of `destroy_with_default`, as it does not
/// evaluate the default value unless necessary. The `destroy_with_default` function should be
/// deprecated in favor of this function.
public macro fun destroy_or<$T>($o: Option<$T>, $default: $T): $T {
    let o = $o;
    if (o.is_some()) {
        o.destroy_some()
    } else {
        o.destroy_none();
        $default
    }
}
