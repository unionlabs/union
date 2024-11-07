// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// A table is a map-like collection. But unlike a traditional collection, it's keys and values are
/// not stored within the `Table` value, but instead are stored using Sui's object system. The
/// `Table` struct acts only as a handle into the object system to retrieve those keys and values.
/// Note that this means that `Table` values with exactly the same key-value mapping will not be
/// equal, with `==`, at runtime. For example
/// ```
/// let table1 = table::new<u64, bool>();
/// let table2 = table::new<u64, bool>();
/// table::add(&mut table1, 0, false);
/// table::add(&mut table1, 1, true);
/// table::add(&mut table2, 0, false);
/// table::add(&mut table2, 1, true);
/// // table1 does not equal table2, despite having the same entries
/// assert!(&table1 != &table2);
/// ```
module sui::table;

use sui::dynamic_field as field;

// Attempted to destroy a non-empty table
const ETableNotEmpty: u64 = 0;

public struct Table<phantom K: copy + drop + store, phantom V: store> has key, store {
    /// the ID of this table
    id: UID,
    /// the number of key-value pairs in the table
    size: u64,
}

/// Creates a new, empty table
public fun new<K: copy + drop + store, V: store>(ctx: &mut TxContext): Table<K, V> {
    Table {
        id: object::new(ctx),
        size: 0,
    }
}

/// Adds a key-value pair to the table `table: &mut Table<K, V>`
/// Aborts with `sui::dynamic_field::EFieldAlreadyExists` if the table already has an entry with
/// that key `k: K`.
public fun add<K: copy + drop + store, V: store>(table: &mut Table<K, V>, k: K, v: V) {
    field::add(&mut table.id, k, v);
    table.size = table.size + 1;
}

#[syntax(index)]
/// Immutable borrows the value associated with the key in the table `table: &Table<K, V>`.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the table does not have an entry with
/// that key `k: K`.
public fun borrow<K: copy + drop + store, V: store>(table: &Table<K, V>, k: K): &V {
    field::borrow(&table.id, k)
}

#[syntax(index)]
/// Mutably borrows the value associated with the key in the table `table: &mut Table<K, V>`.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the table does not have an entry with
/// that key `k: K`.
public fun borrow_mut<K: copy + drop + store, V: store>(table: &mut Table<K, V>, k: K): &mut V {
    field::borrow_mut(&mut table.id, k)
}

/// Removes the key-value pair in the table `table: &mut Table<K, V>` and returns the value.
/// Aborts with `sui::dynamic_field::EFieldDoesNotExist` if the table does not have an entry with
/// that key `k: K`.
public fun remove<K: copy + drop + store, V: store>(table: &mut Table<K, V>, k: K): V {
    let v = field::remove(&mut table.id, k);
    table.size = table.size - 1;
    v
}

/// Returns true iff there is a value associated with the key `k: K` in table `table: &Table<K, V>`
public fun contains<K: copy + drop + store, V: store>(table: &Table<K, V>, k: K): bool {
    field::exists_with_type<K, V>(&table.id, k)
}

/// Returns the size of the table, the number of key-value pairs
public fun length<K: copy + drop + store, V: store>(table: &Table<K, V>): u64 {
    table.size
}

/// Returns true iff the table is empty (if `length` returns `0`)
public fun is_empty<K: copy + drop + store, V: store>(table: &Table<K, V>): bool {
    table.size == 0
}

/// Destroys an empty table
/// Aborts with `ETableNotEmpty` if the table still contains values
public fun destroy_empty<K: copy + drop + store, V: store>(table: Table<K, V>) {
    let Table { id, size } = table;
    assert!(size == 0, ETableNotEmpty);
    id.delete()
}

/// Drop a possibly non-empty table.
/// Usable only if the value type `V` has the `drop` ability
public fun drop<K: copy + drop + store, V: drop + store>(table: Table<K, V>) {
    let Table { id, size: _ } = table;
    id.delete()
}
