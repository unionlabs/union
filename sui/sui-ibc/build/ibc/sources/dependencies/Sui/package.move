// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Functions for operating on Move packages from within Move:
/// - Creating proof-of-publish objects from one-time witnesses
/// - Administering package upgrades through upgrade policies.
module sui::package;

use std::ascii::String;
use std::type_name;
use sui::types;

/// Allows calling `.burn` to destroy a `Publisher`.
public use fun burn_publisher as Publisher.burn;

/// Allows calling `.module_` to access the name of the module a
/// `Publisher` was derived from.
public use fun published_module as Publisher.module_;

/// Allows calling `.package` to access the address of the package
/// a `Publisher` was derived from.
public use fun published_package as Publisher.package;

/// Allows calling `.package` to access the package this cap
/// authorizes upgrades for.
public use fun upgrade_package as UpgradeCap.package;

/// Allows calling `.policy` to access the most permissive kind of
/// upgrade this cap will authorize.
public use fun upgrade_policy as UpgradeCap.policy;

/// Allows calling `.authorize` to initiate an upgrade.
public use fun authorize_upgrade as UpgradeCap.authorize;

/// Allows calling `.commit` to finalize an upgrade.
public use fun commit_upgrade as UpgradeCap.commit;

/// Allows calling `.package` to access the package this ticket
/// authorizes an upgrade for.
public use fun ticket_package as UpgradeTicket.package;

/// Allows calling `.policy` to access the kind of upgrade this
/// ticket authorizes.
public use fun ticket_policy as UpgradeTicket.policy;

/// Allows calling `.digest` to access the digest of the bytecode
/// used for this upgrade.
public use fun ticket_digest as UpgradeTicket.digest;

/// Allows calling `.cap` to fetch the ID of the cap this receipt
/// should be applied to.
public use fun receipt_cap as UpgradeReceipt.cap;

/// Allows calling `.package` to fetch the ID of the package after
/// upgrade.
public use fun receipt_package as UpgradeReceipt.package;

/// Tried to create a `Publisher` using a type that isn't a
/// one-time witness.
const ENotOneTimeWitness: u64 = 0;
/// Tried to set a less restrictive policy than currently in place.
const ETooPermissive: u64 = 1;
/// This `UpgradeCap` has already authorized a pending upgrade.
const EAlreadyAuthorized: u64 = 2;
/// This `UpgradeCap` has not authorized an upgrade.
const ENotAuthorized: u64 = 3;
/// Trying to commit an upgrade to the wrong `UpgradeCap`.
const EWrongUpgradeCap: u64 = 4;

/// Update any part of the package (function implementations, add new
/// functions or types, change dependencies)
const COMPATIBLE: u8 = 0;
/// Add new functions or types, or change dependencies, existing
/// functions can't change.
const ADDITIVE: u8 = 128;
/// Only be able to change dependencies.
const DEP_ONLY: u8 = 192;

/// This type can only be created in the transaction that
/// generates a module, by consuming its one-time witness, so it
/// can be used to identify the address that published the package
/// a type originated from.
public struct Publisher has key, store {
    id: UID,
    package: String,
    module_name: String,
}

/// Capability controlling the ability to upgrade a package.
public struct UpgradeCap has key, store {
    id: UID,
    /// (Mutable) ID of the package that can be upgraded.
    package: ID,
    /// (Mutable) The number of upgrades that have been applied
    /// successively to the original package.  Initially 0.
    version: u64,
    /// What kind of upgrades are allowed.
    policy: u8,
}

/// Permission to perform a particular upgrade (for a fixed version of
/// the package, bytecode to upgrade with and transitive dependencies to
/// depend against).
///
/// An `UpgradeCap` can only issue one ticket at a time, to prevent races
/// between concurrent updates or a change in its upgrade policy after
/// issuing a ticket, so the ticket is a "Hot Potato" to preserve forward
/// progress.
public struct UpgradeTicket {
    /// (Immutable) ID of the `UpgradeCap` this originated from.
    cap: ID,
    /// (Immutable) ID of the package that can be upgraded.
    package: ID,
    /// (Immutable) The policy regarding what kind of upgrade this ticket
    /// permits.
    policy: u8,
    /// (Immutable) SHA256 digest of the bytecode and transitive
    /// dependencies that will be used in the upgrade.
    digest: vector<u8>,
}

