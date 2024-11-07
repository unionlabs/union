// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// This module implements the Kiosk Extensions functionality. It allows
/// exposing previously protected (only-owner) methods to third-party apps.
///
/// A Kiosk Extension is a module that implements any functionality on top of
/// the `Kiosk` without discarding nor blocking the base. Given that `Kiosk`
/// itself is a trading primitive, most of the extensions are expected to be
/// related to trading. However, there's no limit to what can be built using the
/// `kiosk_extension` module, as it gives certain benefits such as using `Kiosk`
/// as the storage for any type of data / assets.
///
/// ### Flow:
/// - An extension can only be installed by the Kiosk Owner and requires an
/// authorization via the `KioskOwnerCap`.
/// - When installed, the extension is given a permission bitmap that allows it
/// to perform certain protected actions (eg `place`, `lock`). However, it is
/// possible to install an extension that does not have any permissions.
/// - Kiosk Owner can `disable` the extension at any time, which prevents it
/// from performing any protected actions. The storage is still available to the
/// extension until it is completely removed.
/// - A disabled extension can be `enable`d at any time giving the permissions
/// back to the extension.
/// - An extension permissions follow the all-or-nothing policy. Either all of
/// the requested permissions are granted or none of them (can't install).
///
/// ### Examples:
/// - An Auction extension can utilize the storage to store Auction-related data
/// while utilizing the same `Kiosk` object that the items are stored in.
/// - A Marketplace extension that implements custom events and fees for the
/// default trading functionality.
///
/// ### Notes:
/// - Trading functionality can utilize the `PurchaseCap` to build a custom
/// logic around the purchase flow. However, it should be carefully managed to
/// prevent asset locking.
/// - `kiosk_extension` is a friend module to `kiosk` and has access to its
/// internal functions (such as `place_internal` and `lock_internal` to
/// implement custom authorization scheme for `place` and `lock` respectively).
module sui::kiosk_extension;

use sui::bag::{Self, Bag};
use sui::dynamic_field as df;
use sui::kiosk::{Kiosk, KioskOwnerCap};
use sui::transfer_policy::TransferPolicy;

/// Trying to add an extension while not being the owner of the Kiosk.
const ENotOwner: u64 = 0;
/// Extension is trying to access a permissioned action while not having
/// the required permission.
const EExtensionNotAllowed: u64 = 2;
/// Extension is not installed in the Kiosk.
const EExtensionNotInstalled: u64 = 3;

/// Value that represents the `place` permission in the permissions bitmap.
const PLACE: u128 = 1;

/// Value that represents the `lock` and `place` permission in the
/// permissions bitmap.
const LOCK: u128 = 2;

/// The Extension struct contains the data used by the extension and the
/// configuration for this extension. Stored under the `ExtensionKey`
/// dynamic field.
public struct Extension has store {
    /// Storage for the extension, an isolated Bag. By putting the extension
    /// into a single dynamic field, we reduce the amount of fields on the
    /// top level (eg items / listings) while giving extension developers
    /// the ability to store any data they want.
    storage: Bag,
    /// Bitmap of permissions that the extension has (can be revoked any
    /// moment). It's all or nothing policy - either the extension has the
    /// required permissions or no permissions at all.
    ///
    /// 1st bit - `place` - allows to place items for sale
    /// 2nd bit - `lock` and `place` - allows to lock items (and place)
    ///
    /// For example:
    /// - `10` - allows to place items and lock them.
    /// - `11` - allows to place items and lock them (`lock` includes `place`).
    /// - `01` - allows to place items, but not lock them.
    /// - `00` - no permissions.
    permissions: u128,
    /// Whether the extension can call protected actions. By default, all
    /// extensions are enabled (on `add` call), however the Kiosk
    /// owner can disable them at any time.
    ///
    /// Disabling the extension does not limit its access to the storage.
    is_enabled: bool,
}

/// The `ExtensionKey` is a typed dynamic field key used to store the
/// extension configuration and data. `Ext` is a phantom type that is used
/// to identify the extension witness.
public struct ExtensionKey<phantom Ext> has store, copy, drop {}

// === Management ===

/// Add an extension to the Kiosk. Can only be performed by the owner. The
/// extension witness is required to allow extensions define their set of
/// permissions in the custom `add` call.
public fun add<Ext: drop>(
    _ext: Ext,
    self: &mut Kiosk,
    cap: &KioskOwnerCap,
    permissions: u128,
    ctx: &mut TxContext,
) {
    assert!(self.has_access(cap), ENotOwner);
    df::add(
        self.uid_mut_as_owner(cap),
        ExtensionKey<Ext> {},
        Extension {
            storage: bag::new(ctx),
            permissions,
            is_enabled: true,
        },
    )
}

