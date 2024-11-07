// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module sui::ecdsa_r1;

#[allow(unused_const)]
/// Error if the public key cannot be recovered from the signature.
const EFailToRecoverPubKey: u64 = 0;

#[allow(unused_const)]
/// Error if the signature is invalid.
const EInvalidSignature: u64 = 1;

#[allow(unused_const)]
/// Hash function name that are valid for ecrecover and secp256k1_verify.
const KECCAK256: u8 = 0;
#[allow(unused_const)]
const SHA256: u8 = 1;

/// @param signature: A 65-bytes signature in form (r, s, v) that is signed using
/// Secp256r1. Reference implementation on signature generation using RFC6979:
/// https://github.com/MystenLabs/fastcrypto/blob/74aec4886e62122a5b769464c2bea5f803cf8ecc/fastcrypto/src/secp256r1/mod.rs
/// The accepted v values are {0, 1, 2, 3}.
/// @param msg: The message that the signature is signed against, this is raw message without hashing.
/// @param hash: The u8 representing the name of hash function used to hash the message when signing.
///
/// If the signature is valid, return the corresponding recovered Secpk256r1 public
/// key, otherwise throw error. This is similar to ecrecover in Ethereum, can only be
/// applied to Secp256r1 signatures. May fail with `EFailToRecoverPubKey` or `EInvalidSignature`.
public native fun secp256r1_ecrecover(
    signature: &vector<u8>,
    msg: &vector<u8>,
    hash: u8,
): vector<u8>;

/// @param signature: A 64-bytes signature in form (r, s) that is signed using
/// Secp256r1. This is an non-recoverable signature without recovery id.
/// Reference implementation on signature generation using RFC6979:
/// https://github.com/MystenLabs/fastcrypto/blob/74aec4886e62122a5b769464c2bea5f803cf8ecc/fastcrypto/src/secp256r1/mod.rs
/// @param public_key: The public key to verify the signature against
/// @param msg: The message that the signature is signed against, this is raw message without hashing.
/// @param hash: The u8 representing the name of hash function used to hash the message when signing.
///
/// If the signature is valid to the pubkey and hashed message, return true. Else false.
public native fun secp256r1_verify(
    signature: &vector<u8>,
    public_key: &vector<u8>,
    msg: &vector<u8>,
    hash: u8,
): bool;
