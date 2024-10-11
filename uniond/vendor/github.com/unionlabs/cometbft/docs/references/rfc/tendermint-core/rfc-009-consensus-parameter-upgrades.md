# RFC 009 : Consensus Parameter Upgrade Considerations

## Changelog

- 06-Jan-2011: Initial draft (@williambanfield).

## Abstract

This document discusses the challenges of adding additional consensus parameters
to Tendermint and proposes a few solutions that can enable addition of consensus
parameters in a backwards-compatible way.

## Background

This section provides an overview of the issues of adding consensus parameters
to Tendermint.

### Hash Compatibility

Tendermint produces a hash of a subset of the consensus parameters. The values
that are hashed currently are the `BlockMaxGas` and the `BlockMaxSize`. These
are currently in the [HashedParams struct][hashed-params]. This hash is included
in the block and validators use it to validate that their local view of the consensus
parameters matches what the rest of the network is configured with.

Any new consensus parameters added to Tendermint should be included in this
hash. This presents a challenge for verification of historical blocks when consensus
parameters are added. If a network produced blocks with a version of Tendermint that
did not yet have the new consensus parameters, the parameter hash it produced will
not reference the new parameters. Any nodes joining the network with the newer
version of Tendermint will have the new consensus parameters. Tendermint will need
to handle this case so that new versions of Tendermint with new consensus parameters
can still validate old blocks correctly without having to do anything overly complex
or hacky.

### Allowing Developer-Defined Values and the `EndBlock` Problem

When new consensus parameters are added, application developers may wish to set
values for them so that the developer-defined values may be used as soon as the
software upgrades. We do not currently have a clean mechanism for handling this.

Consensus parameter updates are communicated from the application to Tendermint
within `EndBlock` of some height `H` and take effect at the next height, `H+1`.
This means that for updates that add a consensus parameter, there is a single
height where the new parameters cannot take effect. The parameters did not exist
in the version of the software that emitted the `EndBlock` response for height `H-1`,
so they cannot take effect at height `H`. The first height that the updated params
can take effect is height `H+1`. As of now, height `H` must run with the defaults.

## Discussion

### Hash Compatibility

This section discusses possible solutions to the problem of maintaining backwards-compatibility
of hashed parameters while adding new parameters.

#### Never Hash Defaults

One solution to the problem of backwards-compatibility is to never include parameters
in the hash if the are using the default value. This means that blocks produced
before the parameters existed will have implicitly been created with the defaults.
This works because any software with newer versions of Tendermint must be using the
defaults for new parameters when validating old blocks since the defaults can not
have been updated until a height at which the parameters existed.

#### Only Update HashedParams on Hash-Breaking Releases

An alternate solution to never hashing defaults is to not update the hashed
parameters on non-hash-breaking releases. This means that when new consensus
parameters are added to Tendermint, there may be a release that makes use of the
parameters but does not verify that they are the same across all validators by
referencing them in the hash. This seems reasonably safe given the fact that
only a very far subset of the consensus parameters are currently verified at all.

#### Version The Consensus Parameter Hash Scheme

The upcoming work on [soft upgrades](https://github.com/tendermint/spec/pull/222)
proposes applying different hashing rules depending on the active block version.
The consensus parameter hash could be versioned in the same way. When different
block versions are used, a different set of consensus parameters will be included
in the hash.

### Developer Defined Values

This section discusses possible solutions to the problem of allowing application
developers to define values for the new parameters during the upgrade that adds
the parameters.

#### Using `InitChain` for New Values

One solution to the problem of allowing application developers to define values
for new consensus parameters is to call the `InitChain` ABCI method on application
startup and fetch the value for any new consensus parameters. The [response object][init-chain-response]
contains a field for `ConsensusParameter` updates so this may serve as a natural place
to put this logic.

This poses a few difficulties. Nodes replaying old blocks while running new
software do not ever call `InitChain` after the initial time. They will therefore
not have a way to determine that the parameters changed at some height by using a
call to `InitChain`. The `EndBlock` response is how parameter changes at a height
are currently communicated to Tendermint and conflating these cases seems risky.

#### Force Defaults For Single Height

An alternate option is to not use `InitChain` and instead require chains to use the
default values of the new parameters for a single height.

As documented in the upcoming [ADR-74][adr-74], popular chains often simply use the default
values. Additionally, great care is being taken to ensure that logic governed by upcoming
consensus parameters is not liveness-breaking. This means that, at worst-case,
chains will experience a single slow height while waiting for the new values to
by applied.

#### Add a new `UpgradeChain` method

An additional method for allowing chains to update the consensus parameters that
do not yet exist is to add a new `UpgradeChain` method to `ABCI`. The upgrade chain
method would be called when the chain detects that the version of block that it
is about to produce does not match the previous block. This method would be called
after `EndBlock` and would return the set of consensus parameters to use at the
next height. It would therefore give an application the chance to set the new
consensus parameters before running a height with these new parameter.

### References

[hashed-params]: https://github.com/tendermint/tendermint/blob/0ae974e63911804d4a2007bd8a9b3ad81d6d2a90/types/params.go#L49
[init-chain-response]: https://github.com/tendermint/tendermint/blob/0ae974e63911804d4a2007bd8a9b3ad81d6d2a90/abci/types/types.pb.go#L1616
[adr-74]: https://github.com/tendermint/tendermint/pull/7503
