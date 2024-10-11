# ADR 010: Crypto Changes

## Context

Tendermint is a cryptographic protocol that uses and composes a variety of cryptographic primitives.

After nearly 4 years of development, Tendermint has recently undergone multiple security reviews to search for vulnerabilities and to assess the the use and composition of cryptographic primitives.

### Hash Functions

Tendermint uses RIPEMD160 universally as a hash function, most notably in its Merkle tree implementation.

RIPEMD160 was chosen because it provides the shortest fingerprint that is long enough to be considered secure (ie. birthday bound of 80-bits).
It was also developed in the open academic community, unlike NSA-designed algorithms like SHA256.

That said, the cryptographic community appears to unanimously agree on the security of SHA256. It has become a universal standard, especially now that SHA1 is broken, being required in TLS connections and having optimized support in hardware.

### Merkle Trees

Tendermint uses a simple Merkle tree to compute digests of large structures like transaction batches
and even blockchain headers. The Merkle tree length prefixes byte arrays before concatenating and hashing them.
It uses RIPEMD160.

### Addresses

ED25519 addresses are computed using the RIPEMD160 of the Amino encoding of the public key.
RIPEMD160 is generally considered an outdated hash function, and is much slower
than more modern functions like SHA256 or Blake2.

### Authenticated Encryption

Tendermint P2P connections use authenticated encryption to provide privacy and authentication in the communications.
This is done using the simple Station-to-Station protocol with the NaCL Ed25519 library.

While there have been no vulnerabilities found in the implementation, there are some concerns:

- NaCL uses Salsa20, a not-widely used and relatively out-dated stream cipher that has been obsoleted by ChaCha20
- Connections use RIPEMD160 to compute a value that is used for the encryption nonce with subtle requirements on how it's used

## Decision

### Hash Functions

Use the first 20-bytes of the SHA256 hash instead of RIPEMD160 for everything

### Merkle Trees

TODO

### Addresses

Compute ED25519 addresses as the first 20-bytes of the SHA256 of the raw 32-byte public key

### Authenticated Encryption

Make the following changes:

- Use xChaCha20 instead of xSalsa20 - https://github.com/tendermint/tendermint/issues/1124
- Use an HKDF instead of RIPEMD160 to compute nonces - https://github.com/tendermint/tendermint/issues/1165

## Status

Implemented

## Consequences

### Positive

- More modern and standard cryptographic functions with wider adoption and hardware acceleration

### Negative

- Exact authenticated encryption construction isn't already provided in a well-used library

### Neutral

## References
