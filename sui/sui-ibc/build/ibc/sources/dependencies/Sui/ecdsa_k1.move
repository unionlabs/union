// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module sui::ecdsa_k1;

#[allow(unused_const)]
/// Error if the public key cannot be recovered from the signature.
const EFailToRecoverPubKey: u64 = 0;

#[allow(unused_const)]
/// Error if the signature is invalid.
const EInvalidSignature: u64 = 1;

#[allow(unused_const)]
/// Error if the public key is invalid.
const EInvalidPubKey: u64 = 2;

#[allow(unused_const)]
#[test_only]
/// Error if the private key is invalid.
const EInvalidPrivKey: u64 = 3;

#[allow(unused_const)]
#[test_only]
/// Error if the given hash function does not exist.
const EInvalidHashFunction: u64 = 4;

#[allow(unused_const)]
#[test_only]
/// Error if the seed is invalid.
const EInvalidSeed: u64 = 5;

#[allow(unused_const)]
/// Hash function name that are valid for ecrecover and secp256k1_verify.
const KECCAK256: u8 = 0;
#[allow(unused_const)]
const SHA256: u8 = 1;

/// @param signature: A 65-bytes signature in form (r, s, v) that is signed using
/// Secp256k1. Reference implementation on signature generation using RFC6979:
/// https://github.com/MystenLabs/narwhal/blob/5d6f6df8ccee94446ff88786c0dbbc98be7cfc09/crypto/src/secp256k1.rs
/// The accepted v values are {0, 1, 2, 3}.
/// @param msg: The message that the signature is signed against, this is raw message without hashing.
/// @param hash: The hash function used to hash the message when signing.
///
/// If the signature is valid, return the corresponding recovered Secpk256k1 public
/// key, otherwise throw error. This is similar to ecrecover in Ethereum, can only be
/// applied to Secp256k1 signatures. May abort with `EFailToRecoverPubKey` or `EInvalidSignature`.
public native fun secp256k1_ecrecover(
    signature: &vector<u8>,
    msg: &vector<u8>,
    hash: u8,
): vector<u8>;

/// @param pubkey: A 33-bytes compressed public key, a prefix either 0x02 or 0x03 and a 256-bit integer.
///
/// If the compressed public key is valid, return the 65-bytes uncompressed public key,
/// otherwise throw error. May abort with `EInvalidPubKey`.
public native fun decompress_pubkey(pubkey: &vector<u8>): vector<u8>;

/// @param signature: A 64-bytes signature in form (r, s) that is signed using
/// Secp256k1. This is an non-recoverable signature without recovery id.
/// Reference implementation on signature generation using RFC6979:
/// https://github.com/MystenLabs/fastcrypto/blob/74aec4886e62122a5b769464c2bea5f803cf8ecc/fastcrypto/src/secp256k1/mod.rs#L193
/// @param public_key: The public key to verify the signature against
/// @param msg: The message that the signature is signed against, this is raw message without hashing.
/// @param hash: The hash function used to hash the message when signing.
///
/// If the signature is valid to the pubkey and hashed message, return true. Else false.
public native fun secp256k1_verify(
    signature: &vector<u8>,
    public_key: &vector<u8>,
    msg: &vector<u8>,
    hash: u8,
): bool;

#[test_only]
/// @param private_key: A 32-bytes private key that is used to sign the message.
/// @param msg: The message to sign, this is raw message without hashing.
/// @param hash: The hash function used to hash the message when signing.
/// @param recoverable: A boolean flag to indicate if the produced signature should be recoverable.
///
/// Return the signature in form (r, s) that is signed using Secp256k1.
/// If `recoverable` is true, the signature will be in form (r, s, v) where v is the recovery id.
///
/// This should ONLY be used in tests, because it will reveal the private key onchain.
public native fun secp256k1_sign(
    private_key: &vector<u8>,
    msg: &vector<u8>,
    hash: u8,
    recoverable: bool,
): vector<u8>;

#[test_only]
public struct KeyPair has drop {
    private_key: vector<u8>,
    public_key: vector<u8>,
}

#[test_only]
public fun private_key(self: &KeyPair): &vector<u8> {
    &self.private_key
}

#[test_only]
public fun public_key(self: &KeyPair): &vector<u8> {
    &self.public_key
}

#[test_only]
/// @param seed: A 32-bytes seed that is used to generate the keypair.
///
/// Returns a Secp256k1 keypair deterministically generated from the seed.
public native fun secp256k1_keypair_from_seed(seed: &vector<u8>): KeyPair;
