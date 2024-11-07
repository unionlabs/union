// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Defines the `DenyList` type. The `DenyList` shared object is used to restrict access to
/// instances of certain core types from being used as inputs by specified addresses in the deny
/// list.
module sui::deny_list;

use sui::bag::{Self, Bag};
use sui::config::{Self, Config};
use sui::dynamic_object_field as ofield;
use sui::table::{Self, Table};
use sui::vec_set::{Self, VecSet};

/// Trying to create a deny list object when not called by the system address.
const ENotSystemAddress: u64 = 0;
/// The specified address to be removed is not already in the deny list.
const ENotDenied: u64 = 1;
/// The specified address cannot be added to the deny list.
const EInvalidAddress: u64 = 1;

/// The index into the deny list vector for the `sui::coin::Coin` type.
const COIN_INDEX: u64 = 0;

/// These addresses are reserved and cannot be added to the deny list.
/// The addresses listed are well known package and object addresses. So it would be
/// meaningless to add them to the deny list.
const RESERVED: vector<address> = vector[
    @0x0,
    @0x1,
    @0x2,
    @0x3,
    @0x4,
    @0x5,
    @0x6,
    @0x7,
    @0x8,
    @0x9,
    @0xA,
    @0xB,
    @0xC,
    @0xD,
    @0xE,
    @0xF,
    @0x403,
    @0xDEE9,
];

/// A shared object that stores the addresses that are blocked for a given core type.
public struct DenyList has key {
    id: UID,
    /// The individual deny lists.
    lists: Bag,
}

// === V2 ===

/// The capability used to write to the deny list config. Ensures that the Configs for the
/// DenyList are modified only by this module.
public struct ConfigWriteCap() has drop;

/// The dynamic object field key used to store the `Config` for a given type, essentially a
/// `(per_type_index, per_type_key)` pair.
public struct ConfigKey has copy, drop, store {
    per_type_index: u64,
    per_type_key: vector<u8>,
}

/// The setting key used to store the deny list for a given address in the `Config`.
public struct AddressKey(address) has copy, drop, store;

/// The setting key used to store the global pause setting in the `Config`.
public struct GlobalPauseKey() has copy, drop, store;

/// The event emitted when a new `Config` is created for a given type. This can be useful for
/// tracking the `ID` of a type's `Config` object.
public struct PerTypeConfigCreated has copy, drop, store {
    key: ConfigKey,
    config_id: ID,
}

public(package) fun v2_add(
    deny_list: &mut DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
    addr: address,
    ctx: &mut TxContext,
) {
    let per_type_config = deny_list.per_type_config_entry!(per_type_index, per_type_key, ctx);
    let setting_name = AddressKey(addr);
    let next_epoch_entry = per_type_config.entry!<_, AddressKey, bool>(
        &mut ConfigWriteCap(),
        setting_name,
        |_deny_list, _cap, _ctx| true,
        ctx,
    );
    *next_epoch_entry = true;
}

public(package) fun v2_remove(
    deny_list: &mut DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
    addr: address,
    ctx: &mut TxContext,
) {
    let per_type_config = deny_list.per_type_config_entry!(per_type_index, per_type_key, ctx);
    let setting_name = AddressKey(addr);
    per_type_config.remove_for_next_epoch<_, AddressKey, bool>(
        &mut ConfigWriteCap(),
        setting_name,
        ctx,
    );
}

public(package) fun v2_contains_current_epoch(
    deny_list: &DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
    addr: address,
    ctx: &TxContext,
): bool {
    if (!deny_list.per_type_exists(per_type_index, per_type_key)) return false;
    let per_type_config = deny_list.borrow_per_type_config(per_type_index, per_type_key);
    let setting_name = AddressKey(addr);
    config::read_setting(object::id(per_type_config), setting_name, ctx).destroy_or!(false)
}

public(package) fun v2_contains_next_epoch(
    deny_list: &DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
    addr: address,
): bool {
    if (!deny_list.per_type_exists(per_type_index, per_type_key)) return false;
    let per_type_config = deny_list.borrow_per_type_config(per_type_index, per_type_key);
    let setting_name = AddressKey(addr);
    per_type_config.read_setting_for_next_epoch(setting_name).destroy_or!(false)
}

// public(package) fun v2_per_type_contains(
//     per_type_config: ID,
//     addr: address,
// ): bool {
//    // TODO can read from the config directly once the ID is set
// }

public(package) fun v2_enable_global_pause(
    deny_list: &mut DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
    ctx: &mut TxContext,
) {
    let per_type_config = deny_list.per_type_config_entry!(per_type_index, per_type_key, ctx);
    let setting_name = GlobalPauseKey();
    let next_epoch_entry = per_type_config.entry!<_, GlobalPauseKey, bool>(
        &mut ConfigWriteCap(),
        setting_name,
        |_deny_list, _cap, _ctx| true,
        ctx,
    );
    *next_epoch_entry = true;
}

