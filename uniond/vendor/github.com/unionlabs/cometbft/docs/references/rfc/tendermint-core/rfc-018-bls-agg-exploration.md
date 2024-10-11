# RFC 018: BLS Signature Aggregation Exploration

## Changelog

- 01-April-2022: Initial draft (@williambanfield).
- 15-April-2022: Draft complete (@williambanfield).

## Abstract

## Background

### Glossary

The terms that are attached to these types of cryptographic signing systems
become confusing quickly. Different sources appear to use slightly different
meanings of each term and this can certainly add to the confusion. Below is
a brief glossary that may be helpful in understanding the discussion that follows.

- **Short Signature**: A signature that does not vary in length with the
number of signers.
- **Multi-Signature**: A signature generated over a single message
where, given the message and signature, a verifier is able to determine that
all parties signed the message. May be short or may vary with the number of signers.
- **Aggregated Signature**: A _short_ signature generated over messages with
possibly different content where, given the messages and signature, a verifier
should be able to determine that all the parties signed the designated messages.
- **Threshold Signature**: A _short_ signature generated from multiple signers
where, given a message and the signature, a verifier is able to determine that
a large enough share of the parties signed the message. The identities of the
parties that contributed to the signature are not revealed.
- **BLS Signature**: An elliptic-curve pairing-based signature system that
has some nice properties for short multi-signatures. May stand for
_Boneh-Lynn-Schacham_ or _Barreto-Lynn-Scott_ depending on the context. A
BLS signature is type of signature scheme that is distinct from other forms
of elliptic-curve signatures such as ECDSA and EdDSA.
- **Interactive**: Cryptographic scheme where parties need to perform one or
more request-response cycles to produce the cryptographic material. For
example, an interactive signature scheme may require the signer and the
verifier to cooperate to create and/or verify the signature, rather than a
signature being created ahead of time.
- **Non-interactive**: Cryptographic scheme where parties do not need to
perform any request-response cycles to produce the cryptographic material.

### Brief notes on pairing-based elliptic-curve cryptography

Pairing-based elliptic-curve cryptography is quite complex and relies on several
types of high-level math. Cryptography, in general, relies on being able to find
problems with an asymmetry between the difficulty of calculating the solution
and verifying that a given solution is correct.

Pairing-based cryptography works by operating on mathematical functions that
satisfy the property of **bilinear mapping**. This property is satisfied for
functions `e` with values `P`, `Q`, `R` and `S` where `e(P, Q + R) = e(P, Q) * e(P, R)`
and `e(P + S, Q) = e(P, Q) * e(S, Q)`. The most familiar example of this is
exponentiation. Written in common notation, `g^P*(Q+R) = g^(P*Q) * g^(P*R)` for
some value `g`.

Pairing-based elliptic-curve cryptography creates a bilinear mapping using
elliptic curves over a finite field. With some original curve, you can define two groups,
`G1` and `G2` which are points of the original curve _modulo_ different values.
Finally, you define a third group `Gt`, where points from `G1` and `G2` satisfy
the property of bilinearity with `Gt`. In this scheme, the function `e` takes
as inputs points in `G1` and `G2` and outputs values in `Gt`. Succinctly, given
some point `P` in `G1` and some point `Q` in `G1`, `e(P, Q) = C` where `C` is in `Gt`.
You can efficiently compute the mapping of points in `G1` and `G2` into `Gt`,
but you cannot efficiently determine what points were summed and paired to
produce the value in `Gt`.

Functions are then defined to map digital signatures, messages, and keys into
and out of points of `G1` or `G2` and signature verification is the process
of calculating if a set of values representing a message, public key, and digital
signature produce the same value in `Gt` through `e`.

Signatures can be created as either points in `G1` with public keys being
created as points in `G2` or vice versa. For the case of BLS12-381, the popular
curve used, points in `G1` are represented with 48 bytes and points in `G2` are
represented with 96 bytes. It is up to the implementer of the cryptosystem to
decide which should be larger, the public keys or the signatures.

BLS signatures rely on pairing-based elliptic-curve cryptography to produce
various types of signatures. For a more in-depth but still high level discussion
pairing-based elliptic-curve cryptography, see Vitalik Buterin's post on
[Exploring Elliptic Curve Pairings][vitalik-pairing-post]. For much more in
depth discussion, see the specific paper on BLS12-381, [Short signatures from
 the Weil Pairing][bls-weil-pairing] and
