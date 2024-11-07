// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Similar to `sui::table`, an `ObjectTable<K, V>` is a map-like collection. But unlike
/// `sui::table`, the values bound to these dynamic fields _must_ be objects themselves. This allows
/// for the objects to still exist within in storage, which may be important for external tools.
/// The difference is otherwise not observable from within Move.
module sui::object_table;

use sui::dynamic_object_field as ofield;

// Attempted to destroy a non-empty table
const ETableNotEmpty: u64 = 0;

public struct ObjectTable<phantom K: copy + drop + store, phantom V: key + store> has key, store {
    /// the ID of this table
    id: UID,
    /// the number of key-value pairs in the table
    size: u64,
}

/// Creates a new, empty table
public fun new<K: copy + drop + store, V: key + store>(ctx: &mut TxContext): ObjectTable<K, V> {
    ObjectTable {
        id: object::new(ctx),
        size: 0,
    }
}

/// Adds a key-value pair to the table `table: &mut ObjectTable<K, V>`
/// Aborts with `sui::dynamic_field::EFieldAlreadyExists` if the table already has an entry with
/// that key `k: K`.
public fun add<K: copy + drop + store, V: key + store>(table: &mut ObjectTable<K, V>, k: K, v: V) {
    ofield::add(&mut table.id, k, v);
    table.size = table.size + 1;
}

#[syntax(index)]
/// Immutable borrows the value associated with the key in the table `table: &ObjectTable<K, V>`.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the table does not have an entry with
/// that key `k: K`.
public fun borrow<K: copy + drop + store, V: key + store>(table: &ObjectTable<K, V>, k: K): &V {
    ofield::borrow(&table.id, k)
}

#[syntax(index)]
/// Mutably borrows the value associated with the key in the table `table: &mut ObjectTable<K, V>`.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the table does not have an entry with
/// that key `k: K`.
public fun borrow_mut<K: copy + drop + store, V: key + store>(
    table: &mut ObjectTable<K, V>,
    k: K,
): &mut V {
    ofield::borrow_mut(&mut table.id, k)
}

/// Removes the key-value pair in the table `table: &mut ObjectTable<K, V>` and returns the value.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the table does not have an entry with
/// that key `k: K`.
public fun remove<K: copy + drop + store, V: key + store>(table: &mut ObjectTable<K, V>, k: K): V {
    let v = ofield::remove(&mut table.id, k);
    table.size = table.size - 1;
    v
}

/// Returns true iff there is a value associated with the key `k: K` in table
/// `table: &ObjectTable<K, V>`
public fun contains<K: copy + drop + store, V: key + store>(table: &ObjectTable<K, V>, k: K): bool {
    ofield::exists_<K>(&table.id, k)
}

/// Returns the size of the table, the number of key-value pairs
public fun length<K: copy + drop + store, V: key + store>(table: &ObjectTable<K, V>): u64 {
    table.size
}

/// Returns true iff the table is empty (if `length` returns `0`)
public fun is_empty<K: copy + drop + store, V: key + store>(table: &ObjectTable<K, V>): bool {
    table.size == 0
}

/// Destroys an empty table
/// Aborts with `ETableNotEmpty` if the table still contains values
public fun destroy_empty<K: copy + drop + store, V: key + store>(table: ObjectTable<K, V>) {
    let ObjectTable { id, size } = table;
    assert!(size == 0, ETableNotEmpty);
    id.delete()
}

/// Returns the ID of the object associated with the key if the table has an entry with key `k: K`
/// Returns none otherwise
public fun value_id<K: copy + drop + store, V: key + store>(
    table: &ObjectTable<K, V>,
    k: K,
): Option<ID> {
    ofield::id(&table.id, k)
}
