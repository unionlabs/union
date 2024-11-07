// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Group operations of BLS12-381.
module sui::bls12381;

use sui::group_ops::{Self, Element};

/// @param signature: A 48-bytes signature that is a point on the G1 subgroup.
/// @param public_key: A 96-bytes public key that is a point on the G2 subgroup.
/// @param msg: The message that we test the signature against.
///
/// If the signature is a valid signature of the message and public key according to
/// BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_, return true. Otherwise, return false.
public native fun bls12381_min_sig_verify(
    signature: &vector<u8>,
    public_key: &vector<u8>,
    msg: &vector<u8>,
): bool;

/// @param signature: A 96-bytes signature that is a point on the G2 subgroup.
/// @param public_key: A 48-bytes public key that is a point on the G1 subgroup.
/// @param msg: The message that we test the signature against.
///
/// If the signature is a valid signature of the message and public key according to
/// BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_, return true. Otherwise, return false.
public native fun bls12381_min_pk_verify(
    signature: &vector<u8>,
    public_key: &vector<u8>,
    msg: &vector<u8>,
): bool;

/////////////////////////////////////////////
////// Elliptic curve operations //////

public struct Scalar {}
public struct G1 {}
public struct G2 {}
public struct GT {}
public struct UncompressedG1 {}

// Scalars are encoded using big-endian byte order.
// G1 and G2 are encoded using big-endian byte order and points are compressed. See
// https://www.ietf.org/archive/id/draft-irtf-cfrg-pairing-friendly-curves-11.html and
// https://docs.rs/bls12_381/latest/bls12_381/notes/serialization/index.html for details.
// GT is encoded using big-endian byte order and points are uncompressed and not intended
// to be deserialized.
// UncompressedG1 elements are G1 elements in uncompressed form. They are larger but faster to 
// use since they do not have to be uncompressed before use. They can not be constructed 
// on their own but have to be created from G1 elements.

// Const elements.
const SCALAR_ZERO_BYTES: vector<u8> =
    x"0000000000000000000000000000000000000000000000000000000000000000";
const SCALAR_ONE_BYTES: vector<u8> =
    x"0000000000000000000000000000000000000000000000000000000000000001";
const G1_IDENTITY_BYTES: vector<u8> =
    x"c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
const G1_GENERATOR_BYTES: vector<u8> =
    x"97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb";
const G2_IDENTITY_BYTES: vector<u8> =
    x"c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
const G2_GENERATOR_BYTES: vector<u8> =
    x"93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8";
const GT_IDENTITY_BYTES: vector<u8> =
    x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
const GT_GENERATOR_BYTES: vector<u8> =
    x"1250ebd871fc0a92a7b2d83168d0d727272d441befa15c503dd8e90ce98db3e7b6d194f60839c508a84305aaca1789b6089a1c5b46e5110b86750ec6a532348868a84045483c92b7af5af689452eafabf1a8943e50439f1d59882a98eaa0170f19f26337d205fb469cd6bd15c3d5a04dc88784fbb3d0b2dbdea54d43b2b73f2cbb12d58386a8703e0f948226e47ee89d06fba23eb7c5af0d9f80940ca771b6ffd5857baaf222eb95a7d2809d61bfe02e1bfd1b68ff02f0b8102ae1c2d5d5ab1a1368bb445c7c2d209703f239689ce34c0378a68e72a6b3b216da0e22a5031b54ddff57309396b38c881c4c849ec23e87193502b86edb8857c273fa075a50512937e0794e1e65a7617c90d8bd66065b1fffe51d7a579973b1315021ec3c19934f11b8b424cd48bf38fcef68083b0b0ec5c81a93b330ee1a677d0d15ff7b984e8978ef48881e32fac91b93b47333e2ba5703350f55a7aefcd3c31b4fcb6ce5771cc6a0e9786ab5973320c806ad360829107ba810c5a09ffdd9be2291a0c25a99a201b2f522473d171391125ba84dc4007cfbf2f8da752f7c74185203fcca589ac719c34dffbbaad8431dad1c1fb597aaa5018107154f25a764bd3c79937a45b84546da634b8f6be14a8061e55cceba478b23f7dacaa35c8ca78beae9624045b4b604c581234d086a9902249b64728ffd21a189e87935a954051c7cdba7b3872629a4fafc05066245cb9108f0242d0fe3ef0f41e58663bf08cf068672cbd01a7ec73baca4d72ca93544deff686bfd6df543d48eaa24afe47e1efde449383b676631";