/// Issued as a result of a successful upgrade, containing the
/// information to be used to update the `UpgradeCap`.  This is a "Hot
/// Potato" to ensure that it is used to update its `UpgradeCap` before
/// the end of the transaction that performed the upgrade.
public struct UpgradeReceipt {
    /// (Immutable) ID of the `UpgradeCap` this originated from.
    cap: ID,
    /// (Immutable) ID of the package after it was upgraded.
    package: ID,
}

/// Claim a Publisher object.
/// Requires a One-Time-Witness to prove ownership. Due to this
/// constraint there can be only one Publisher object per module
/// but multiple per package (!).
public fun claim<OTW: drop>(otw: OTW, ctx: &mut TxContext): Publisher {
    assert!(types::is_one_time_witness(&otw), ENotOneTimeWitness);

    let tyname = type_name::get_with_original_ids<OTW>();

    Publisher {
        id: object::new(ctx),
        package: tyname.get_address(),
        module_name: tyname.get_module(),
    }
}

#[allow(lint(self_transfer))]
/// Claim a Publisher object and send it to transaction sender.
/// Since this function can only be called in the module initializer,
/// the sender is the publisher.
public fun claim_and_keep<OTW: drop>(otw: OTW, ctx: &mut TxContext) {
    sui::transfer::public_transfer(claim(otw, ctx), ctx.sender())
}

/// Destroy a Publisher object effectively removing all privileges
/// associated with it.
public fun burn_publisher(self: Publisher) {
    let Publisher { id, package: _, module_name: _ } = self;
    id.delete();
}

/// Check whether type belongs to the same package as the publisher object.
public fun from_package<T>(self: &Publisher): bool {
    type_name::get_with_original_ids<T>().get_address() == self.package
}

/// Check whether a type belongs to the same module as the publisher object.
public fun from_module<T>(self: &Publisher): bool {
    let tyname = type_name::get_with_original_ids<T>();

    (tyname.get_address() == self.package) && (tyname.get_module() == self.module_name)
}

/// Read the name of the module.
public fun published_module(self: &Publisher): &String {
    &self.module_name
}

/// Read the package address string.
public fun published_package(self: &Publisher): &String {
    &self.package
}

/// The ID of the package that this cap authorizes upgrades for.
/// Can be `0x0` if the cap cannot currently authorize an upgrade
/// because there is already a pending upgrade in the transaction.
/// Otherwise guaranteed to be the latest version of any given
/// package.
public fun upgrade_package(cap: &UpgradeCap): ID {
    cap.package
}

/// The most recent version of the package, increments by one for each
/// successfully applied upgrade.
public fun version(cap: &UpgradeCap): u64 {
    cap.version
}

/// The most permissive kind of upgrade currently supported by this
/// `cap`.
public fun upgrade_policy(cap: &UpgradeCap): u8 {
    cap.policy
}

/// The package that this ticket is authorized to upgrade
public fun ticket_package(ticket: &UpgradeTicket): ID {
    ticket.package
}

/// The kind of upgrade that this ticket authorizes.
public fun ticket_policy(ticket: &UpgradeTicket): u8 {
    ticket.policy
}

/// ID of the `UpgradeCap` that this `receipt` should be used to
/// update.
public fun receipt_cap(receipt: &UpgradeReceipt): ID {
    receipt.cap
}

/// ID of the package that was upgraded to: the latest version of
/// the package, as of the upgrade represented by this `receipt`.
public fun receipt_package(receipt: &UpgradeReceipt): ID {
    receipt.package
}

/// A hash of the package contents for the new version of the
/// package.  This ticket only authorizes an upgrade to a package
/// that matches this digest.  A package's contents are identified
/// by two things:
///
///  - modules: [[u8]]       a list of the package's module contents
///  - deps:    [[u8; 32]]   a list of 32 byte ObjectIDs of the
///                          package's transitive dependencies
///
/// A package's digest is calculated as:
///
///   sha3_256(sort(modules ++ deps))
public fun ticket_digest(ticket: &UpgradeTicket): &vector<u8> {
    &ticket.digest
}