public(package) fun v2_disable_global_pause(
    deny_list: &mut DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
    ctx: &mut TxContext,
) {
    let per_type_config = deny_list.per_type_config_entry!(per_type_index, per_type_key, ctx);
    let setting_name = GlobalPauseKey();
    per_type_config.remove_for_next_epoch<_, GlobalPauseKey, bool>(
        &mut ConfigWriteCap(),
        setting_name,
        ctx,
    );
}

public(package) fun v2_is_global_pause_enabled_current_epoch(
    deny_list: &DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
    ctx: &TxContext,
): bool {
    if (!deny_list.per_type_exists(per_type_index, per_type_key)) return false;
    let per_type_config = deny_list.borrow_per_type_config(per_type_index, per_type_key);
    let setting_name = GlobalPauseKey();
    config::read_setting(object::id(per_type_config), setting_name, ctx).destroy_or!(false)
}

public(package) fun v2_is_global_pause_enabled_next_epoch(
    deny_list: &DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
): bool {
    if (!deny_list.per_type_exists(per_type_index, per_type_key)) return false;
    let per_type_config = deny_list.borrow_per_type_config(per_type_index, per_type_key);
    let setting_name = GlobalPauseKey();
    per_type_config.read_setting_for_next_epoch(setting_name).destroy_or!(false)
}

// public(package) fun v2_per_type_is_global_pause_enabled(
//     per_type_config: ID,
// ): bool {
//    // TODO can read from the config directly once the ID is set
// }

public(package) fun migrate_v1_to_v2(
    deny_list: &mut DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
    ctx: &mut TxContext,
) {
    let bag_entry: &mut PerTypeList = &mut deny_list.lists[per_type_index];
    let elements = if (!bag_entry.denied_addresses.contains(per_type_key)) vector[]
    else bag_entry.denied_addresses.remove(per_type_key).into_keys();
    elements.do_ref!(|addr| {
        let addr = *addr;
        let denied_count = &mut bag_entry.denied_count[addr];
        *denied_count = *denied_count - 1;
        if (*denied_count == 0) {
            bag_entry.denied_count.remove(addr);
        }
    });
    let per_type_config = deny_list.per_type_config_entry!(per_type_index, per_type_key, ctx);
    elements.do!(|addr| {
        let setting_name = AddressKey(addr);
        let next_epoch_entry = per_type_config.entry!<_, AddressKey, bool>(
            &mut ConfigWriteCap(),
            setting_name,
            |_deny_list, _cap, _ctx| true,
            ctx,
        );
        *next_epoch_entry = true;
    });
}

fun add_per_type_config(
    deny_list: &mut DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
    ctx: &mut TxContext,
) {
    let key = ConfigKey { per_type_index, per_type_key };
    let config = config::new(&mut ConfigWriteCap(), ctx);
    let config_id = object::id(&config);
    ofield::internal_add(&mut deny_list.id, key, config);
    sui::event::emit(PerTypeConfigCreated { key, config_id });
}

fun borrow_per_type_config_mut(
    deny_list: &mut DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
): &mut Config<ConfigWriteCap> {
    let key = ConfigKey { per_type_index, per_type_key };
    ofield::internal_borrow_mut(&mut deny_list.id, key)
}

fun borrow_per_type_config(
    deny_list: &DenyList,
    per_type_index: u64,
    per_type_key: vector<u8>,
): &Config<ConfigWriteCap> {
    let key = ConfigKey { per_type_index, per_type_key };
    ofield::internal_borrow(&deny_list.id, key)
}

fun per_type_exists(deny_list: &DenyList, per_type_index: u64, per_type_key: vector<u8>): bool {
    let key = ConfigKey { per_type_index, per_type_key };
    ofield::exists_(&deny_list.id, key)
}

macro fun per_type_config_entry(
    $deny_list: &mut DenyList,
    $per_type_index: u64,
    $per_type_key: vector<u8>,
    $ctx: &mut TxContext,
): &mut Config<ConfigWriteCap> {
    let deny_list = $deny_list;
    let per_type_index = $per_type_index;
    let per_type_key = $per_type_key;
    let ctx = $ctx;
    if (!deny_list.per_type_exists(per_type_index, per_type_key)) {
        deny_list.add_per_type_config(per_type_index, per_type_key, ctx);
    };
    deny_list.borrow_per_type_config_mut(per_type_index, per_type_key)
}

// === V1 ===

