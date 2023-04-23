ecdsa
=====

[![Build Status](https://github.com/decred/dcrd/workflows/Build%20and%20Test/badge.svg)](https://github.com/decred/dcrd/actions)
[![ISC License](https://img.shields.io/badge/license-ISC-blue.svg)](http://copyfree.org)
[![GoDoc](https://img.shields.io/badge/godoc-reference-blue.svg)](https://pkg.go.dev/github.com/decred/dcrd/dcrec/secp256k1/v4/ecdsa)

Package ecdsa provides secp256k1-optimized ECDSA signing and verification.

This package provides data structures and functions necessary to produce and
verify deterministic canonical signatures in accordance with RFC6979 and
BIP0062, optimized specifically for the secp256k1 curve using the Elliptic Curve
Digital Signature Algorithm (ECDSA), as defined in FIPS 186-3.  See
https://www.secg.org/sec2-v2.pdf for details on the secp256k1 standard.

It also provides functions to parse and serialize the ECDSA signatures with the
more strict Distinguished Encoding Rules (DER) of ISO/IEC 8825-1 and some
additional restrictions specific to secp256k1.

In addition, it supports a custom "compact" signature format which allows
efficient recovery of the public key from a given valid signature and message
hash combination.

A comprehensive suite of tests is provided to ensure proper functionality.

## ECDSA use in Decred

At the time of this writing, ECDSA signatures are heavily used for proving coin
ownership in Decred as the vast majority of transactions consist of what is
effectively transferring ownership of coins to a public key associated with a
private key only known to the recipient of the coins along with an encumbrance
that requires an ECDSA signature that proves the new owner possesses the private
key without actually revealing it.

## Installation and Updating

This package is part of the `github.com/decred/dcrd/dcrec/secp256k1/v4` module.
Use the standard go tooling for working with modules to incorporate it.

## Examples

* [Sign Message](https://pkg.go.dev/github.com/decred/dcrd/dcrec/secp256k1/v4/ecdsa#example-package-SignMessage)  
  Demonstrates signing a message with a secp256k1 private key that is first
  parsed from raw bytes and serializing the generated signature.

* [Verify Signature](https://pkg.go.dev/github.com/decred/dcrd/dcrec/secp256k1/v4/ecdsa#example-Signature.Verify)  
  Demonstrates verifying a secp256k1 signature against a public key that is
  first parsed from raw bytes.  The signature is also parsed from raw bytes.

## License

Package ecdsa is licensed under the [copyfree](http://copyfree.org) ISC License.
