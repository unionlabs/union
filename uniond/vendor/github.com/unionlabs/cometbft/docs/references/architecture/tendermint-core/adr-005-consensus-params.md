# ADR 005: Consensus Params

## Context

Consensus critical parameters controlling blockchain capacity have until now been hard coded, loaded from a local config, or neglected.
Since they may be need to be different in different networks, and potentially to evolve over time within
networks, we seek to initialize them in a genesis file, and expose them through the ABCI.

While we have some specific parameters now, like maximum block and transaction size, we expect to have more in the future,
such as a period over which evidence is valid, or the frequency of checkpoints.

## Decision

### ConsensusParams

No consensus critical parameters should ever be found in the `config.toml`.

A new `ConsensusParams` is optionally included in the `genesis.json` file,
and loaded into the `State`. Any items not included are set to their default value.
A value of 0 is undefined (see ABCI, below). A value of -1 is used to indicate the parameter does not apply.
The parameters are used to determine the validity of a block (and tx) via the union of all relevant parameters.

```
type ConsensusParams struct {
    BlockSize
    TxSize
    BlockGossip
}

type BlockSize struct {
    MaxBytes int
    MaxTxs int
    MaxGas int
}

type TxSize struct {
    MaxBytes int
    MaxGas int
}

type BlockGossip struct {
    BlockPartSizeBytes int
}
```

The `ConsensusParams` can evolve over time by adding new structs that cover different aspects of the consensus rules.

The `BlockPartSizeBytes` and the `BlockSize.MaxBytes` are enforced to be greater than 0.
The former because we need a part size, the latter so that we always have at least some sanity check over the size of blocks.

### ABCI

#### InitChain

InitChain currently takes the initial validator set. It should be extended to also take parts of the ConsensusParams.
There is some case to be made for it to take the entire Genesis, except there may be things in the genesis,
like the BlockPartSize, that the app shouldn't really know about.

#### EndBlock

The EndBlock response includes a `ConsensusParams`, which includes BlockSize and TxSize, but not BlockGossip.
Other param struct can be added to `ConsensusParams` in the future.
The `0` value is used to denote no change.
Any other value will update that parameter in the `State.ConsensusParams`, to be applied for the next block.
Tendermint should have hard-coded upper limits as sanity checks.

## Status

Implemented

## Consequences

### Positive

- Alternative capacity limits and consensus parameters can be specified without re-compiling the software.
- They can also change over time under the control of the application

### Negative

- More exposed parameters is more complexity
- Different rules at different heights in the blockchain complicates fast sync

### Neutral

- The TxSize, which checks validity, may be in conflict with the config's `max_block_size_tx`, which determines proposal sizes
