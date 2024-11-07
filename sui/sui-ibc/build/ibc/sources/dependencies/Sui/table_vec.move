// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// A basic scalable vector library implemented using `Table`.
module sui::table_vec;

use sui::table::{Self, Table};

public struct TableVec<phantom Element: store> has store {
    /// The contents of the table vector.
    contents: Table<u64, Element>,
}

const EIndexOutOfBound: u64 = 0;
const ETableNonEmpty: u64 = 1;

/// Create an empty TableVec.
public fun empty<Element: store>(ctx: &mut TxContext): TableVec<Element> {
    TableVec {
        contents: table::new(ctx),
    }
}

/// Return a TableVec of size one containing element `e`.
public fun singleton<Element: store>(e: Element, ctx: &mut TxContext): TableVec<Element> {
    let mut t = empty(ctx);
    t.push_back(e);
    t
}

/// Return the length of the TableVec.
public fun length<Element: store>(t: &TableVec<Element>): u64 {
    t.contents.length()
}

/// Return if the TableVec is empty or not.
public fun is_empty<Element: store>(t: &TableVec<Element>): bool {
    t.length() == 0
}

#[syntax(index)]
/// Acquire an immutable reference to the `i`th element of the TableVec `t`.
/// Aborts if `i` is out of bounds.
public fun borrow<Element: store>(t: &TableVec<Element>, i: u64): &Element {
    assert!(t.length() > i, EIndexOutOfBound);
    &t.contents[i]
}

/// Add element `e` to the end of the TableVec `t`.
public fun push_back<Element: store>(t: &mut TableVec<Element>, e: Element) {
    let key = t.length();
    t.contents.add(key, e);
}

#[syntax(index)]
/// Return a mutable reference to the `i`th element in the TableVec `t`.
/// Aborts if `i` is out of bounds.
public fun borrow_mut<Element: store>(t: &mut TableVec<Element>, i: u64): &mut Element {
    assert!(t.length() > i, EIndexOutOfBound);
    &mut t.contents[i]
}

/// Pop an element from the end of TableVec `t`.
/// Aborts if `t` is empty.
public fun pop_back<Element: store>(t: &mut TableVec<Element>): Element {
    let length = length(t);
    assert!(length > 0, EIndexOutOfBound);
    t.contents.remove(length - 1)
}

/// Destroy the TableVec `t`.
/// Aborts if `t` is not empty.
public fun destroy_empty<Element: store>(t: TableVec<Element>) {
    assert!(length(&t) == 0, ETableNonEmpty);
    let TableVec { contents } = t;
    contents.destroy_empty();
}

/// Drop a possibly non-empty TableVec `t`.
/// Usable only if the value type `Element` has the `drop` ability
public fun drop<Element: drop + store>(t: TableVec<Element>) {
    let TableVec { contents } = t;
    contents.drop()
}

/// Swaps the elements at the `i`th and `j`th indices in the TableVec `t`.
/// Aborts if `i` or `j` is out of bounds.
public fun swap<Element: store>(t: &mut TableVec<Element>, i: u64, j: u64) {
    assert!(t.length() > i, EIndexOutOfBound);
    assert!(t.length() > j, EIndexOutOfBound);
    if (i == j) {
        return
    };
    let element_i = t.contents.remove(i);
    let element_j = t.contents.remove(j);
    t.contents.add(j, element_i);
    t.contents.add(i, element_j);
}

/// Swap the `i`th element of the TableVec `t` with the last element and then pop the TableVec.
/// This is O(1), but does not preserve ordering of elements in the TableVec.
/// Aborts if `i` is out of bounds.
public fun swap_remove<Element: store>(t: &mut TableVec<Element>, i: u64): Element {
    assert!(t.length() > i, EIndexOutOfBound);
    let last_idx = t.length() - 1;
    t.swap(i, last_idx);
    t.pop_back()
}

#[test]
fun test_swap() {
    let ctx = &mut sui::tx_context::dummy();
    let mut tv = singleton(0, ctx);
    tv.push_back(1);
    tv.push_back(2);
    tv.push_back(3);
    tv.push_back(4);
    tv.swap(4, 2);
    tv.check_pop(2);
    tv.check_pop(3);
    tv.check_pop(4);
    tv.check_pop(1);
    tv.check_pop(0);
    tv.drop()
}

#[test_only]
fun check_pop(tv: &mut TableVec<u64>, expected_value: u64) {
    let value = tv.pop_back();
    assert!(value == expected_value, value * 100 + expected_value);
}
