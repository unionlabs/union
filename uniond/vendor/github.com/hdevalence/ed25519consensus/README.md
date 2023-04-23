# Ed25519 for consensus-critical contexts

This library provides an Ed25519 implementation with validation rules intended
for consensus-critical contexts.

Ed25519 signatures are widely used in consensus-critical contexts (e.g.,
blockchains), where different nodes must agree on whether or not a given
signature is valid.  However, Ed25519 does not clearly define criteria for
signature validity, and even standards-conformant implementations are not
required to agree on whether a signature is valid.

Different Ed25519 implementations may not (and in practice, do not) agree on
validation criteria in subtle edge cases.   This poses a double risk to the use
of Ed25519 in consensus-critical contexts.  First, the presence of multiple
Ed25519 implementations may open the possibility of consensus divergence.
Second, even when a single implementation is used, the protocol implicitly
includes that particular version's validation criteria as part of the consensus
rules.  However, if the implementation is not intended to be used in
consensus-critical contexts, it may change validation criteria between releases.

For instance, the initial implementation of Zcash consensus in zcashd inherited
validity criteria from a then-current version of libsodium (1.0.15). Due to a
bug in libsodium, this was different from the intended criteria documented in
the Zcash protocol specification 3 (before the specification was changed to
match libsodium 1.0.15 in specification version 2020.1.2). Also, libsodium
never guaranteed stable validity criteria, and changed behavior in a later
point release. This forced zcashd to use an older version of the library before
eventually patching a newer version to have consistent validity criteria. To be
compatible, Zebra had to implement a special library, ed25519-zebra to provide
Zcash-flavored Ed25519, attempting to match libsodium 1.0.15 exactly. And the
initial attempt to implement ed25519-zebra was also incompatible, because it
precisely matched the wrong compile-time configuration of libsodium.

This problem is fixed by [ZIP215], a specification of a precise set of
validation criteria for Ed25519 signatures.
This repository contains a fork of Go's `crypto/ed25519` package with support
for [ZIP215] verification.

Note that the ZIP215 rules ensure that individual and batch verification are
guaranteed to give the same results, so unlike `ed25519.Verify`, `ed25519consensus.Verify` is
compatible with batch verification (though this is not yet implemented by this
library).

[ZIP215]: https://zips.z.cash/zip-0215
