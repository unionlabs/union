# ADR 018: ABCI Validator Improvements

## Changelog

016-08-2018: Follow up from review: - Revert changes to commit round - Remind about justification for removing pubkey - Update pros/cons
05-08-2018: Initial draft

## Context

ADR 009 introduced major improvements to the ABCI around validators and the use
of Amino. Here we follow up with some additional changes to improve the naming
and expected use of Validator messages.

## Decision

### Validator

Currently a Validator contains `address` and `pub_key`, and one or the other is
optional/not-sent depending on the use case. Instead, we should have a
`Validator` (with just the address, used for RequestBeginBlock)
and a `ValidatorUpdate` (with the pubkey, used for ResponseEndBlock):

```
message Validator {
    bytes address
    int64 power
}

message ValidatorUpdate {
    PubKey pub_key
    int64 power
}
```

As noted in [ADR-009](adr-009-ABCI-design.md),
the `Validator` does not contain a pubkey because quantum public keys are
quite large and it would be wasteful to send them all over ABCI with every block.
Thus, applications that want to take advantage of the information in BeginBlock
are _required_ to store pubkeys in state (or use much less efficient lazy means
of verifying BeginBlock data).

### RequestBeginBlock

LastCommitInfo currently has an array of `SigningValidator` that contains
information for each validator in the entire validator set.
Instead, this should be called `VoteInfo`, since it is information about the
validator votes.

Note that all votes in a commit must be from the same round.

```
message LastCommitInfo {
  int64 round
  repeated VoteInfo commit_votes
}

message VoteInfo {
    Validator validator
    bool signed_last_block
}
```

### ResponseEndBlock

Use ValidatorUpdates instead of Validators. Then it's clear we don't need an
address, and we do need a pubkey.

We could require the address here as well as a sanity check, but it doesn't seem
necessary.

### InitChain

Use ValidatorUpdates for both Request and Response. InitChain
is about setting/updating the initial validator set, unlike BeginBlock
which is just informational.

## Status

Implemented

## Consequences

### Positive

- Clarifies the distinction between the different uses of validator information

### Negative

- Apps must still store the public keys in state to utilize the RequestBeginBlock info

### Neutral

- ResponseEndBlock does not require an address

## References

- [Latest ABCI Spec](https://github.com/tendermint/tendermint/blob/v0.22.8/docs/app-dev/abci-spec.md)
- [ADR-009](https://github.com/tendermint/tendermint/blob/v0.22.8/docs/architecture/adr-009-ABCI-design.md)
- [Issue #1712 - Don't send PubKey in
  RequestBeginBlock](https://github.com/tendermint/tendermint/issues/1712)
