// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_const, unused_function)]
module sui::zklogin_verified_id;

use std::string::String;

const EFunctionDisabled: u64 = 0;

/// Possession of a VerifiedID proves that the user's address was created using zklogin and the given parameters.
public struct VerifiedID has key {
    /// The ID of this VerifiedID
    id: UID,
    /// The address this VerifiedID is associated with
    owner: address,
    /// The name of the key claim
    key_claim_name: String,
    /// The value of the key claim
    key_claim_value: String,
    /// The issuer
    issuer: String,
    /// The audience (wallet)
    audience: String,
}

/// Returns the address associated with the given VerifiedID
public fun owner(verified_id: &VerifiedID): address {
    verified_id.owner
}

/// Returns the name of the key claim associated with the given VerifiedID
public fun key_claim_name(verified_id: &VerifiedID): &String {
    &verified_id.key_claim_name
}

/// Returns the value of the key claim associated with the given VerifiedID
public fun key_claim_value(verified_id: &VerifiedID): &String {
    &verified_id.key_claim_value
}

/// Returns the issuer associated with the given VerifiedID
public fun issuer(verified_id: &VerifiedID): &String {
    &verified_id.issuer
}

/// Returns the audience (wallet) associated with the given VerifiedID
public fun audience(verified_id: &VerifiedID): &String {
    &verified_id.audience
}

/// Delete a VerifiedID
public fun delete(verified_id: VerifiedID) {
    let VerifiedID { id, owner: _, key_claim_name: _, key_claim_value: _, issuer: _, audience: _ } =
        verified_id;
    id.delete();
}

/// This function has been disabled.
public fun verify_zklogin_id(
    _key_claim_name: String,
    _key_claim_value: String,
    _issuer: String,
    _audience: String,
    _pin_hash: u256,
    _ctx: &mut TxContext,
) {
    assert!(false, EFunctionDisabled);
}

/// This function has been disabled.
public fun check_zklogin_id(
    _address: address,
    _key_claim_name: &String,
    _key_claim_value: &String,
    _issuer: &String,
    _audience: &String,
    _pin_hash: u256,
): bool {
    assert!(false, EFunctionDisabled);
    false
}

/// Returns true if `address` was created using zklogin and the given parameters.
///
/// Aborts with `EInvalidInput` if any of `kc_name`, `kc_value`, `iss` and `aud` is not a properly encoded UTF-8
/// string or if the inputs are longer than the allowed upper bounds: `kc_name` must be at most 32 characters,
/// `kc_value` must be at most 115 characters and `aud` must be at most 145 characters.
native fun check_zklogin_id_internal(
    address: address,
    key_claim_name: &vector<u8>,
    key_claim_value: &vector<u8>,
    issuer: &vector<u8>,
    audience: &vector<u8>,
    pin_hash: u256,
): bool;
