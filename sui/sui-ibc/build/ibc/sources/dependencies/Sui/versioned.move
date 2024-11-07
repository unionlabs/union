// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module sui::versioned;

use sui::dynamic_field;

/// Failed to upgrade the inner object due to invalid capability or new version.
const EInvalidUpgrade: u64 = 0;

/// A wrapper type that supports versioning of the inner type.
/// The inner type is a dynamic field of the Versioned object, and is keyed using version.
/// User of this type could load the inner object using corresponding type based on the version.
/// You can also upgrade the inner object to a new type version.
/// If you want to support lazy upgrade of the inner type, one caveat is that all APIs would have
/// to use mutable reference even if it's a read-only API.
public struct Versioned has key, store {
    id: UID,
    version: u64,
}

/// Represents a hot potato object generated when we take out the dynamic field.
/// This is to make sure that we always put a new value back.
public struct VersionChangeCap {
    versioned_id: ID,
    old_version: u64,
}

/// Create a new Versioned object that contains a initial value of type `T` with an initial version.
public fun create<T: store>(init_version: u64, init_value: T, ctx: &mut TxContext): Versioned {
    let mut self = Versioned {
        id: object::new(ctx),
        version: init_version,
    };
    dynamic_field::add(&mut self.id, init_version, init_value);
    self
}

/// Get the current version of the inner type.
public fun version(self: &Versioned): u64 {
    self.version
}

/// Load the inner value based on the current version. Caller specifies an expected type T.
/// If the type mismatch, the load will fail.
public fun load_value<T: store>(self: &Versioned): &T {
    dynamic_field::borrow(&self.id, self.version)
}

/// Similar to load_value, but return a mutable reference.
public fun load_value_mut<T: store>(self: &mut Versioned): &mut T {
    dynamic_field::borrow_mut(&mut self.id, self.version)
}

/// Take the inner object out for upgrade. To ensure we always upgrade properly, a capability object is returned
/// and must be used when we upgrade.
public fun remove_value_for_upgrade<T: store>(self: &mut Versioned): (T, VersionChangeCap) {
    (
        dynamic_field::remove(&mut self.id, self.version),
        VersionChangeCap {
            versioned_id: object::id(self),
            old_version: self.version,
        },
    )
}

/// Upgrade the inner object with a new version and new value. Must use the capability returned
/// by calling remove_value_for_upgrade.
public fun upgrade<T: store>(
    self: &mut Versioned,
    new_version: u64,
    new_value: T,
    cap: VersionChangeCap,
) {
    let VersionChangeCap { versioned_id, old_version } = cap;
    assert!(versioned_id == object::id(self), EInvalidUpgrade);
    assert!(old_version < new_version, EInvalidUpgrade);
    dynamic_field::add(&mut self.id, new_version, new_value);
    self.version = new_version;
}

/// Destroy this Versioned container, and return the inner object.
public fun destroy<T: store>(self: Versioned): T {
    let Versioned { mut id, version } = self;
    let ret = dynamic_field::remove(&mut id, version);
    id.delete();
    ret
}
