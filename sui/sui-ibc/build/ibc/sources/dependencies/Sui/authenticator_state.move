// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_use)]
// Module for storing authenticator state, which is currently just the set of valid JWKs used by
// zklogin.
//
// This module is not currently accessible from user contracts, and is used only to record the JWK
// state to the chain for auditability + restore from snapshot purposes.
module sui::authenticator_state;

use std::string::{Self, String, utf8};
use sui::dynamic_field;

/// Sender is not @0x0 the system address.
const ENotSystemAddress: u64 = 0;
const EWrongInnerVersion: u64 = 1;
const EJwksNotSorted: u64 = 2;

const CurrentVersion: u64 = 1;

/// Singleton shared object which stores the global authenticator state.
/// The actual state is stored in a dynamic field of type AuthenticatorStateInner to support
/// future versions of the authenticator state.
public struct AuthenticatorState has key {
    id: UID,
    version: u64,
}

public struct AuthenticatorStateInner has store {
    version: u64,
    /// List of currently active JWKs.
    active_jwks: vector<ActiveJwk>,
}

#[allow(unused_field)]
/// Must match the JWK struct in fastcrypto-zkp
public struct JWK has store, drop, copy {
    kty: String,
    e: String,
    n: String,
    alg: String,
}

#[allow(unused_field)]
/// Must match the JwkId struct in fastcrypto-zkp
public struct JwkId has store, drop, copy {
    iss: String,
    kid: String,
}

#[allow(unused_field)]
public struct ActiveJwk has store, drop, copy {
    jwk_id: JwkId,
    jwk: JWK,
    epoch: u64,
}

#[test_only]
public fun create_active_jwk(iss: String, kid: String, kty: String, epoch: u64): ActiveJwk {
    ActiveJwk {
        jwk_id: JwkId {
            iss: iss,
            kid: kid,
        },
        jwk: JWK {
            kty: kty,
            e: utf8(b"AQAB"),
            n: utf8(b"test"),
            alg: utf8(b"RS256"),
        },
        epoch,
    }
}

fun active_jwk_equal(a: &ActiveJwk, b: &ActiveJwk): bool {
    // note: epoch is ignored
    jwk_equal(&a.jwk, &b.jwk) && jwk_id_equal(&a.jwk_id, &b.jwk_id)
}

fun jwk_equal(a: &JWK, b: &JWK): bool {
    (&a.kty == &b.kty) &&
        (&a.e == &b.e) &&
        (&a.n == &b.n) &&
        (&a.alg == &b.alg)
}

fun jwk_id_equal(a: &JwkId, b: &JwkId): bool {
    (&a.iss == &b.iss) && (&a.kid == &b.kid)
}

// Compare the underlying byte arrays lexicographically. Since the strings may be utf8 this
// ordering is not necessarily the same as the string ordering, but we just need some
// canonical that is cheap to compute.
fun string_bytes_lt(a: &String, b: &String): bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    if (a_bytes.length() < b_bytes.length()) {
        true
    } else if (a_bytes.length() > b_bytes.length()) {
        false
    } else {
        let mut i = 0;
        while (i < a_bytes.length()) {
            let a_byte = a_bytes[i];
            let b_byte = b_bytes[i];
            if (a_byte < b_byte) {
                return true
            } else if (a_byte > b_byte) {
                return false
            };
            i = i + 1;
        };
        // all bytes are equal
        false
    }
}

fun jwk_lt(a: &ActiveJwk, b: &ActiveJwk): bool {
    // note: epoch is ignored
    if (&a.jwk_id.iss != &b.jwk_id.iss) {
        return string_bytes_lt(&a.jwk_id.iss, &b.jwk_id.iss)
    };
    if (&a.jwk_id.kid != &b.jwk_id.kid) {
        return string_bytes_lt(&a.jwk_id.kid, &b.jwk_id.kid)
    };
    if (&a.jwk.kty != &b.jwk.kty) {
        return string_bytes_lt(&a.jwk.kty, &b.jwk.kty)
    };
    if (&a.jwk.e != &b.jwk.e) {
        return string_bytes_lt(&a.jwk.e, &b.jwk.e)
    };
    if (&a.jwk.n != &b.jwk.n) {
        return string_bytes_lt(&a.jwk.n, &b.jwk.n)
    };
    string_bytes_lt(&a.jwk.alg, &b.jwk.alg)
}

