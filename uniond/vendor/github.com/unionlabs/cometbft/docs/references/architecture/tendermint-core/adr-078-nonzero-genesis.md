# ADR 078: Non-Zero Genesis

## Changelog

- 2020-07-26: Initial draft (@erikgrinaker)
- 2020-07-28: Use weak chain linking, i.e. `predecessor` field (@erikgrinaker)
- 2020-07-31: Drop chain linking (@erikgrinaker)
- 2020-08-03: Add `State.InitialHeight` (@erikgrinaker)
- 2021-02-11: Migrate to tendermint repo (Originally [RFC 002](https://github.com/tendermint/spec/pull/119))

## Author(s)

- Erik Grinaker (@erikgrinaker)

## Context

The recommended upgrade path for block protocol-breaking upgrades is currently to hard fork the
chain (see e.g. [`cosmoshub-3` upgrade](https://blog.cosmos.network/cosmos-hub-3-upgrade-announcement-39c9da941aee).
This is done by halting all validators at a predetermined height, exporting the application
state via application-specific tooling, and creating an entirely new chain using the exported
application state.

As far as Tendermint is concerned, the upgraded chain is a completely separate chain, with e.g.
a new chain ID and genesis file. Notably, the new chain starts at height 1, and has none of the
old chain's block history. This causes problems for integrators, e.g. coin exchanges and
wallets, that assume a monotonically increasing height for a given blockchain. Users also find
it confusing that a given height can now refer to distinct states depending on the chain
version.

An ideal solution would be to always retain block backwards compatibility in such a way that chain
history is never lost on upgrades. However, this may require a significant amount of engineering
work that is not viable for the planned Stargate release (Tendermint 0.34), and may prove too
restrictive for future development.

As a first step, allowing the new chain to start from an initial height specified in the genesis
file would at least provide monotonically increasing heights. There was a proposal to include the
last block header of the previous chain as well, but since the genesis file is not verified and
hashed (only specific fields are) this would not be trustworthy.

External tooling will be required to map historical heights onto e.g. archive nodes that contain
blocks from previous chain version. Tendermint will not include any such functionality.

## Proposal

Tendermint will allow chains to start from an arbitrary initial height:

- A new field `initial_height` is added to the genesis file, defaulting to `1`. It can be set to any
non-negative integer, and `0` is considered equivalent to `1`.

- A new field `InitialHeight` is added to the ABCI `RequestInitChain` message, with the same value
and semantics as the genesis field.

- A new field `InitialHeight` is added to the `state.State` struct, where `0` is considered invalid.
  Including the field here simplifies implementation, since the genesis value does not have to be
  propagated throughout the code base separately, but it is not strictly necessary.

ABCI applications may have to be updated to handle arbitrary initial heights, otherwise the initial
block may fail.

## Status

Implemented

## Consequences

### Positive

- Heights can be unique throughout the history of a "logical" chain, across hard fork upgrades.

### Negative

- Upgrades still cause loss of block history.

- Integrators will have to map height ranges to specific archive nodes/networks to query history.

### Neutral

- There is no explicit link to the last block of the previous chain.

## References

- [#2543: Allow genesis file to start from non-zero height w/ prev block header](https://github.com/tendermint/tendermint/issues/2543)
