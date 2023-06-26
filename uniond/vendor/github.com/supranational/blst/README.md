[![Build Status](https://travis-ci.com/supranational/blst.svg?branch=master)](https://travis-ci.com/github/supranational/blst) [![Actions status](https://github.com/supranational/blst/workflows/build/badge.svg)](https://github.com/supranational/blst/actions) [![Total alerts](https://img.shields.io/lgtm/alerts/g/supranational/blst.svg?logo=lgtm&logoWidth=18)](https://lgtm.com/projects/g/supranational/blst/alerts/)
<div align="left">
  <img src=blst_logo_small.png>
</div>

# blst
blst (pronounced 'blast') is a BLS12-381 signature library focused on performance and security. It is written in C and assembly.

## Table of Contents

  * [Status](#status)
  * [General notes on implementation](#general-notes-on-implementation)
  * [Platform and Language Compatibility](#platform-and-language-compatibility)
  * [API](#api)
  * [Introductory Tutorial](#introductory-tutorial)
    + [Public Keys and Signatures](#public-keys-and-signatures)
    + [Signature Verification](#signature-verification)
    + [Signature Aggregation](#signature-aggregation)
    + [Serialization Format](#serialization-format)
  * [Build](#build)
    + [C static library](#c-static-library)
  * [Language-specific notes](#language-specific-notes)
    + [Go](#go)
    + [Rust](#rust)
  * [Repository Structure](#repository-structure)
  * [Performance](#performance)
  * [License](#license)

## Status
**This library is under active development**

An initial audit of this library was conducted by NCC Group in January 2021 and can be found [here](https://research.nccgroup.com/wp-content/uploads/2021/01/NCC_Group_EthereumFoundation_ETHF002_Report_2021-01-20_v1.0.pdf).

Formal verification of this library by Galois is on-going and can be found [here](https://github.com/GaloisInc/BLST-Verification).

This library is compliant with the following IETF draft specifications:
- [IETF BLS Signature V4](https://tools.ietf.org/html/draft-irtf-cfrg-bls-signature)
- [IETF Hash-to-Curve V13](https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve)

The serialization formatting is implemented according to [the ZCash definition](#serialization-format).

## General notes on implementation
The goal of the blst library is to provide a foundational component for applications and other libraries that require high performance and formally verified BLS12-381 operations. With that in mind some decisions are made to maximize the public good beyond BLS12-381. For example, the field operations are optimized for general 384-bit usage, as opposed to tuned specifically for the 381-bit BLS12-381 curve parameters. With the formal verification of these foundational components, we believe they can provide a reliable building block for other curves that would like high performance and an extra element of security.

The library deliberately abstains from dealing with memory management and multi-threading, with the rationale that these ultimately belong in language-specific bindings. Another responsibility that is left to application is random number generation. All this in the name of run-time neutrality, which makes integration into more stringent environments like Intel SGX or ARM TrustZone trivial.

## Platform and Language Compatibility

This library primarily supports x86_64 and ARM64 hardware platforms, and Linux, Mac, and Windows operating systems. But it does have a portable replacement for the assembly modules, which can be compiled for a plethora of other platforms. Problem reports for these will be considered and are likely to be addressed.

This repository includes explicit bindings for:
- [Go](bindings/go)
- [Rust](bindings/rust)

Unless deemed appropriate to implement, bindings for other languages will be provided using [SWIG](http://swig.org). Proof-of-concept scripts are available for:
- [Python](bindings/python)
- [Java](bindings/java)
- [Node.js](bindings/node.js)

## API

The blst API is defined in the C header [bindings/blst.h](bindings/blst.h). The API can be categorized as follows, with some example operations:
- Field Operations (add, sub, mul, neg, inv, to/from Montgomery)
- Curve Operations (add, double, mul, to/from affine, group check)
- Intermediate (hash to curve, pairing, serdes)
- BLS12-381 signature (sign, verify, aggregate)

Note: there is also an auxiliary header file, [bindings/blst_aux.h](bindings/blst_aux.h), that is used as a staging area for experimental interfaces that may or may not get promoted to blst.h.

## Introductory Tutorial

Programming is understanding, and understanding implies mastering the lingo. So we have a pair of additive groups being mapped to multiplicative one... What does it mean? Well, this tutorial is not about explaining that, but rather about making the connection between what you're supposed to know about [pairing-based cryptography](https://en.wikipedia.org/wiki/Pairing-based_cryptography) and the interface provided by the library.

### Public Keys and Signatures

We have two elliptic curves, E1 and E2, points on which are contained in `blst_p1` and `blst_p2`, or `blst_p1_affine` and `blst_p2_affine` structures. Elements in the multiplicative group are held in a `blst_fp12` structure. One of the curves, or more specifically, a subset of points that form a cyclic group, is chosen for public keys, and another, for signatures. The choice is denoted by the subroutines' suffixes, `_pk_in_g1` or `_pk_in_g2`. The most common choice appears to be the former, that is, `blst_p1` for public keys, and `blst_p2` for signatures. But it all starts with a secret key...

The secret key is held in a 256-bit `blst_scalar` structure which can be instantiated with either [`blst_keygen`](https://tools.ietf.org/html/draft-irtf-cfrg-bls-signature#section-2.3), or deserialized with `blst_scalar_from_bendian` or `blst_scalar_from_lendian` from a previously serialized byte sequence. It shouldn't come as surprise that there are two uses for a secret key:

- generating the associated public key, either with `blst_sk_to_pk_in_g1` or `blst_sk_to_pk_in_g2`;
- performing a sign operation, either with `blst_sign_pk_in_g1` or `blst_sign_pk_in_g2`;

As for signing, unlike what your intuition might suggest, `blst_sign_*` doesn't sign a message, but rather a point on the corresponding elliptic curve. You can obtain this point from a message by calling `blst_hash_to_g2` or `blst_encode_to_g2` (see the [IETF hash-to-curve](https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve#section-3) draft for distinction). Another counter-intuitive aspect is the apparent g1 vs. g2 naming mismatch, in the sense that `blst_sign_pk_in_g1` accepts output from `blst_hash_to_g2`, and `blst_sign_pk_in_g2` accepts output from `blst_hash_to_g1`. This is because, as you should recall, public keys and signatures come from complementary groups.

Now that you have a public key and signature, as points on corresponding elliptic curves, you can serialize them with `blst_p1_serialize`/`blst_p1_compress` and `blst_p2_serialize`/`blst_p2_compress` and send the resulting byte sequences over the network for deserialization/uncompression and verification.

### Signature Verification

Even though there are "single-shot" `blst_core_verify_pk_in_g1` and `blst_core_verify_pk_in_g2`, you should really familiarize yourself with the more generalized pairing interface. `blst_pairing` is an opaque structure, and the only thing you know about it is `blst_pairing_sizeof`, which is how much memory you're supposed to allocate for it. In order to verify an aggregated signature for a set of public keys and messages, or just one[!], you would:
```
blst_pairing_init(ctx, hash_or_encode, domain_separation_tag);
blst_pairing_aggregate_pk_in_g1(ctx, PK[0], aggregated_signature, message[0]);
blst_pairing_aggregate_pk_in_g1(ctx, PK[1], NULL, message[1]);
...
blst_pairing_commit(ctx);
result = blst_pairing_finalverify(ctx, NULL);
```
**The essential point to note** is that it's the caller's responsibility to ensure that public keys are group-checked with `blst_p1_affine_in_g1`. This is because it's a relatively expensive operation and it's naturally assumed that the application would cache the check's outcome. Signatures are group-checked internally. Not shown in the pseudo-code snippet above, but `aggregate` and `commit` calls return `BLST_ERROR` denoting success or failure in performing the operation. Call to `finalverify`, on the other hand, returns boolean.

Another, potentially more useful usage pattern is:
```
blst_p2_affine_in_g2(signature);
blst_aggregated_in_g2(gtsig, signature);
blst_pairing_init(ctx, hash_or_encode, domain_separation_tag);
blst_pairing_aggregate_pk_in_g1(ctx, PK[0], NULL, message[0]);
blst_pairing_aggregate_pk_in_g1(ctx, PK[1], NULL, message[1]);
...
blst_pairing_commit(ctx);
result = blst_pairing_finalverify(ctx, gtsig);
```
What is useful about it is that `aggregated_signature` can be handled in a separate thread. And while we are at it, aggregate calls can also be executed in different threads. This naturally implies that each thread will operate on its own `blst_pairing` context, which will have to be combined with `blst_pairing_merge` as threads join.

### Signature Aggregation

Aggregation is a trivial operation of performing point additions, with `blst_p2_add_or_double_affine` or `blst_p1_add_or_double_affine`. Note that the accumulator is a non-affine point.

---

That's about what you need to know to get started with nitty-gritty of actual function declarations.

### Serialization Format

From the ZCash BLS12-381 specification

* Fq elements are encoded in big-endian form. They occupy 48 bytes in this form.
* Fq2 elements are encoded in big-endian form, meaning that the Fq2 element c0 + c1 * u is represented by the Fq element c1 followed by the Fq element c0. This means Fq2 elements occupy 96 bytes in this form.
* The group G1 uses Fq elements for coordinates. The group G2 uses Fq2 elements for coordinates.
* G1 and G2 elements can be encoded in uncompressed form (the x-coordinate followed by the y-coordinate) or in compressed form (just the x-coordinate). G1 elements occupy 96 bytes in uncompressed form, and 48 bytes in compressed form. G2 elements occupy 192 bytes in uncompressed form, and 96 bytes in compressed form.

The most-significant three bits of a G1 or G2 encoding should be masked away before the coordinate(s) are interpreted. These bits are used to unambiguously represent the underlying element:

* The most significant bit, when set, indicates that the point is in compressed form. Otherwise, the point is in uncompressed form.
* The second-most significant bit indicates that the point is at infinity. If this bit is set, the remaining bits of the group element's encoding should be set to zero.
* The third-most significant bit is set if (and only if) this point is in compressed form _and_ it is not the point at infinity _and_ its y-coordinate is the lexicographically largest of the two associated with the encoded x-coordinate.

## Build
The build process is very simple and only requires a C complier. It's integrated into the Go and Rust ecosystems, so that respective users would go about as they would with any other external module. Otherwise, a binary library would have to be compiled.

### C static library
A static library called libblst.a can be built in the current working directory of the user's choice:

Linux, Mac, and Windows (in MinGW or Cygwin environments)
```
/some/where/build.sh
```

Windows (Visual C)
```
\some\where\build.bat
```

If final application crashes with an "illegal instruction" exception [after copying to another system], pass <nobr>`-D__BLST_PORTABLE__`</nobr> on `build.sh` command line. If you don't use build.sh, complement the `CFLAGS` environment variable with the said command line option. If you compile a Go application, you will need to modify the `CGO_CFLAGS` variable instead. And if you compile a Rust application, you can pass <nobr>`--features portable`</nobr> to `cargo build`. Alternatively, if you compile on an older Intel system, but will execute application on a newer one, consider instead passing <nobr>`--features force-adx`</nobr> for better performance.

## Language-specific notes

### [Go](bindings/go)
There are two primary modes of operation that can be chosen based on type definitions in the application.

For minimal-pubkey-size operations:
```
type PublicKey = blst.P1Affine
type Signature = blst.P2Affine
type AggregateSignature = blst.P2Aggregate
type AggregatePublicKey = blst.P1Aggregate
```

For minimal-signature-size operations:
```
type PublicKey = blst.P2Affine
type Signature = blst.P1Affine
type AggregateSignature = blst.P1Aggregate
type AggregatePublicKey = blst.P2Aggregate
```

For more details see the Go binding [readme](bindings/go/README.md).

### [Rust](bindings/rust)
[`blst`](https://crates.io/crates/blst) is the Rust binding crate.

To use min-pk version:
```
use blst::min_pk::*;
```

To use min-sig version:
```
use blst::min_sig::*;
```

For more details see the Rust binding [readme](bindings/rust/README.md).

## Repository Structure

**Root** - Contains various configuration files, documentation, licensing, and a build script
* **Bindings** - Contains the files that define the blst interface
    * blst.h - provides C API to blst library
    * blst_aux.h - contains experimental functions not yet committed for long-term maintenance
    * blst.hpp - provides foundational class-oriented C++ interface to blst library
    * blst.swg - provides SWIG definitions for creating blst bindings for other languages, such as Java and Python
    * **Go** - folder containing Go bindings for blst, including tests and benchmarks
    * **Java** - folder containing an example of how to use SWIG Java bindings for blst
    * **Node.js** - folder containing an example of how to use SWIG Javascript bindings for blst
    * **Python** - folder containing an example of how to use SWIG Python bindings for blst
    * **Rust** - folder containing Rust bindings for blst, including tests and benchmarks
    * **Vectors**
        * **Hash_to_curve**: folder containing test for hash_to_curve from IETF specification
* **Src** - folder containing C code for lower level blst functions such as field operations, extension field operations, hash-to-field, and more
    * **Asm** - folder containing Perl scripts that are used to generate assembly code for different hardware platforms including x86 with ADX instructions, x86 without ADX instructions, and ARMv8, and [ABI](https://en.wikipedia.org/wiki/Application_binary_interface)[1]
* **Build** - this folder containing a set of pre-generated assembly files for a variety of operating systems and maintenance scripts.
    * **Coff** - assembly code for use on Window systems with GNU toolchain
    * **Elf** - assembly code for use on Unix systems
    * **Mach-o** - assembly code for use on Apple operating systems
    * **Win64** - assembly code for use on Windows systems with Microsoft toolchain

[1]: See [refresh.sh](build/refresh.sh) for usage. This method allows for simple reuse of optimized assembly across various platforms with minimal effort.

## Performance
Currently both the [Go](bindings/go) and [Rust](bindings/rust) bindings provide benchmarks for a variety of signature related operations.

## License
The blst library is licensed under the [Apache License Version 2.0](LICENSE) software license.