#[allow(unused_function)]
/// Create and share the AuthenticatorState object. This function is call exactly once, when
/// the authenticator state object is first created.
/// Can only be called by genesis or change_epoch transactions.
fun create(ctx: &TxContext) {
    assert!(ctx.sender() == @0x0, ENotSystemAddress);

    let version = CurrentVersion;

    let inner = AuthenticatorStateInner {
        version,
        active_jwks: vector[],
    };

    let mut self = AuthenticatorState {
        id: object::authenticator_state(),
        version,
    };

    dynamic_field::add(&mut self.id, version, inner);
    transfer::share_object(self);
}

fun load_inner_mut(self: &mut AuthenticatorState): &mut AuthenticatorStateInner {
    let version = self.version;

    // replace this with a lazy update function when we add a new version of the inner object.
    assert!(version == CurrentVersion, EWrongInnerVersion);

    let inner: &mut AuthenticatorStateInner = dynamic_field::borrow_mut(&mut self.id, self.version);

    assert!(inner.version == version, EWrongInnerVersion);
    inner
}

fun load_inner(self: &AuthenticatorState): &AuthenticatorStateInner {
    let version = self.version;

    // replace this with a lazy update function when we add a new version of the inner object.
    assert!(version == CurrentVersion, EWrongInnerVersion);

    let inner: &AuthenticatorStateInner = dynamic_field::borrow(&self.id, self.version);

    assert!(inner.version == version, EWrongInnerVersion);
    inner
}

fun check_sorted(new_active_jwks: &vector<ActiveJwk>) {
    let mut i = 0;
    while (i < new_active_jwks.length() - 1) {
        let a = &new_active_jwks[i];
        let b = &new_active_jwks[i + 1];
        assert!(jwk_lt(a, b), EJwksNotSorted);
        i = i + 1;
    };
}

#[allow(unused_function)]
/// Record a new set of active_jwks. Called when executing the AuthenticatorStateUpdate system
/// transaction. The new input vector must be sorted and must not contain duplicates.
/// If a new JWK is already present, but with a previous epoch, then the epoch is updated to
/// indicate that the JWK has been validated in the current epoch and should not be expired.
fun update_authenticator_state(
    self: &mut AuthenticatorState,
    new_active_jwks: vector<ActiveJwk>,
    ctx: &TxContext,
) {
    // Validator will make a special system call with sender set as 0x0.
    assert!(ctx.sender() == @0x0, ENotSystemAddress);

    check_sorted(&new_active_jwks);
    let new_active_jwks = deduplicate(new_active_jwks);

    let inner = self.load_inner_mut();

    let mut res = vector[];
    let mut i = 0;
    let mut j = 0;
    let active_jwks_len = inner.active_jwks.length();
    let new_active_jwks_len = new_active_jwks.length();

    while (i < active_jwks_len && j < new_active_jwks_len) {
        let old_jwk = &inner.active_jwks[i];
        let new_jwk = &new_active_jwks[j];

        // when they are equal, push only one, but use the max epoch of the two
        if (active_jwk_equal(old_jwk, new_jwk)) {
            let mut jwk = *old_jwk;
            jwk.epoch = old_jwk.epoch.max(new_jwk.epoch);
            res.push_back(jwk);
            i = i + 1;
            j = j + 1;
        } else if (jwk_id_equal(&old_jwk.jwk_id, &new_jwk.jwk_id)) {
            // if only jwk_id is equal, then the key has changed. Providers should not send
            // JWKs like this, but if they do, we must ignore the new JWK to avoid having a
            // liveness / forking issues
            res.push_back(*old_jwk);
            i = i + 1;
            j = j + 1;
        } else if (jwk_lt(old_jwk, new_jwk)) {
            res.push_back(*old_jwk);
            i = i + 1;
        } else {
            res.push_back(*new_jwk);
            j = j + 1;
        }
    };

    while (i < active_jwks_len) {
        res.push_back(inner.active_jwks[i]);
        i = i + 1;
    };
    while (j < new_active_jwks_len) {
        res.push_back(new_active_jwks[j]);
        j = j + 1;
    };

    inner.active_jwks = res;
}