/// Stores the addresses that are denied for a given core type.
public struct PerTypeList has key, store {
    id: UID,
    /// Number of object types that have been banned for a given address.
    /// Used to quickly skip checks for most addresses.
    denied_count: Table<address, u64>,
    /// Set of addresses that are banned for a given type.
    /// For example with `sui::coin::Coin`: If addresses A and B are banned from using
    /// "0...0123::my_coin::MY_COIN", this will be "0...0123::my_coin::MY_COIN" -> {A, B}.
    denied_addresses: Table<vector<u8>, VecSet<address>>,
}

/// Adds the given address to the deny list of the specified type, preventing it
/// from interacting with instances of that type as an input to a transaction. For coins,
/// the type specified is the type of the coin, not the coin type itself. For example,
/// "00...0123::my_coin::MY_COIN" would be the type, not "00...02::coin::Coin".
public(package) fun v1_add(
    deny_list: &mut DenyList,
    per_type_index: u64,
    `type`: vector<u8>,
    addr: address,
) {
    let reserved = RESERVED;
    assert!(!reserved.contains(&addr), EInvalidAddress);
    let bag_entry: &mut PerTypeList = &mut deny_list.lists[per_type_index];
    bag_entry.v1_per_type_list_add(`type`, addr)
}

fun v1_per_type_list_add(list: &mut PerTypeList, `type`: vector<u8>, addr: address) {
    if (!list.denied_addresses.contains(`type`)) {
        list.denied_addresses.add(`type`, vec_set::empty());
    };
    let denied_addresses = &mut list.denied_addresses[`type`];
    let already_denied = denied_addresses.contains(&addr);
    if (already_denied) return;

    denied_addresses.insert(addr);
    if (!list.denied_count.contains(addr)) {
        list.denied_count.add(addr, 0);
    };
    let denied_count = &mut list.denied_count[addr];
    *denied_count = *denied_count + 1;
}

/// Removes a previously denied address from the list.
/// Aborts with `ENotDenied` if the address is not on the list.
public(package) fun v1_remove(
    deny_list: &mut DenyList,
    per_type_index: u64,
    `type`: vector<u8>,
    addr: address,
) {
    let reserved = RESERVED;
    assert!(!reserved.contains(&addr), EInvalidAddress);
    let bag_entry: &mut PerTypeList = &mut deny_list.lists[per_type_index];
    bag_entry.v1_per_type_list_remove(`type`, addr)
}

fun v1_per_type_list_remove(list: &mut PerTypeList, `type`: vector<u8>, addr: address) {
    let denied_addresses = &mut list.denied_addresses[`type`];
    assert!(denied_addresses.contains(&addr), ENotDenied);
    denied_addresses.remove(&addr);
    let denied_count = &mut list.denied_count[addr];
    *denied_count = *denied_count - 1;
    if (*denied_count == 0) {
        list.denied_count.remove(addr);
    }
}

/// Returns true iff the given address is denied for the given type.
public(package) fun v1_contains(
    deny_list: &DenyList,
    per_type_index: u64,
    `type`: vector<u8>,
    addr: address,
): bool {
    let reserved = RESERVED;
    if (reserved.contains(&addr)) return false;
    let bag_entry: &PerTypeList = &deny_list.lists[per_type_index];
    bag_entry.v1_per_type_list_contains(`type`, addr)
}

fun v1_per_type_list_contains(list: &PerTypeList, `type`: vector<u8>, addr: address): bool {
    if (!list.denied_count.contains(addr)) return false;

    let denied_count = &list.denied_count[addr];
    if (*denied_count == 0) return false;

    if (!list.denied_addresses.contains(`type`)) return false;

    let denied_addresses = &list.denied_addresses[`type`];
    denied_addresses.contains(&addr)
}

#[allow(unused_function)]
/// Creation of the deny list object is restricted to the system address
/// via a system transaction.
fun create(ctx: &mut TxContext) {
    assert!(ctx.sender() == @0x0, ENotSystemAddress);

    let mut lists = bag::new(ctx);
    lists.add(COIN_INDEX, per_type_list(ctx));
    let deny_list_object = DenyList {
        id: object::sui_deny_list_object_id(),
        lists,
    };
    transfer::share_object(deny_list_object);
}

fun per_type_list(ctx: &mut TxContext): PerTypeList {
    PerTypeList {
        id: object::new(ctx),
        denied_count: table::new(ctx),
        denied_addresses: table::new(ctx),
    }
}

#[test_only]
public fun reserved_addresses(): vector<address> {
    RESERVED
}

#[test_only]
public fun create_for_test(ctx: &mut TxContext) {
    create(ctx);
}

#[test_only]
/// Creates and returns a new DenyList object for testing purposes. It
/// doesn't matter which object ID the list has in this kind of test.
public fun new_for_testing(ctx: &mut TxContext): DenyList {
    let mut lists = bag::new(ctx);
    lists.add(COIN_INDEX, per_type_list(ctx));
    DenyList {
        id: object::new(ctx),
        lists,
    }
}