/// Expose the constants representing various upgrade policies
public fun compatible_policy(): u8 { COMPATIBLE }

public fun additive_policy(): u8 { ADDITIVE }

public fun dep_only_policy(): u8 { DEP_ONLY }

/// Restrict upgrades through this upgrade `cap` to just add code, or
/// change dependencies.
public entry fun only_additive_upgrades(cap: &mut UpgradeCap) {
    cap.restrict(ADDITIVE)
}

/// Restrict upgrades through this upgrade `cap` to just change
/// dependencies.
public entry fun only_dep_upgrades(cap: &mut UpgradeCap) {
    cap.restrict(DEP_ONLY)
}

/// Discard the `UpgradeCap` to make a package immutable.
public entry fun make_immutable(cap: UpgradeCap) {
    let UpgradeCap { id, package: _, version: _, policy: _ } = cap;
    id.delete();
}

/// Issue a ticket authorizing an upgrade to a particular new bytecode
/// (identified by its digest).  A ticket will only be issued if one has
/// not already been issued, and if the `policy` requested is at least as
/// restrictive as the policy set out by the `cap`.
///
/// The `digest` supplied and the `policy` will both be checked by
/// validators when running the upgrade.  I.e. the bytecode supplied in
/// the upgrade must have a matching digest, and the changes relative to
/// the parent package must be compatible with the policy in the ticket
/// for the upgrade to succeed.
public fun authorize_upgrade(cap: &mut UpgradeCap, policy: u8, digest: vector<u8>): UpgradeTicket {
    let id_zero = @0x0.to_id();
    assert!(cap.package != id_zero, EAlreadyAuthorized);
    assert!(policy >= cap.policy, ETooPermissive);

    let package = cap.package;
    cap.package = id_zero;

    UpgradeTicket {
        cap: object::id(cap),
        package,
        policy,
        digest,
    }
}

/// Consume an `UpgradeReceipt` to update its `UpgradeCap`, finalizing
/// the upgrade.
public fun commit_upgrade(cap: &mut UpgradeCap, receipt: UpgradeReceipt) {
    let UpgradeReceipt { cap: cap_id, package } = receipt;

    assert!(object::id(cap) == cap_id, EWrongUpgradeCap);
    assert!(cap.package.to_address() == @0x0, ENotAuthorized);

    cap.package = package;
    cap.version = cap.version + 1;
}

#[test_only]
/// Test-only function to claim a Publisher object bypassing OTW check.
public fun test_claim<OTW: drop>(_: OTW, ctx: &mut TxContext): Publisher {
    let tyname = type_name::get_with_original_ids<OTW>();

    Publisher {
        id: object::new(ctx),
        package: tyname.get_address(),
        module_name: tyname.get_module(),
    }
}

#[test_only]
/// Test-only function to simulate publishing a package at address
/// `ID`, to create an `UpgradeCap`.
public fun test_publish(package: ID, ctx: &mut TxContext): UpgradeCap {
    UpgradeCap {
        id: object::new(ctx),
        package,
        version: 1,
        policy: COMPATIBLE,
    }
}

#[test_only]
/// Test-only function that takes the role of the actual `Upgrade`
/// command, converting the ticket for the pending upgrade to a
/// receipt for a completed upgrade.
public fun test_upgrade(ticket: UpgradeTicket): UpgradeReceipt {
    let UpgradeTicket { cap, package, policy: _, digest: _ } = ticket;

    // Generate a fake package ID for the upgraded package by
    // hashing the existing package and cap ID.
    let mut data = cap.to_bytes();
    data.append(package.to_bytes());
    let package = object::id_from_bytes(sui::hash::blake2b256(&data));

    UpgradeReceipt {
        cap,
        package,
    }
}

fun restrict(cap: &mut UpgradeCap, policy: u8) {
    assert!(cap.policy <= policy, ETooPermissive);
    cap.policy = policy;
}
