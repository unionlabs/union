// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module sui::ed25519;

/// @param signature: 32-byte signature that is a point on the Ed25519 elliptic curve.
/// @param public_key: 32-byte signature that is a point on the Ed25519 elliptic curve.
/// @param msg: The message that we test the signature against.
///
/// If the signature is a valid Ed25519 signature of the message and public key, return true.
/// Otherwise, return false.
public native fun ed25519_verify(
    signature: &vector<u8>,
    public_key: &vector<u8>,
    msg: &vector<u8>,
): bool;
