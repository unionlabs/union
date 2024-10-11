schnorr
=======

[![Build Status](https://github.com/decred/dcrd/workflows/Build%20and%20Test/badge.svg)](https://github.com/decred/dcrd/actions)
[![ISC License](https://img.shields.io/badge/license-ISC-blue.svg)](http://copyfree.org)
[![GoDoc](https://img.shields.io/badge/godoc-reference-blue.svg)](https://pkg.go.dev/github.com/decred/dcrd/dcrec/secp256k1/v4/schnorr)

Package schnorr provides custom Schnorr signing and verification via secp256k1.

This package provides data structures and functions necessary to produce and
verify deterministic canonical Schnorr signatures using a custom scheme named
`EC-Schnorr-DCRv0` that is described herein.  The signatures and implementation
are optimized specifically for the secp256k1 curve.  See
https://www.secg.org/sec2-v2.pdf for details on the secp256k1 standard.

It also provides functions to parse and serialize the Schnorr signatures
according to the specification described herein.

A comprehensive suite of tests is provided to ensure proper functionality.

## Overview

A Schnorr signature is a digital signature scheme that is known for its
simplicity, provable security and efficient generation of short signatures.

It provides many advantages over ECDSA signatures that make them ideal for use
with the only real downside being that they are not well standardized at the
time of this writing.

Some of the advantages over ECDSA include:

* They are linear which makes them easier to aggregate and use in protocols that
  build on them such as multi-party signatures, threshold signatures, adaptor
  signatures, and blind signatures
* They are provably secure with weaker assumptions than the best known security
  proofs for ECDSA
  *  Specifically Schnorr signatures are provably secure under SUF-CMA (Strong
     Existential Unforgeability under Chosen Message Attack) in the ROM (Random
     Oracle Model) which guarantees that as long as the hash function behaves
     ideally, the only way to break Schnorr signatures is by solving the ECDLP
     (Elliptic Curve Discrete Logarithm Problem).
* Their relatively straightforward and efficient aggregation properties make
  them excellent for scalability and allow them to provide some nice privacy
  characteristics
* They support faster batch verification unlike the standardized version of
  ECDSA signatures

## Custom Schnorr-based Signature Scheme

As mentioned in the overview, the primary downside of Schnorr signatures for
elliptic curves is that they are not standardized as well as ECDSA signatures
which means there are a number of variations that are not compatible with each
other.

In addition, many of the standardization attempts have various disadvantages
that make them unsuitable for use in Decred.  Some of these details will be
discussed further in the Design section along with providing some insight into
the design decisions made.

Consequently, this package implements a custom Schnorr-based signature scheme
named `EC-Schnorr-DCRv0` suitable for use in Decred.

The following provides a high-level overview of the key design features of the
scheme:

* Uses signatures of the form `(R, s)`
* Produces 64-byte signatures by only encoding the `x` coordinate of `R`
* Enforces even `y` coordinates for `R` to support efficient verification by
  disambiguating the two possible `y` coordinates
* Canonically encodes by both components of the signature with 32-bytes each
* Uses BLAKE-256 with 14 rounds for the hash function to calculate challenge `e`
* Uses RFC6979 to obviate the need for an entropy source at signing time
* Produces deterministic signatures for a given message and private key pair

## Design Details And Rationale

As previously mentioned, unfortunately, Schnorr signatures for elliptic curves
are not standardized as well as ECDSA signatures which means there are a number
of variations that are not compatible with each other.  For example, different
schemes can be found in ISO/IEC 14888-3:2016 (`EC-SDSA`, `EC-SDSA-opt`, and
`EC-FSDSA`), BSI TR-03111, Versions 2.0 (`EC-Schnorr-v2.0`) and 2.10
(`EC-Schnorr-v2.10`), and published by Clause Schnorr himself in Efficient
Signature Generation by Smart Cards in the Journal of Cryptology, Vol.4,
pp.161-174, 1991 (`Sc91`).  There are other variants not covered here as well.

Further, each of these schemes have various disadvantages that are discussed
more in the following sections which make them unsuitable for use with Decred.
Consequently, Decred makes use of a custom scheme named `EC-Schnorr-DCRv0`.

That said, the various schemes are all fairly simple variations which involve
using an agreed upon elliptic curve with generator `G`, and hash function `hash`
where the signer, with public key `Q` and signing message `m`, picks a _unique_
random point `R` and two integers `s` and `e` such that the following equations
hold:

*  `e = hash(R || m)`
*  `s*G = R + e*Q`

The signature itself then consists of two components with two major variants
that depend on which of `e` or `R` the signer wishes to reveal and further minor
variants which depend on the data that is hashed as well as the sign used in the
calculation of `s`, which in a sense, can be thought of as the sign applied to
the public key.

For reference, the following is a high-level breakdown of some of the key
differences between the aforementioned schemes, where `d` is the signers
private key and `k` is the secret nonce used to generate the random point
`R = k*G` (note that `R.x` and `R.y` indicate the x and y coordinates of the
point `R`, respectively):

| Scheme           | Pubkey | Challenge (e)       | 1st part  | 2nd part (mod N) |
|------------------|--------|---------------------|-----------|------------------|
| Sc91             | -d*G   | hash(R ∥ m)         | e         | k + d*e          |
| EC-SDSA          | -d*G   | hash(R.x ∥ R.y ∥ m) | e         | k + d*e          |
| EC-SDSA-opt      | -d*G   | hash(R.x ∥ m)       | e         | k + d*e          |
| EC-FSDSA         |  d*G   | hash(R.x ∥ R.y ∥ m) | R.x ∥ R.y | k - d*e          |
| EC-Schnorr-v2.0  |  d*G   | hash(m ∥ R.x)       | e         | k - d*e          |
| EC-Schnorr-v2.10 | -d*G   | hash(R.x ∥ R.y ∥ m) | e         | k + d*e          |
| EC-Schnorr-DCRv0 |  d*G   | hash(R.x ∥ m)       | R.x       | k - d*e          |

Notice the main differences are:

* The signature pair is either `(e, s)` or `(R, s)`
* The `y` coordinate of `R` is either explicit or implicit leaving it up to the
  verifier to calculate from the `x` coordinate
* The exact serialization of the data used to create the challenge `e` varies, but
  they all commit to the random point `R` and the message `m`
* The calculation of `s` either uses addition or subtraction meaning the
  verifier must use the opposite operation which essentially changes the sign of
  the public key in the addition case

### Revealing `e` vs `R`

The first main difference is whether or not `e` or `R` is revealed, meaning the
signature is either the pair `(e, s)` or the pair `(R, s)`, respectively.

In the case `e` is revealed, the verifier must calculate `R = s*G - e*Q` and
thus implies the pair must satisfy `e = hash(s*G - e*Q || m)`.  On the other
hand, when `R` is revealed, the verifier must calculate
`s*G = R + hash(R || m)*Q`.

This is an important distinction because, while the first approach of revealing
`e` does have the theoretical potential for smaller signatures, since it is the
result of a hash function as opposed to encoding `R`, it also has a couple of
important disadvantages that the second approach does not:

* It makes the implementation of secure joint multi-party and threshold
  signatures more difficult
* It does not support fast batch verification due to the fact the necessary
  elliptic curve operations are performed inside the hash function which
  prevents the ability to take advantage of their homomorphic properties

Further, when the size of the field elements for the elliptic curve is the same
size as the hash function, as is the case in Decred, techniques which are
discussed in the next section can be used to make `(R, s)` signatures the same
size as `(e, s)` signatures.

Therefore, the second approach of `(R, s)` signatures is chosen.

### Explicit Versus Implicit `y` Coordinate for `R`

The second main difference is whether or not the full `R` point is explicitly or
implicitly specified.  Explicitly specifying it refers to committing directly to
and/or revealing both the `x` and `y` coordinates of the point while the
implicit option only makes use of the `x` coordinate.

Using the implicit option provides for smaller signatures of the `(R, s)` form
since it means the `R` component requires less bytes to encode.

However, even though all that is required for single signature validations in
some implicit schemes is the `x` coordinate of `R`, in order to support batch
validation the verifiers must know the full point, which includes the `y`
coordinate.

Since there are two possible `y` coordinates corresponding to the `x`
coordinate, in cases where the full point `R` must be known to verifiers, some
additional semantics are required to correctly identify which one is the correct
one.

While there are various methods, the choice made for `EC-Schnorr-DCRv0` is to
enforce the `y` coordinate to be even which can be efficiently accomplished at
signing time by negating the nonce in the case it is odd and it does not involve
adding an additional byte in the signature to specify the oddness.  This is
possible because all valid points on the secp256k1 curve always have
corresponding `y` coordinates where one is even and the other is odd due to them
being the negation of each other modulo the underlying field prime, which an odd
number.

### Challenge (`e`) Calculation

The third main difference is the specific format of the data that is used to
create the challenge via the hash function.  Some variants include both the `x`
and `y` coordinates of the point `R` while others only include the `x`
coordinate.  See the previous section about explicit versus implicit `y`
coordinates since this choice also ties into that. Also, one of the variants
reverses the order of the concatenation of the message `m` and the point `R`.

Combining these details with the following additional information specific to
Decred results in the choice of `e = BLAKE-256(R.x || m)` for the challenge with
additional restrictions on `R` to ensure verifiers can reconstruct the full
point:

* The order of the committed information for the challenge is not important and
  the more common formulation consists of `R` followed by `m`
* Compact signatures are more desirable since they end up in the public ledger
* The BLAKE-256 hash function with 14 rounds is already in widespread use and
  satisfies all requirements needed for the hash function

### Sign For `s` Calculation

The final main difference is whether or not `s` is calculated as `s = k - d*e`
or `s = k + d*e`.  The verification equation naturally changes depending on
which variant is used: `R = s*G + e*Q` for the first and `R = s*G - e*Q` for
the second.  For this reason, it is perhaps easier to think of it terms of the
the signature scheme choosing whether or not to flip the sign on the public key
`Q` depending on which `s` calculation is used.

The only option among the covered standardization attempts that returns
signatures pairs in the `(R, s)` form uses the `s = k - d*e` variant, so that
variant is chosen for `EC-Schnorr-DCRv0` as well.

## Future Design Considerations

It is worth noting that there are some additional optimizations and
modifications that have been identified since the introduction of
`EC-Schnorr-DCRv0` that can be made to further harden security for multi-party
and threshold signature use cases as well provide the opportunity for faster
signature verification with a sufficiently optimized implementation.

However, the v0 scheme is used in the existing consensus rules and any changes
to the signature scheme would invalidate existing uses.  Therefore changes in
this regard will need to come in the form of a v1 signature scheme and be
accompanied by the necessary consensus updates.

## EC-Schnorr-DCRv0 Specification

### EC-Schnorr-DCRv0 Signing Algorithm

The algorithm for producing an EC-Schnorr-DCRv0 signature is as follows:

G = curve generator
n = curve order
d = private key
m = message
r, s = signature

1. Fail if m is not 32 bytes
2. Fail if d = 0 or d >= n
3. Use RFC6979 to generate a deterministic nonce k in [1, n-1] parameterized by
   the private key, message being signed, extra data that identifies the scheme
   (`0x0b75f97b60e8a5762876c004829ee9b926fa6f0d2eeaec3a4fd1446a768331cb`), and
   an iteration count
4. R = kG
5. Negate nonce k if R.y is odd (R.y is the y coordinate of the point R)
6. r = R.x (R.x is the x coordinate of the point R)
7. e = BLAKE-256(r || m) (Ensure r is padded to 32 bytes)
8. Repeat from step 3 (with iteration + 1) if e >= n
9. s = k - e*d mod n
10. Return (r, s)

### EC-Schnorr-DCRv0 Verification Algorithm

The algorithm for verifying an EC-Schnorr-DCRv0 signature is as follows:

G = curve generator
n = curve order
p = field size
Q = public key
m = message
r, s = signature

1. Fail if m is not 32 bytes
2. Fail if Q is not a point on the curve
3. Fail if r >= p
4. Fail if s >= n
5. e = BLAKE-256(r || m) (Ensure r is padded to 32 bytes)
6. Fail if e >= n
7. R = s*G + e*Q
8. Fail if R is the point at infinity
9. Fail if R.y is odd
10. Verified if R.x == r

### EC-Schnorr-DCRv0 Signature Serialization Format

The serialization format consists of the two components of the signature, `R.x`
(the `x` coordinate of the point `R`) and `s`, each serialized as a padded
big-endian uint256 (32-bytes with the most significant byte first) and
concatenated together to form a 64-byte signature.

See the file `signature_test.go` for test vectors.

## Schnorr use in Decred

At the time of this writing, Schnorr signatures are not yet in widespread use on
the Decred network, largely due to the current lack of support in wallets and
infrastructure for secure multi-party and threshold signatures.

However, the consensus rules and scripting engine supports the necessary
primitives and given many of the beneficial properties of Schnorr signatures, a
good goal is to work towards providing the additional infrastructure to increase
their usage.

## Acknowledgements

The EC-Schnorr-DCRv0 scheme is partly based on work that was ongoing in the
Bitcoin community at the time it was implemented along with the following
standardization attempts:

* `EC-SDSA`, `EC-SDSA-opt`, and `EC-FSDSA` specified in ISO/IEC 14888-3:2016
* `EC-Schnorr-v2.0` and `EC-Schnorr-v2.10` specified in BSI TR-03111, Versions 2.0
  and v2.10, respectively
* `Sc91` specified in Efficient Signature Generation by Smart Cards in the
  Journal of Cryptology, Vol.4, pp.161-174, 1991

## Installation and Updating

This package is part of the `github.com/decred/dcrd/dcrec/secp256k1/v4` module.
Use the standard go tooling for working with modules to incorporate it.

## Examples

* [Sign Message](https://pkg.go.dev/github.com/decred/dcrd/dcrec/secp256k1/v4/schnorr#example-package-SignMessage)  
  Demonstrates signing a message with the EC-Schnorr-DCRv0 scheme using a
  secp256k1 private key that is first parsed from raw bytes and serializing the
  generated signature.

* [Verify Signature](https://pkg.go.dev/github.com/decred/dcrd/dcrec/secp256k1/v4/schnorr#example-Signature.Verify)  
  Demonstrates verifying an EC-Schnorr-DCRv0 signature against a public key that
  is first parsed from raw bytes.  The signature is also parsed from raw bytes.

## License

Package schnorr is licensed under the [copyfree](http://copyfree.org) ISC
License.
