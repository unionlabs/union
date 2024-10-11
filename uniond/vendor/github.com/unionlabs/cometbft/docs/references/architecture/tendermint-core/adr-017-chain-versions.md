# ADR 017: Chain Versions

## TODO

- clarify how to handle slashing when ChainID changes

## Changelog

- 28-07-2018: Updates from review
  - split into two ADRs - one for protocol, one for chains
- 16-07-2018: Initial draft - was originally joint ADR for protocol and chain
  versions

## Context

Software and Protocol versions are covered in a separate ADR.

Here we focus on chain versions.

## Requirements

We need to version blockchains across protocols, networks, forks, etc.
We need chain identifiers and descriptions so we can talk about a multitude of chains,
and especially the differences between them, in a meaningful way.

### Networks

We need to support many independent networks running the same version of the software,
even possibly starting from the same initial state.
They must have distinct identifiers so that peers know which one they are joining and so
validators and users can prevent replay attacks.

Call this the `NetworkName` (note we currently call this `ChainID` in the software. In this
ADR, ChainID has a different meaning).
It represents both the application being run and the community or intention
of running it.

Peers only connect to other peers with the same NetworkName.

### Forks

We need to support existing networks upgrading and forking, wherein they may do any of:

    - revert back to some height, continue with the same versions but new blocks
    - arbitrarily mutate state at some height, continue with the same versions (eg. Dao Fork)
    - change the AppVersion at some height

Note because of Tendermint's voting power threshold rules, a chain can only be extended under the "original" rules and under the new rules
if 1/3 or more is double signing, which is expressly prohibited, and is supposed to result in their punishment on both chains. Since they can censor
the punishment, the chain is expected to be hardforked to remove the validators. Thus, if both branches are to continue after a fork,
they will each require a new identifier, and the old chain identifier will be retired (ie. only useful for syncing history, not for new blocks)..

TODO: explain how to handle slashing when chain id changed!

We need a consistent way to describe forks.

## Proposal

### ChainDescription

ChainDescription is a complete immutable description of a blockchain. It takes the following form:

```
ChainDescription = <NetworkName>/<BlockVersion>/<AppVersion>/<StateHash>/<ValHash>/<ConsensusParamsHash>
```

Here, StateHash is the merkle root of the initial state, ValHash is the merkle root of the initial Tendermint validator set,
and ConsensusParamsHash is the merkle root of the initial Tendermint consensus parameters.

The `genesis.json` file must contain enough information to compute this value. It need not contain the StateHash or ValHash itself,
but contain the state from which they can be computed with the given protocol versions.

NOTE: consider splitting NetworkName into NetworkName and AppName - this allows
folks to independently use the same application for different networks (ie we
could imagine multiple communities of validators wanting to put up a Hub using
the same app but having a distinct network name. Arguably not needed if
differences will come via different initial state / validators).

#### ChainID

Define `ChainID = TMHASH(ChainDescriptor)`. It's the unique ID of a blockchain.

It should be Bech32 encoded when handled by users, eg. with `cosmoschain` prefix.

#### Forks and Upgrades

When a chain forks or upgrades but continues the same history, it takes a new ChainDescription as follows:

```
ChainDescription = <ChainID>/x/<Height>/<ForkDescription>
```

Where

- ChainID is the ChainID from the previous ChainDescription (ie. its hash)
- `x` denotes that a change occurred
- `Height` is the height the change occurred
- ForkDescription has the same form as ChainDescription but for the fork
- this allows forks to specify new versions for tendermint or the app, as well as arbitrary changes to the state or validator set
