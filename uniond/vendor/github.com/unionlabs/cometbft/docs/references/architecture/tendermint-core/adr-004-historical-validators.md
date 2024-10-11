# ADR 004: Historical Validators

## Context

Right now, we can query the present validator set, but there is no history.
If you were offline for a long time, there is no way to reconstruct past validators. This is needed for the light client and we agreed needs enhancement of the API.

## Decision

For every block, store a new structure that contains either the latest validator set,
or the height of the last block for which the validator set changed. Note this is not
the height of the block which returned the validator set change itself, but the next block,
ie. the first block it comes into effect for.

Storing the validators will be handled by the `state` package.

At some point in the future, we may consider more efficient storage in the case where the validators
are updated frequently - for instance by only saving the diffs, rather than the whole set.

An alternative approach suggested keeping the validator set, or diffs of it, in a merkle IAVL tree.
While it might afford cheaper proofs that a validator set has not changed, it would be more complex,
and likely less efficient.

## Status

Implemented

## Consequences

### Positive

- Can query old validator sets, with proof.

### Negative

- Writes an extra structure to disk with every block.

### Neutral
