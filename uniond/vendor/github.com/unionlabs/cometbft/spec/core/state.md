---
order: 4
---

# State

The state contains information whose cryptographic digest is included in block headers, and thus is
necessary for validating new blocks. For instance, the validators set and the results of
transactions are never included in blocks, but their Merkle roots are:
the state keeps track of them.

The `State` object itself is an implementation detail, since it is never
included in a block or gossiped over the network, and we never compute
its hash. The persistence or query interface of the `State` object
is an implementation detail and not included in the specification.
However, the types in the `State` object are part of the specification, since
the Merkle roots of the `State` objects are included in blocks and values are used during
validation.

```go
type State struct {
    ChainID        string
    InitialHeight  int64

    LastBlockHeight int64
    LastBlockID     types.BlockID
    LastBlockTime   time.Time

    Version     Version
    LastResults []Result
    AppHash     []byte

    LastValidators ValidatorSet
    Validators     ValidatorSet
    NextValidators ValidatorSet

    ConsensusParams ConsensusParams
}
```

The chain ID and initial height are taken from the genesis file, and not changed again. The
initial height will be `1` in the typical case, `0` is an invalid value.

Note there is a hard-coded limit of 10000 validators. This is inherited from the
limit on the number of votes in a commit.

Further information on [`Validator`'s](./data_structures.md#validator),
[`ValidatorSet`'s](./data_structures.md#validatorset) and
[`ConsensusParams`'s](./data_structures.md#consensusparams) can
be found in [data structures](./data_structures.md)

## Execution

State gets updated at the end of executing a block. Of specific interest is `EndBlockResponse` and
`CommitResponse`

```go
type EndBlockResponse struct {
	ValidatorUpdates      []ValidatorUpdate       `protobuf:"bytes,1,rep,name=validator_updates,json=validatorUpdates,proto3" json:"validator_updates"`
	ConsensusParamUpdates *types1.ConsensusParams `protobuf:"bytes,2,opt,name=consensus_param_updates,json=consensusParamUpdates,proto3" json:"consensus_param_updates,omitempty"`
	Events                []Event                 `protobuf:"bytes,3,rep,name=events,proto3" json:"events,omitempty"`
}
```

where

```go
type ValidatorUpdate struct {
	Power       int64  `protobuf:"varint,2,opt,name=power,proto3" json:"power,omitempty"`
	PubKeyBytes []byte `protobuf:"bytes,3,opt,name=pub_key_bytes,json=pubKeyBytes,proto3" json:"pub_key_bytes,omitempty"`
	PubKeyType  string `protobuf:"bytes,4,opt,name=pub_key_type,json=pubKeyType,proto3" json:"pub_key_type,omitempty"`
}
```

and

```go
type CommitResponse struct {
	// reserve 1
	Data         []byte `protobuf:"bytes,2,opt,name=data,proto3" json:"data,omitempty"`
	RetainHeight int64  `protobuf:"varint,3,opt,name=retain_height,json=retainHeight,proto3" json:"retain_height,omitempty"`
}
```

`ValidatorUpdates` are used to add and remove validators to the current set as well as update
validator power. Setting validator power to 0 in `ValidatorUpdate` will cause the validator to be
removed. `ConsensusParams` are safely copied across (i.e. if a field is nil it gets ignored) and the
`Data` from the `CommitResponse` is used as the `AppHash`

## Version

```go
type Version struct {
  consensus Consensus
  software string
}
```

[`Consensus`](./data_structures.md#version) contains the protocol version for the blockchain and the
application.

## Block

The total size of a block is limited in bytes by the `ConsensusParams.Block.MaxBytes`.
Proposed blocks must be less than this size, and will be considered invalid
otherwise.

The Application may set `ConsensusParams.Block.MaxBytes` to -1.
In that case, the actual block limit is set to 100 MB,
and CometBFT will provide all transactions in the mempool as part of `PrepareProposal`.
The application has to be careful to return a list of transactions in `PrepareProposalResponse`
whose size is less than or equal to `PrepareProposalRequest.MaxTxBytes`.

Blocks should additionally be limited by the amount of "gas" consumed by the
transactions in the block, though this is not yet implemented.

## Evidence

For evidence in a block to be valid, it must satisfy:

```go
block.Header.Time-evidence.Time < ConsensusParams.Evidence.MaxAgeDuration &&
 block.Header.Height-evidence.Height < ConsensusParams.Evidence.MaxAgeNumBlocks
```

A block must not contain more than `ConsensusParams.Evidence.MaxBytes` of evidence. This is
implemented to mitigate spam attacks.

## Validator

Validators from genesis file and `EndBlockResponse` must have pubkeys of type ∈
`ConsensusParams.Validator.PubKeyTypes`.
