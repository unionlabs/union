// Copyright (c) 2020-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

/*
Package schnorr provides custom Schnorr signing and verification via secp256k1.

This package provides data structures and functions necessary to produce and
verify deterministic canonical Schnorr signatures using a custom scheme named
EC-Schnorr-DCRv0 that is described herein.  The signatures and implementation
are optimized specifically for the secp256k1 curve.  See
https://www.secg.org/sec2-v2.pdf for details on the secp256k1 standard.

It also provides functions to parse and serialize the Schnorr signatures
according to the specification described herein.

A comprehensive suite of tests is provided to ensure proper functionality.

# Overview

A Schnorr signature is a digital signature scheme that is known for its
simplicity, provable security and efficient generation of short signatures.

It provides many advantages over ECDSA signatures that make them ideal for use
with the only real downside being that they are not well standardized at the
time of this writing.

Some of the advantages over ECDSA include:

  - They are linear which makes them easier to aggregate and use in protocols that
    build on them such as multi-party signatures, threshold signatures, adaptor
    signatures, and blind signatures
  - They are provably secure with weaker assumptions than the best known security
    proofs for ECDSA
  - Specifically Schnorr signatures are provably secure under SUF-CMA (Strong
    Existential Unforgeability under Chosen Message Attack) in the ROM (Random
    Oracle Model) which guarantees that as long as the hash function behaves
    ideally, the only way to break Schnorr signatures is by solving the ECDLP
    (Elliptic Curve Discrete Logarithm Problem).
  - Their relatively straightforward and efficient aggregation properties make
    them excellent for scalability and allow them to provide some nice privacy
    characteristics
  - They support faster batch verification unlike the standardized version of
    ECDSA signatures

# Custom Schnorr-based Signature Scheme

As mentioned in the overview, the primary downside of Schnorr signatures for
elliptic curves is that they are not standardized as well as ECDSA signatures
which means there are a number of variations that are not compatible with each
other.

In addition, many of the standardization attempts have various disadvantages
that make them unsuitable for use in Decred.  Some of these details and some
insight into the design decisions made are discussed further in the README.md
file.

Consequently, this package implements a custom Schnorr-based signature scheme
named EC-Schnorr-DCRv0 suitable for use in Decred.

The following provides a high-level overview of the key design features of the
scheme:

  - Uses signatures of the form (R, s)
  - Produces 64-byte signatures by only encoding the x coordinate of R
  - Enforces even y coordinates for R to support efficient verification by
    disambiguating the two possible y coordinates
  - Canonically encodes by both components of the signature with 32-bytes each
  - Uses BLAKE-256 with 14 rounds for the hash function to calculate challenge e
  - Uses RFC6979 to obviate the need for an entropy source at signing time
  - Produces deterministic signatures for a given message and private key pair

# EC-Schnorr-DCRv0 Specification

See the README.md file for the specific details of the signing and verification
algorithm as well as the signature serialization format.

# Future Design Considerations

It is worth noting that there are some additional optimizations and
modifications that have been identified since the introduction of
EC-Schnorr-DCRv0 that can be made to further harden security for multi-party and
threshold signature use cases as well provide the opportunity for faster
signature verification with a sufficiently optimized implementation.

However, the v0 scheme is used in the existing consensus rules and any changes
to the signature scheme would invalidate existing uses.  Therefore changes in
this regard will need to come in the form of a v1 signature scheme and be
accompanied by the necessary consensus updates.

# Schnorr use in Decred

At the time of this writing, Schnorr signatures are not yet in widespread use on
the Decred network, largely due to the current lack of support in wallets and
infrastructure for secure multi-party and threshold signatures.

However, the consensus rules and scripting engine supports the necessary
primitives and given many of the beneficial properties of Schnorr signatures, a
good goal is to work towards providing the additional infrastructure to increase
their usage.
*/
package schnorr