fun deduplicate(jwks: vector<ActiveJwk>): vector<ActiveJwk> {
    let mut res = vector[];
    let mut i = 0;
    let mut prev: Option<JwkId> = option::none();
    while (i < jwks.length()) {
        let jwk = &jwks[i];
        if (prev.is_none()) {
            prev.fill(jwk.jwk_id);
        } else if (jwk_id_equal(prev.borrow(), &jwk.jwk_id)) {
            // skip duplicate jwks in input
            i = i + 1;
            continue
        } else {
            *prev.borrow_mut() = jwk.jwk_id;
        };
        res.push_back(*jwk);
        i = i + 1;
    };
    res
}

#[allow(unused_function)]
// Called directly by rust when constructing the ChangeEpoch transaction.
fun expire_jwks(
    self: &mut AuthenticatorState,
    // any jwk below this epoch is not retained
    min_epoch: u64,
    ctx: &TxContext,
) {
    // This will only be called by sui_system::advance_epoch
    assert!(ctx.sender() == @0x0, ENotSystemAddress);

    let inner = load_inner_mut(self);

    let len = inner.active_jwks.length();

    // first we count how many jwks from each issuer are above the min_epoch
    // and store the counts in a vector that parallels the (sorted) active_jwks vector
    let mut issuer_max_epochs = vector[];
    let mut i = 0;
    let mut prev_issuer: Option<String> = option::none();

    while (i < len) {
        let cur = &inner.active_jwks[i];
        let cur_iss = &cur.jwk_id.iss;
        if (prev_issuer.is_none()) {
            prev_issuer.fill(*cur_iss);
            issuer_max_epochs.push_back(cur.epoch);
        } else {
            if (cur_iss == prev_issuer.borrow()) {
                let back = issuer_max_epochs.length() - 1;
                let prev_max_epoch = &mut issuer_max_epochs[back];
                *prev_max_epoch = (*prev_max_epoch).max(cur.epoch);
            } else {
                *prev_issuer.borrow_mut() = *cur_iss;
                issuer_max_epochs.push_back(cur.epoch);
            }
        };
        i = i + 1;
    };

    // Now, filter out any JWKs that are below the min_epoch, unless that issuer has no
    // JWKs >= the min_epoch, in which case we keep all of them.
    let mut new_active_jwks: vector<ActiveJwk> = vector[];
    let mut prev_issuer: Option<String> = option::none();
    let mut i = 0;
    let mut j = 0;
    while (i < len) {
        let jwk = &inner.active_jwks[i];
        let cur_iss = &jwk.jwk_id.iss;

        if (prev_issuer.is_none()) {
            prev_issuer.fill(*cur_iss);
        } else if (cur_iss != prev_issuer.borrow()) {
            *prev_issuer.borrow_mut() = *cur_iss;
            j = j + 1;
        };

        let max_epoch_for_iss = &issuer_max_epochs[j];

        // TODO: if the iss for this jwk has *no* jwks that meet the minimum epoch,
        // then expire nothing.
        if (*max_epoch_for_iss < min_epoch || jwk.epoch >= min_epoch) {
            new_active_jwks.push_back(*jwk);
        };
        i = i + 1;
    };
    inner.active_jwks = new_active_jwks;
}

#[allow(unused_function)]
/// Get the current active_jwks. Called when the node starts up in order to load the current
/// JWK state from the chain.
fun get_active_jwks(self: &AuthenticatorState, ctx: &TxContext): vector<ActiveJwk> {
    assert!(ctx.sender() == @0x0, ENotSystemAddress);
    self.load_inner().active_jwks
}

#[test_only]
public fun create_for_testing(ctx: &TxContext) {
    create(ctx);
}

#[test_only]
public fun update_authenticator_state_for_testing(
    self: &mut AuthenticatorState,
    new_active_jwks: vector<ActiveJwk>,
    ctx: &TxContext,
) {
    self.update_authenticator_state(new_active_jwks, ctx);
}

#[test_only]
public fun expire_jwks_for_testing(self: &mut AuthenticatorState, min_epoch: u64, ctx: &TxContext) {
    self.expire_jwks(min_epoch, ctx);
}

#[test_only]
public fun get_active_jwks_for_testing(
    self: &AuthenticatorState,
    ctx: &TxContext,
): vector<ActiveJwk> {
    self.get_active_jwks(ctx)
}