// Internal types used by group_ops' native functions.
const SCALAR_TYPE: u8 = 0;
const G1_TYPE: u8 = 1;
const G2_TYPE: u8 = 2;
const GT_TYPE: u8 = 3;
const UNCOMPRESSED_G1_TYPE: u8 = 4;

///////////////////////////////
////// Scalar operations //////

public fun scalar_from_bytes(bytes: &vector<u8>): Element<Scalar> {
    group_ops::from_bytes(SCALAR_TYPE, bytes, false)
}

public fun scalar_from_u64(x: u64): Element<Scalar> {
    let mut bytes = SCALAR_ZERO_BYTES;
    group_ops::set_as_prefix(x, true, &mut bytes);
    group_ops::from_bytes(SCALAR_TYPE, &bytes, true)
}

public fun scalar_zero(): Element<Scalar> {
    let zero = SCALAR_ZERO_BYTES;
    group_ops::from_bytes(SCALAR_TYPE, &zero, true)
}

public fun scalar_one(): Element<Scalar> {
    let one = SCALAR_ONE_BYTES;
    group_ops::from_bytes(SCALAR_TYPE, &one, true)
}

public fun scalar_add(e1: &Element<Scalar>, e2: &Element<Scalar>): Element<Scalar> {
    group_ops::add(SCALAR_TYPE, e1, e2)
}

public fun scalar_sub(e1: &Element<Scalar>, e2: &Element<Scalar>): Element<Scalar> {
    group_ops::sub(SCALAR_TYPE, e1, e2)
}

public fun scalar_mul(e1: &Element<Scalar>, e2: &Element<Scalar>): Element<Scalar> {
    group_ops::mul(SCALAR_TYPE, e1, e2)
}

/// Returns e2/e1, fails if a is zero.
public fun scalar_div(e1: &Element<Scalar>, e2: &Element<Scalar>): Element<Scalar> {
    group_ops::div(SCALAR_TYPE, e1, e2)
}

public fun scalar_neg(e: &Element<Scalar>): Element<Scalar> {
    scalar_sub(&scalar_zero(), e)
}

// Fails if e is zero.
public fun scalar_inv(e: &Element<Scalar>): Element<Scalar> {
    scalar_div(e, &scalar_one())
}

/////////////////////////////////
////// G1 group operations //////

public fun g1_from_bytes(bytes: &vector<u8>): Element<G1> {
    group_ops::from_bytes(G1_TYPE, bytes, false)
}

public fun g1_identity(): Element<G1> {
    let identity = G1_IDENTITY_BYTES;
    group_ops::from_bytes(G1_TYPE, &identity, true)
}

public fun g1_generator(): Element<G1> {
    let generator = G1_GENERATOR_BYTES;
    group_ops::from_bytes(G1_TYPE, &generator, true)
}

public fun g1_add(e1: &Element<G1>, e2: &Element<G1>): Element<G1> {
    group_ops::add(G1_TYPE, e1, e2)
}

public fun g1_sub(e1: &Element<G1>, e2: &Element<G1>): Element<G1> {
    group_ops::sub(G1_TYPE, e1, e2)
}

public fun g1_mul(e1: &Element<Scalar>, e2: &Element<G1>): Element<G1> {
    group_ops::mul(G1_TYPE, e1, e2)
}

/// Returns e2 / e1, fails if scalar is zero.
public fun g1_div(e1: &Element<Scalar>, e2: &Element<G1>): Element<G1> {
    group_ops::div(G1_TYPE, e1, e2)
}

public fun g1_neg(e: &Element<G1>): Element<G1> {
    g1_sub(&g1_identity(), e)
}

/// Hash using DST = BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_
public fun hash_to_g1(m: &vector<u8>): Element<G1> {
    group_ops::hash_to(G1_TYPE, m)
}

/// Let 'scalars' be the vector [s1, s2, ..., sn] and 'elements' be the vector [e1, e2, ..., en].
/// Returns s1*e1 + s2*e2 + ... + sn*en.
/// Aborts with `EInputTooLong` if the vectors are larger than 32 (may increase in the future).
public fun g1_multi_scalar_multiplication(
    scalars: &vector<Element<Scalar>>,
    elements: &vector<Element<G1>>,
): Element<G1> {
    group_ops::multi_scalar_multiplication(G1_TYPE, scalars, elements)
}

