# ADR 079: Ed25519 Verification

## Changelog

- 2020-08-21: Initial RFC
- 2021-02-11: Migrate RFC to tendermint repo (Originally [RFC 003](https://github.com/tendermint/spec/pull/144))

## Author(s)

- Marko (@marbar3778)

## Context

Ed25519 keys are the only supported key types for Tendermint validators currently. Tendermint-Go wraps the ed25519 key implementation from the go standard library. As more clients are implemented to communicate with the canonical Tendermint implementation (Tendermint-Go) different implementations of ed25519 will be used. Due to [RFC 8032](https://www.rfc-editor.org/rfc/rfc8032.html) not guaranteeing implementation compatibility, Tendermint clients must to come to an agreement of how to guarantee implementation compatibility. [Zcash](https://z.cash/) has multiple implementations of their client and have identified this as a problem as well. The team at Zcash has made a proposal to address this issue, [Zcash improvement proposal 215](https://zips.z.cash/zip-0215).

## Proposal

- Tendermint-Go would adopt [hdevalence/ed25519consensus](https://github.com/hdevalence/ed25519consensus).
    - This library is implements `ed25519.Verify()` in accordance to zip-215. Tendermint-go will continue to use `crypto/ed25519` for signing and key generation.

- Tendermint-rs would adopt [ed25519-zebra](https://github.com/ZcashFoundation/ed25519-zebra)
    - related [issue](https://github.com/informalsystems/tendermint-rs/issues/355)

Signature verification is one of the major bottlenecks of Tendermint-go, batch verification can not be used unless it has the same consensus rules, ZIP 215 makes verification safe in consensus critical areas.

This change constitutes a breaking changes, therefore must be done in a major release. No changes to validator keys or operations will be needed for this change to be enabled.

This change has no impact on signature aggregation. To enable this signature aggregation Tendermint will have to use different signature schema (Schnorr, BLS, ...). Secondly, this change will enable safe batch verification for the Tendermint-Go client. Batch verification for the rust client is already supported in the library being used.

As part of the acceptance of this proposal it would be best to contract or discuss with a third party the process of conducting a security review of the go library.

## Status

Accepted (implicitly tracked in
[\#9186](https://github.com/tendermint/tendermint/issues/9186))

## Consequences

### Positive

- Consistent signature verification across implementations
- Enable safe batch verification

### Negative

#### Tendermint-Go

- Third_party dependency
    - library has not gone through a security review.
    - unclear maintenance schedule
- Fragmentation of the ed25519 key for the go implementation, verification is done using a third party library while the rest
  uses the go standard library

### Neutral

## References

[It’s 255:19AM. Do you know what your validation criteria are?](https://hdevalence.ca/blog/2020-10-04-its-25519am)