[Compact Multi-Signatures for Smaller Blockchains][multi-signatures-smaller-blockchains].

### Adoption

BLS signatures have already gained traction within several popular projects.

- Algorand is working on an implementation.
- [Zcash][zcash-adoption] has adopted BLS12-381 into the protocol.
- [Ethereum 2.0][eth-2-adoption] has adopted BLS12-381 into the protocol.
- [Chia Network][chia-adoption] has adopted BLS for signing blocks.
- [Ostracon][line-ostracon-pr], a fork of Tendermint has adopted BLS for signing blocks.

### What systems may be affected by adding aggregated signatures?

#### Gossip

Gossip could be updated to aggregate vote signatures during a consensus round.
This appears to be of frankly little utility. Creating an aggregated signature
incurs overhead, so frequently re-aggregating may incur a significant
overhead. How costly this is is still subject to further investigation and
performance testing.

Even if vote signatures were aggregated before gossip, each validator would still
need to receive and verify vote extension data from each (individual) peer validator in
order for consensus to proceed. That displaces any advantage gained by aggregating signatures across the vote message in the presence of vote extensions.

#### Block Creation

When creating a block, the proposer may create a small set of short
multi-signatures and attach these to the block instead of including one
signature per validator.

#### Block Verification

Currently, we verify each validator signature using the public key associated
with that validator.  With signature aggregation, verification of blocks would
not verify many signatures individually, but would instead check the (single)
multi-signature using the public keys stored by the validator. This would also
require a mechanism for indicating which validators are included in the
aggregated signature.

#### IBC Relaying

IBC would no longer need to transmit a large set of signatures when
updating state. These state updates do not happen for every IBC packet, only
when changing an IBC light client's view of the counterparty chain's state.
General [IBC packets][ibc-packet] only contain enough information to correctly
route the data to the counterparty chain.

IBC does persist commit signatures to the chain in these `MsgUpdateClient`
message when updating state. This message would no longer need the full set
of unique signatures and would instead only need one signature for all of the
data in the header.

Adding BLS signatures would create a new signature type that must be
understood by the IBC module and by the relayers. For some operations, such
as state updates, the set of data written into the chain and received by the
IBC module could be slightly smaller.

## Discussion

### What are the proposed benefits to aggregated signatures?

#### Reduce Block Size

At the moment, a commit contains a 64-byte (512-bit) signature for each validator
that voted for the block. For the Cosmos Hub, which has 175 validators in the
active set, this amounts to about 11 KiB per block. That gives an upper bound of
around 113 GiB over the lifetime of the chain's 10.12M blocks. (Note, the Hub has
increased the number of validators in the active set over time so the total
signature size over the history of the chain is likely somewhat less than that).