/// Convert an `Element<G1>` to uncompressed form.
public fun g1_to_uncompressed_g1(e: &Element<G1>): Element<UncompressedG1> {
    group_ops::convert(G1_TYPE, UNCOMPRESSED_G1_TYPE, e)
}

/////////////////////////////////
////// G2 group operations //////

public fun g2_from_bytes(bytes: &vector<u8>): Element<G2> {
    group_ops::from_bytes(G2_TYPE, bytes, false)
}

public fun g2_identity(): Element<G2> {
    let identity = G2_IDENTITY_BYTES;
    group_ops::from_bytes(G2_TYPE, &identity, true)
}

public fun g2_generator(): Element<G2> {
    let generator = G2_GENERATOR_BYTES;
    group_ops::from_bytes(G2_TYPE, &generator, true)
}

public fun g2_add(e1: &Element<G2>, e2: &Element<G2>): Element<G2> {
    group_ops::add(G2_TYPE, e1, e2)
}

public fun g2_sub(e1: &Element<G2>, e2: &Element<G2>): Element<G2> {
    group_ops::sub(G2_TYPE, e1, e2)
}

public fun g2_mul(e1: &Element<Scalar>, e2: &Element<G2>): Element<G2> {
    group_ops::mul(G2_TYPE, e1, e2)
}

/// Returns e2 / e1, fails if scalar is zero.
public fun g2_div(e1: &Element<Scalar>, e2: &Element<G2>): Element<G2> {
    group_ops::div(G2_TYPE, e1, e2)
}

public fun g2_neg(e: &Element<G2>): Element<G2> {
    g2_sub(&g2_identity(), e)
}

/// Hash using DST = BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_
public fun hash_to_g2(m: &vector<u8>): Element<G2> {
    group_ops::hash_to(G2_TYPE, m)
}

/// Let 'scalars' be the vector [s1, s2, ..., sn] and 'elements' be the vector [e1, e2, ..., en].
/// Returns s1*e1 + s2*e2 + ... + sn*en.
/// Aborts with `EInputTooLong` if the vectors are larger than 32 (may increase in the future).
public fun g2_multi_scalar_multiplication(
    scalars: &vector<Element<Scalar>>,
    elements: &vector<Element<G2>>,
): Element<G2> {
    group_ops::multi_scalar_multiplication(G2_TYPE, scalars, elements)
}

/////////////////////////////////
////// Gt group operations //////

public fun gt_identity(): Element<GT> {
    let identity = GT_IDENTITY_BYTES;
    group_ops::from_bytes(GT_TYPE, &identity, true)
}

public fun gt_generator(): Element<GT> {
    let generator = GT_GENERATOR_BYTES;
    group_ops::from_bytes(GT_TYPE, &generator, true)
}

public fun gt_add(e1: &Element<GT>, e2: &Element<GT>): Element<GT> {
    group_ops::add(GT_TYPE, e1, e2)
}

public fun gt_sub(e1: &Element<GT>, e2: &Element<GT>): Element<GT> {
    group_ops::sub(GT_TYPE, e1, e2)
}

public fun gt_mul(e1: &Element<Scalar>, e2: &Element<GT>): Element<GT> {
    group_ops::mul(GT_TYPE, e1, e2)
}

/// Returns e2 / e1, fails if scalar is zero.
public fun gt_div(e1: &Element<Scalar>, e2: &Element<GT>): Element<GT> {
    group_ops::div(GT_TYPE, e1, e2)
}

public fun gt_neg(e: &Element<GT>): Element<GT> {
    gt_sub(&gt_identity(), e)
}

/////////////////////
////// Pairing //////

public fun pairing(e1: &Element<G1>, e2: &Element<G2>): Element<GT> {
    group_ops::pairing(G1_TYPE, e1, e2)
}

///////////////////////////////////////
/// UncompressedG1 group operations ///

/// Create a `Element<G1>` from its uncompressed form.
public fun uncompressed_g1_to_g1(e: &Element<UncompressedG1>): Element<G1> {
    group_ops::convert(UNCOMPRESSED_G1_TYPE, G1_TYPE, e)
}

/// Compute the sum of a list of uncompressed elements.
/// This is significantly faster and cheaper than summing the elements.
public fun uncompressed_g1_sum(terms: &vector<Element<UncompressedG1>>): Element<UncompressedG1> {
    group_ops::sum(UNCOMPRESSED_G1_TYPE, terms)
}
