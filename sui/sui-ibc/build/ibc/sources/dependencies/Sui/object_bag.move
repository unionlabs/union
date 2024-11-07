// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Similar to `sui::bag`, an `ObjectBag` is a heterogeneous map-like collection. But unlike
/// `sui::bag`, the values bound to these dynamic fields _must_ be objects themselves. This allows
/// for the objects to still exist in storage, which may be important for external tools.
/// The difference is otherwise not observable from within Move.
module sui::object_bag;

use sui::dynamic_object_field as ofield;

// Attempted to destroy a non-empty bag
const EBagNotEmpty: u64 = 0;

public struct ObjectBag has key, store {
    /// the ID of this bag
    id: UID,
    /// the number of key-value pairs in the bag
    size: u64,
}

/// Creates a new, empty bag
public fun new(ctx: &mut TxContext): ObjectBag {
    ObjectBag {
        id: object::new(ctx),
        size: 0,
    }
}

/// Adds a key-value pair to the bag `bag: &mut ObjectBag`
/// Aborts with `sui::dynamic_field::EFieldAlreadyExists` if the bag already has an entry with
/// that key `k: K`.
public fun add<K: copy + drop + store, V: key + store>(bag: &mut ObjectBag, k: K, v: V) {
    ofield::add(&mut bag.id, k, v);
    bag.size = bag.size + 1;
}

#[syntax(index)]
/// Immutably borrows the value associated with the key in the bag `bag: &ObjectBag`.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the bag does not have an entry with
/// that key `k: K`.
/// Aborts with `sui::dynamic_field::EFieldTypeMismatch` if the bag has an entry for the key, but
/// the value does not have the specified type.
public fun borrow<K: copy + drop + store, V: key + store>(bag: &ObjectBag, k: K): &V {
    ofield::borrow(&bag.id, k)
}

#[syntax(index)]
/// Mutably borrows the value associated with the key in the bag `bag: &mut ObjectBag`.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the bag does not have an entry with
/// that key `k: K`.
/// Aborts with `sui::dynamic_field::EFieldTypeMismatch` if the bag has an entry for the key, but
/// the value does not have the specified type.
public fun borrow_mut<K: copy + drop + store, V: key + store>(bag: &mut ObjectBag, k: K): &mut V {
    ofield::borrow_mut(&mut bag.id, k)
}

/// Mutably borrows the key-value pair in the bag `bag: &mut ObjectBag` and returns the value.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the bag does not have an entry with
/// that key `k: K`.
/// Aborts with `sui::dynamic_field::EFieldTypeMismatch` if the bag has an entry for the key, but
/// the value does not have the specified type.
public fun remove<K: copy + drop + store, V: key + store>(bag: &mut ObjectBag, k: K): V {
    let v = ofield::remove(&mut bag.id, k);
    bag.size = bag.size - 1;
    v
}

/// Returns true iff there is an value associated with the key `k: K` in the bag `bag: &ObjectBag`
public fun contains<K: copy + drop + store>(bag: &ObjectBag, k: K): bool {
    ofield::exists_<K>(&bag.id, k)
}

/// Returns true iff there is an value associated with the key `k: K` in the bag `bag: &ObjectBag`
/// with an assigned value of type `V`
public fun contains_with_type<K: copy + drop + store, V: key + store>(bag: &ObjectBag, k: K): bool {
    ofield::exists_with_type<K, V>(&bag.id, k)
}

/// Returns the size of the bag, the number of key-value pairs
public fun length(bag: &ObjectBag): u64 {
    bag.size
}

/// Returns true iff the bag is empty (if `length` returns `0`)
public fun is_empty(bag: &ObjectBag): bool {
    bag.size == 0
}

/// Destroys an empty bag
/// Aborts with `EBagNotEmpty` if the bag still contains values
public fun destroy_empty(bag: ObjectBag) {
    let ObjectBag { id, size } = bag;
    assert!(size == 0, EBagNotEmpty);
    id.delete()
}

/// Returns the ID of the object associated with the key if the bag has an entry with key `k: K`
/// Returns none otherwise
public fun value_id<K: copy + drop + store>(bag: &ObjectBag, k: K): Option<ID> {
    ofield::id(&bag.id, k)
}