Signature aggregation would only produce two signatures for the entire block.
One for the yeas and one for the nays. Each BLS aggregated signature is 48
bytes, per the [IETF standard of BLS signatures][bls-ietf-ecdsa-compare].
Over the lifetime of the same Cosmos Hub chain, that would amount to about 1
GB, a savings of 112 GB. While that is a large factor of reduction it's worth
bearing in mind that, at [GCP's cost][gcp-storage-pricing] of $.026 USD per GB,
that is a total savings of around $2.50 per month.

#### Reduce Signature Creation and Verification Time

From the [IETF draft standard on BLS Signatures][bls-ietf], BLS signatures can be
created in 370 microseconds and verified in 2700 microseconds. Our current
[Ed25519 implementation][voi-ed25519-perf] was benchmarked locally to take
13.9 microseconds to produce a signature and 2.03 milliseconds to batch verify
128 signatures, which is slightly fewer than the 175 in the Hub. blst, a popular
implementation of BLS signature aggregation was benchmarked to perform verification
on 100 signatures in 1.5 milliseconds [when run locally][blst-verify-bench]
on an 8 thread machine and pre-aggregated public keys. It is worth noting that
the `ed25519` library verification time grew steadily with the number of signatures,
whereas the bls library verification time remains constant. This is because the
number of operations used to verify a signature does not grow at all with the
number of signatures included in the aggregate signature (as long as the signers
signed over the same message data as is the case in Tendermint).

It is worth noting that this would also represent a _degredation_ in signature
verification time for chains with small validator sets. When batch verifying
only 32 signatures, our ed25519 library takes .57 milliseconds, whereas BLS
would still require the same 1.5 milliseconds.

For massive validator sets, blst dominates, taking the same 1.5 milliseconds to
check an aggregated signature from 1024 validators versus our ed25519 library's
13.066 milliseconds to batch verify a set of that size.

#### Reduce Light-Client Verification Time

The light client aims to be a faster and lighter-weight way to verify that a
block was voted on by a Tendermint network. The light client fetches
Tendermint block headers and commit signatures, performing public key
verification to ensure that the associated validator set signed the block.
Reducing the size of the commit signature would allow the light client to fetch
block data more quickly.

Additionally, the faster signature verification times of BLS signatures mean
that light client verification would proceed more quickly.

However, verification of an aggregated signature is all-or-nothing. The verifier
cannot check that some singular signer had a signature included in the block.
Instead, the verifier must use all public keys to check if some signature
was included. This does mean that any light client implementation must always
be able to fetch all public keys for any height instead of potentially being
able to check if some singular validator's key signed the block.

#### Reduce Gossip Bandwidth

##### Vote Gossip

It is possible to aggregate subsets of signatures during voting, so that the
network need not gossip all _n_ validator signatures to all _n_ validators.
Theoretically, subsets of the signatures could be aggregated during consensus
and vote messages could carry those aggregated signatures. Implementing this
would certainly increase the complexity of the gossip layer but could possibly
reduce the total number of signatures required to be verified by each validator.

##### Block Gossip

A reduction in the block size as a result of signature aggregation would
naturally lead to a reduction in the bandwidth required to gossip a block.
Each validator would only send and receive the smaller aggregated signatures
instead of the full list of multi-signatures as we have them now.

### What are the drawbacks to aggregated signatures?

#### Heterogeneous key types cannot be aggregated

Aggregation requires a specific signature algorithm, and our legacy signing schemes
cannot be aggregated. In practice, this means that aggregated signatures could
be created for a subset of validators using BLS signatures, and validators
with other key types (such as Ed25519) would still have to be be separately
propagated in blocks and votes.

#### Many HSMs do not support aggregated signatures

**Hardware Signing Modules** (HSM) are a popular way to manage private keys.
They provide additional security for key management and should be used when
possible for storing highly sensitive private key material.

Below is a list of popular HSMs along with their support for BLS signatures.

- YubiKey
    - [No support][yubi-key-bls-support]
- Amazon Cloud HSM
    - [No support][cloud-hsm-support]
- Ledger
    - [Lists support for the BLS12-381 curve][ledger-bls-announce]

I cannot find support listed for Google Cloud, although perhaps it exists.

## Feasibility of implementation

This section outlines the various hurdles that would exist to implementing BLS
signature aggregation into Tendermint. It aims to demonstrate that we _could_
implement BLS signatures but that it would incur risk and require breaking changes for a
reasonably unclear benefit.

### Can aggregated signatures be added as soft-upgrades?

In my estimation, yes. With the implementation of proposer-based timestamps,
all validators now produce signatures on only one of two messages:

1. A [CanonicalVote][canonical-vote-proto] where the BlockID is the hash of the block or
2. A `CanonicalVote` where the `BlockID` is nil.

The block structure can be updated to perform hashing and validation in a new
way as a soft upgrade. This would look like adding a new section to the [Block.Commit][commit-proto] structure
alongside the current `Commit.Signatures` field. This new field, tentatively named
`AggregatedSignature` would contain the following structure:

```proto
message AggregatedSignature {
  // yeas is a BitArray representing which validators in the active validator
  // set issued a 'yea' vote for the block.
  tendermint.libs.bits.BitArray yeas = 1;

  // absent is a BitArray representing which validators in the active
  // validator set did not issue votes for the block.
  tendermint.libs.bits.BitArray absent = 2;

  // yea_signature is an aggregated signature produced from all of the vote
  // signatures for the block.
  repeated bytes yea_signature = 3;

  // nay_signature is an aggregated signature produced from all of the vote
  // signatures from votes for 'nil' for this block.
  // nay_signature should be made from all of the validators that were both not
  // in the 'yeas' BitArray and not in the 'absent' BitArray.
  repeated bytes nay_signature = 4;
}
```

Adding this new field as a soft upgrade would mean hashing this data structure
into the blockID along with the old `Commit.Signatures` when both are present
as well as ensuring that the voting power represented in the new
`AggregatedSignature` and `Signatures` field was enough to commit the block
during block validation. One can certainly imagine other possible schemes for
implementing this but the above should serve as a simple enough proof of concept.

### Implementing vote-time and commit-time signature aggregation separately

Implementing aggregated BLS signatures as part of the block structure can easily be
achieved without implementing any 'vote-time' signature aggregation.
The block proposer would gather all of the votes, complete with signatures,
as it does now, and produce a set of aggregate signatures from all of the
individual vote signatures.

Implementing 'vote-time' signature aggregation cannot be achieved without
also implementing commit-time signature aggregation. This is because such
signatures cannot be dis-aggregated into their constituent pieces. Therefore,
in order to implement 'vote-time' signature aggregation, we would need to
either first implement 'commit-time' signature aggregation, or implement both
'vote-time' signature aggregation while also updating the block creation and
verification protocols to allow for aggregated signatures.

### Updating IBC clients

In order for IBC clients to function, they must be able to perform light-client
verification of blocks on counterparty chains. Because BLS signatures are not
currently part of light-clients, chains that transmit messages over IBC
cannot update to using BLS signatures without their counterparties first
being upgraded to parse and verify BLS. If chains upgrade without their
counterparties first updating, they will lose the ability to interoperate with
non-updated chains.

### New attack surfaces

BLS signatures and signature aggregation comes with a new set of attack surfaces.
Additionally, it's not clear that all possible major attacks are currently known
on the BLS aggregation schemes since new ones have been discovered since the ietf
draft standard was written. The known attacks are manageable and are listed below.
Our implementation would need to prevent against these but this does not appear
to present a significant hurdle to implementation.

#### Rogue key attack prevention

Generating an aggregated signature requires guarding against what is called
a [rogue key attack][bls-ietf-terms]. A rogue key attack is one in which a
malicious actor can craft an _aggregate_ key that can produce signatures that
appear to include a signature from a private key that the malicious actor
does not actually know. In Tendermint terms, this would look like a Validator
producing a vote signed by both itself and some other validator where the other
validator did not actually produce the vote itself.

The main mechanisms for preventing this require that each entity prove that it
can can sign data with just their private key. The options involve either
ensuring that each entity sign a _different_ message when producing every
signature _or_ producing a [proof of possession][bls-ietf-pop] (PoP) when announcing
their key to the network.

A PoP is a message that demonstrates ownership of a private
key. A simple scheme for PoP is one where the entity announcing
its new public key to the network includes a digital signature over the bytes
of the public key generated using the associated private key. Everyone receiving
the public key and associated proof-of-possession can easily verify the
signature and be sure the entity owns the private key.

This PoP scheme suits the Tendermint use case quite well since
validator keys change infrequently so the associated PoPs would not be onerous
to produce, verify, and store. Using this scheme allows signature verification
to proceed more quickly, since all signatures are over identical data and
can therefore be checked using an aggregated public key instead of one at a
time, public key by public key.

#### Summing Zero Attacks

[Summing zero attacks][summing-zero-paper] are attacks that rely on using the '0' point of an
elliptic curve. For BLS signatures, if the point 0 is chosen as the private
key, then the 0 point will also always be the public key and all signatures
produced by the key will also be the 0 point. This is easy enough to
detect when verifying each signature individually.

However, because BLS signature aggregation creates an aggregated signature and
an aggregated public key, a set of colluding signers can create a pair or set
of signatures that are non-zero but which aggregate ("sum") to 0. The signatures that sum zero along with the
summed public key of the colluding signers will verify any message. This would
allow the colluding signers to sign any block or message with the same signature.
This would be reasonably easy to detect and create evidence for because, in
all other cases, the same signature should not verify more than message. It's
not exactly clear how such an attack would advantage the colluding validators
because the normal mechanisms of evidence gathering would still detect the
double signing, regardless of the signatures on both blocks being identical.

### Backwards Compatibility

Backwards compatibility is an important consideration for signature verification.
Specifically, it is important to consider whether chains using current versions
of IBC would be able to interact with chains adopting BLS.

Because the `Block` shared by IBC and Tendermint is produced and parsed using
protobuf, new structures can be added to the Block without breaking the
ability of legacy users to parse the new structure. Breaking changes between
current users of IBC and new Tendermint blocks only occur if data that is
relied upon by the current users is no longer included in the current fields.

For the case of BLS aggregated signatures, a new `AggregatedSignature` field
can therefore be added to the `Commit` field without breaking current users.
Current users will be broken when counterparty chains upgrade to the new version
and _begin using_ BLS signatures. Once counterparty chains begin using BLS
signatures, the BlockID hashes will include hashes of the `AggregatedSignature`
data structure that the legacy users will not be able to compute. Additionally,
the legacy software will not be able to parse and verify the signatures to
ensure that a supermajority of validators from the counterparty chain signed
the block.

### Library Support

Libraries for BLS signature creation are limited in number, although active
development appears to be ongoing. Cryptographic algorithms are difficult to
implement correctly and correctness issues are extremely serious and dangerous.
No further exploration of BLS should be undertaken without strong assurance of
a well-tested library with continuing support for creating and verifying BLS
signatures.

At the moment, there is one candidate, `blst`, that appears to be the most
mature and well vetted. While this library is undergoing continuing auditing
and is supported by funds from the Ethereum foundation, adopting a new cryptographic
library presents some serious risks. Namely, if the support for the library were
to be discontinued, Tendermint may become saddled with the requirement of supporting
a very complex piece of software or force a massive ecosystem-wide migration away
from BLS signatures.

This is one of the more serious reasons to avoid adopting BLS signatures at this
time. There is no gold standard library. Some projects look promising, but no
project has been formally verified with a long term promise of being supported
well into the future.

#### Go Standard Library

The Go Standard library has no implementation of BLS signatures.

#### BLST

[blst][blst], or 'blast' is an implementation of BLS signatures written in C
that provides bindings into Go as part of the repository. This library is
actively undergoing formal verification by Galois and previously received an
initial audit by NCC group, a firm I'd never heard of.

`blst` is [targeted for use in prysm][prysm-blst], the golang implementation of Ethereum 2.0.

#### Gnark-Crypto

[Gnark-Crypto][gnark] is a Go-native implementation of elliptic-curve pairing-based
cryptography. It is not audited and is documented as 'as-is', although
development appears to be active so formal verification may be forthcoming.

#### CIRCL

[CIRCL][circl] is a go-native implementation of several cryptographic primitives,
bls12-381 among them. The library is written and maintained by Cloudflare and
appears to receive frequent contributions. However, it lists itself as experimental
and urges users to take caution before using it in production.

### Added complexity to light client verification

Implementing BLS signature aggregation in Tendermint would pose issues for the
light client. The light client currently validates a subset of the signatures
on a block when performing the verification algorithm. This is no longer possible
with an aggregated signature. Aggregated signature verification is all-or-nothing.
The light client could no longer check that a subset of validators from some
set of validators is represented in the signature. Instead, it would need to create
a new aggregated key with all the stated signers for each height it verified where
the validator set changed.

This means that the speed advantages gained by using BLS cannot be fully realized
by the light client since the client needs to perform the expensive operation
of re-aggregating the public key. Aggregation is _not_ constant time in the
number of keys and instead grows linearly. When [benchmarked locally][blst-verify-bench-agg],
blst public key aggregation of 128 keys took 2.43 milliseconds. This, along with
the 1.5 milliseconds to verify a signature would raise light client signature
verification time to 3.9 milliseconds, a time above the previously mentioned
batch verification time using our ed25519 library of 2.0 milliseconds.

Schemes to cache aggregated subsets of keys could certainly cut this time down at the
cost of adding complexity to the light client.

### Added complexity to evidence handling

Implementing BLS signature aggregation in Tendermint would add complexity to
the evidence handling within Tendermint. Currently, the light client can submit
evidence of a fork attempt to the chain. This evidence consists of the set of
validators that double-signed, including their public keys, with the conflicting
block.

We can quickly check that the listed validators double signed by verifying
that each of their signatures are in the submitted conflicting block. A BLS
signature scheme would change this by requiring the light client to submit
the public keys of all of the validators that signed the conflicting block so
that the aggregated signature may be checked against the full signature set.
Again, aggregated signature verification is all-or-nothing, so without all of
the public keys, we cannot verify the signature at all. These keys would be
retrievable. Any party that wanted to create a fork would want to convince a
network that its fork is legitimate, so it would need to gossip the public keys.
This does not hamper the feasibility of implementing BLS signature aggregation
into Tendermint, but does represent yet another piece of added complexity to
the associated protocols.

## Open Questions

- _Q_: Can you aggregate Ed25519 signatures in Tendermint?
    - There is a suggested scheme in github issue [7892][suggested-ed25519-agg],
but additional rigor would be required to fully verify its correctness.

## Current Consideration

Adopting a signature aggregation scheme presents some serious risks and costs
to the Tendermint project. It requires multiple backwards-incompatible changes
to the code, namely a change in the structure of the block and a new backwards-incompatible
signature and key type. It risks adding a new signature type for which new attack
types are still being discovered _and_ for which no industry standard, battle-tested
library yet exists.

The gains boasted by this new signing scheme are modest: Verification time is
marginally faster and block sizes shrink by a few kilobytes. These are relatively
minor gains in exchange for the complexity of the change and the listed risks of the technology.
We should take a wait-and-see approach to BLS signature aggregation, monitoring
the up-and-coming projects and consider implementing it as the libraries and
standards develop.

### References

[line-ostracon-pr]: https://github.com/line/ostracon/pull/117
[gcp-storage-pricing]: https://cloud.google.com/storage/pricing#north-america_2
[yubi-key-bls-support]: https://github.com/Yubico/yubihsm-shell/issues/66
[cloud-hsm-support]: https://docs.aws.amazon.com/cloudhsm/latest/userguide/pkcs11-key-types.html
[bls-ietf]: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-04
[bls-ietf-terms]: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-04#section-1.3
[bls-ietf-pop]: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-04#section-3.3
[multi-signatures-smaller-blockchains]: https://eprint.iacr.org/2018/483.pdf
[zcash-adoption]: https://github.com/zcash/zcash/issues/2502
[chia-adoption]: https://github.com/Chia-Network/chia-blockchain#chia-blockchain
[bls-ietf-ecdsa-compare]: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-04#section-1.1
[voi-ed25519-perf]: https://github.com/williambanfield/curve25519-voi/blob/benchmark/primitives/ed25519/PERFORMANCE.txt#L79
[blst-verify-bench]: https://github.com/williambanfield/blst/blame/bench/bindings/go/PERFORMANCE.md#L9
[blst-verify-bench-agg]: https://github.com/williambanfield/blst/blame/bench/bindings/go/PERFORMANCE.md#L23
[vitalik-pairing-post]: https://medium.com/@VitalikButerin/exploring-elliptic-curve-pairings-c73c1864e627
[ledger-bls-announce]: https://www.ledger.com/first-ever-firmware-update-coming-to-the-ledger-nano-x
[commit-proto]: https://github.com/tendermint/tendermint/blob/be7cb50bb3432ee652f88a443e8ee7b8ef7122bc/proto/tendermint/types/types.proto#L121
[canonical-vote-proto]: https://github.com/tendermint/tendermint/blob/be7cb50bb3432ee652f88a443e8ee7b8ef7122bc/spec/core/encoding.md#L283
[blst]: https://github.com/supranational/blst
[prysm-blst]: https://github.com/prysmaticlabs/prysm/blob/develop/go.mod#L75
[gnark]: https://github.com/ConsenSys/gnark-crypto/
[eth-2-adoption]: https://notes.ethereum.org/@GW1ZUbNKR5iRjjKYx6_dJQ/Skxf3tNcg_
[bls-weil-pairing]: https://www.iacr.org/archive/asiacrypt2001/22480516.pdf
[summing-zero-paper]: https://eprint.iacr.org/2021/323.pdf
[circl]: https://github.com/cloudflare/circl
[suggested-ed25519-agg]: https://github.com/tendermint/tendermint/issues/7892