/// Revoke permissions from the extension. While it does not remove the
/// extension completely, it keeps it from performing any protected actions.
/// The storage is still available to the extension (until it's removed).
public fun disable<Ext: drop>(self: &mut Kiosk, cap: &KioskOwnerCap) {
    assert!(self.has_access(cap), ENotOwner);
    assert!(is_installed<Ext>(self), EExtensionNotInstalled);
    extension_mut<Ext>(self).is_enabled = false;
}

/// Re-enable the extension allowing it to call protected actions (eg
/// `place`, `lock`). By default, all added extensions are enabled. Kiosk
/// owner can disable them via `disable` call.
public fun enable<Ext: drop>(self: &mut Kiosk, cap: &KioskOwnerCap) {
    assert!(self.has_access(cap), ENotOwner);
    assert!(is_installed<Ext>(self), EExtensionNotInstalled);
    extension_mut<Ext>(self).is_enabled = true;
}

/// Remove an extension from the Kiosk. Can only be performed by the owner,
/// the extension storage must be empty for the transaction to succeed.
public fun remove<Ext: drop>(self: &mut Kiosk, cap: &KioskOwnerCap) {
    assert!(self.has_access(cap), ENotOwner);
    assert!(is_installed<Ext>(self), EExtensionNotInstalled);

    let Extension {
        storage,
        permissions: _,
        is_enabled: _,
    } = df::remove(self.uid_mut_as_owner(cap), ExtensionKey<Ext> {});

    storage.destroy_empty();
}

// === Storage ===

/// Get immutable access to the extension storage. Can only be performed by
/// the extension as long as the extension is installed.
public fun storage<Ext: drop>(_ext: Ext, self: &Kiosk): &Bag {
    assert!(is_installed<Ext>(self), EExtensionNotInstalled);
    &extension<Ext>(self).storage
}

/// Get mutable access to the extension storage. Can only be performed by
/// the extension as long as the extension is installed. Disabling the
/// extension does not prevent it from accessing the storage.
///
/// Potentially dangerous: extension developer can keep data in a Bag
/// therefore never really allowing the KioskOwner to remove the extension.
/// However, it is the case with any other solution (1) and this way we
/// prevent intentional extension freeze when the owner wants to ruin a
/// trade (2) - eg locking extension while an auction is in progress.
///
/// Extensions should be crafted carefully, and the KioskOwner should be
/// aware of the risks.
public fun storage_mut<Ext: drop>(_ext: Ext, self: &mut Kiosk): &mut Bag {
    assert!(is_installed<Ext>(self), EExtensionNotInstalled);
    &mut extension_mut<Ext>(self).storage
}

// === Protected Actions ===

/// Protected action: place an item into the Kiosk. Can be performed by an
/// authorized extension. The extension must have the `place` permission or
/// a `lock` permission.
///
/// To prevent non-tradable items from being placed into `Kiosk` the method
/// requires a `TransferPolicy` for the placed type to exist.
public fun place<Ext: drop, T: key + store>(
    _ext: Ext,
    self: &mut Kiosk,
    item: T,
    _policy: &TransferPolicy<T>,
) {
    assert!(is_installed<Ext>(self), EExtensionNotInstalled);
    assert!(can_place<Ext>(self) || can_lock<Ext>(self), EExtensionNotAllowed);

    self.place_internal(item)
}

/// Protected action: lock an item in the Kiosk. Can be performed by an
/// authorized extension. The extension must have the `lock` permission.
public fun lock<Ext: drop, T: key + store>(
    _ext: Ext,
    self: &mut Kiosk,
    item: T,
    _policy: &TransferPolicy<T>,
) {
    assert!(is_installed<Ext>(self), EExtensionNotInstalled);
    assert!(can_lock<Ext>(self), EExtensionNotAllowed);

    self.lock_internal(item)
}

// === Field Access ===

/// Check whether an extension of type `Ext` is installed.
public fun is_installed<Ext: drop>(self: &Kiosk): bool {
    df::exists_(self.uid(), ExtensionKey<Ext> {})
}

/// Check whether an extension of type `Ext` is enabled.
public fun is_enabled<Ext: drop>(self: &Kiosk): bool {
    extension<Ext>(self).is_enabled
}

/// Check whether an extension of type `Ext` can `place` into Kiosk.
public fun can_place<Ext: drop>(self: &Kiosk): bool {
    is_enabled<Ext>(self) && extension<Ext>(self).permissions & PLACE != 0
}

/// Check whether an extension of type `Ext` can `lock` items in Kiosk.
/// Locking also enables `place`.
public fun can_lock<Ext: drop>(self: &Kiosk): bool {
    is_enabled<Ext>(self) && extension<Ext>(self).permissions & LOCK != 0
}

// === Internal ===

/// Internal: get a read-only access to the Extension.
fun extension<Ext: drop>(self: &Kiosk): &Extension {
    df::borrow(self.uid(), ExtensionKey<Ext> {})
}

/// Internal: get a mutable access to the Extension.
fun extension_mut<Ext: drop>(self: &mut Kiosk): &mut Extension {
    df::borrow_mut(self.uid_mut_internal(), ExtensionKey<Ext> {})
}
