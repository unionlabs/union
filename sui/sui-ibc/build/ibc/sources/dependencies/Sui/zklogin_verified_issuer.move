// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_const)]
module sui::zklogin_verified_issuer;

use std::string::String;

/// Error if the proof consisting of the inputs provided to the verification function is invalid.
const EInvalidInput: u64 = 0;

/// Error if the proof consisting of the inputs provided to the verification function is invalid.
const EInvalidProof: u64 = 1;

/// Possession of a VerifiedIssuer proves that the user's address was created using zklogin and with the given issuer
/// (identity provider).
public struct VerifiedIssuer has key {
    /// The ID of this VerifiedIssuer
    id: UID,
    /// The address this VerifiedID is associated with
    owner: address,
    /// The issuer
    issuer: String,
}

/// Returns the address associated with the given VerifiedIssuer
public fun owner(verified_issuer: &VerifiedIssuer): address {
    verified_issuer.owner
}

/// Returns the issuer associated with the given VerifiedIssuer
public fun issuer(verified_issuer: &VerifiedIssuer): &String {
    &verified_issuer.issuer
}

/// Delete a VerifiedIssuer
public fun delete(verified_issuer: VerifiedIssuer) {
    let VerifiedIssuer { id, owner: _, issuer: _ } = verified_issuer;
    id.delete();
}

/// Verify that the caller's address was created using zklogin with the given issuer. If so, a VerifiedIssuer object
/// with the issuers id transferred to the caller.
///
/// Aborts with `EInvalidProof` if the verification fails.
public fun verify_zklogin_issuer(address_seed: u256, issuer: String, ctx: &mut TxContext) {
    let sender = ctx.sender();
    assert!(check_zklogin_issuer(sender, address_seed, &issuer), EInvalidProof);
    transfer::transfer(
        VerifiedIssuer {
            id: object::new(ctx),
            owner: sender,
            issuer,
        },
        sender,
    )
}

/// Returns true if `address` was created using zklogin with the given issuer and address seed.
public fun check_zklogin_issuer(address: address, address_seed: u256, issuer: &String): bool {
    check_zklogin_issuer_internal(address, address_seed, issuer.as_bytes())
}

/// Returns true if `address` was created using zklogin with the given issuer and address seed.
///
/// Aborts with `EInvalidInput` if the `iss` input is not a valid UTF-8 string.
native fun check_zklogin_issuer_internal(
    address: address,
    address_seed: u256,
    issuer: &vector<u8>,
): bool;
