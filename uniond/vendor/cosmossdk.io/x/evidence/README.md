---
sidebar_position: 1
---

# `x/evidence`

* [Concepts](#concepts)
* [State](#state)
* [Messages](#messages)
* [Events](#events)
* [Parameters](#parameters)
* [BeginBlock](#beginblock)
* [Client](#client)
    * [CLI](#cli)
    * [REST](#rest)
    * [gRPC](#grpc)

## Abstract

`x/evidence` is an implementation of a Cosmos SDK module, per [ADR 009](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-009-evidence-module.md),
that allows for the submission and handling of arbitrary evidence of misbehavior such
as equivocation and counterfactual signing.

The evidence module differs from standard evidence handling which typically expects the
underlying consensus engine, e.g. CometBFT, to automatically submit evidence when
it is discovered by allowing clients and foreign chains to submit more complex evidence
directly.

All concrete evidence types must implement the `Evidence` interface contract. Submitted
`Evidence` is first routed through the evidence module's `Router` in which it attempts
to find a corresponding registered `Handler` for that specific `Evidence` type.
Each `Evidence` type must have a `Handler` registered with the evidence module's
keeper in order for it to be successfully routed and executed.

Each corresponding handler must also fulfill the `Handler` interface contract. The
`Handler` for a given `Evidence` type can perform any arbitrary state transitions
such as slashing, jailing, and tombstoning.

## Concepts

### Evidence

Any concrete type of evidence submitted to the `x/evidence` module must fulfill the
`Evidence` contract outlined below. Not all concrete types of evidence will fulfill
this contract in the same way and some data may be entirely irrelevant to certain
types of evidence. An additional `ValidatorEvidence`, which extends `Evidence`,
has also been created to define a contract for evidence against malicious validators.

```go
// Evidence defines the contract which concrete evidence types of misbehavior
// must implement.
type Evidence interface {
	proto.Message

	Route() string
	String() string
	Hash() []byte
	ValidateBasic() error

	// Height at which the infraction occurred
	GetHeight() int64
}

// ValidatorEvidence extends Evidence interface to define contract
// for evidence against malicious validators
type ValidatorEvidence interface {
	Evidence

	// The consensus address of the malicious validator at time of infraction
	GetConsensusAddress() sdk.ConsAddress

	// The total power of the malicious validator at time of infraction
	GetValidatorPower() int64

	// The total validator set power at time of infraction
	GetTotalPower() int64
}
```

### Registration & Handling

The `x/evidence` module must first know about all types of evidence it is expected
to handle. This is accomplished by registering the `Route` method in the `Evidence`
contract with what is known as a `Router` (defined below). The `Router` accepts
`Evidence` and attempts to find the corresponding `Handler` for the `Evidence`
via the `Route` method.

```go
type Router interface {
  AddRoute(r string, h Handler) Router
  HasRoute(r string) bool
  GetRoute(path string) Handler
  Seal()
  Sealed() bool
}
```

The `Handler` (defined below) is responsible for executing the entirety of the
business logic for handling `Evidence`. This typically includes validating the
evidence, both stateless checks via `ValidateBasic` and stateful checks via any
keepers provided to the `Handler`. In addition, the `Handler` may also perform
capabilities such as slashing and jailing a validator. All `Evidence` handled
by the `Handler` should be persisted.

```go
// Handler defines an agnostic Evidence handler. The handler is responsible
// for executing all corresponding business logic necessary for verifying the
// evidence as valid. In addition, the Handler may execute any necessary
// slashing and potential jailing.
type Handler func(context.Context, Evidence) error
```


## State

Currently the `x/evidence` module only stores valid submitted `Evidence` in state.
The evidence state is also stored and exported in the `x/evidence` module's `GenesisState`.

```protobuf
// GenesisState defines the evidence module's genesis state.
message GenesisState {
  // evidence defines all the evidence at genesis.
  repeated google.protobuf.Any evidence = 1;
}

```

All `Evidence` is retrieved and stored via a prefix `KVStore` using prefix `0x00` (`KeyPrefixEvidence`).


## Messages

### MsgSubmitEvidence

Evidence is submitted through a `MsgSubmitEvidence` message:

```protobuf
// MsgSubmitEvidence represents a message that supports submitting arbitrary
// Evidence of misbehavior such as equivocation or counterfactual signing.
message MsgSubmitEvidence {
  string              submitter = 1;
  google.protobuf.Any evidence  = 2;
}
```

Note, the `Evidence` of a `MsgSubmitEvidence` message must have a corresponding
`Handler` registered with the `x/evidence` module's `Router` in order to be processed
and routed correctly.

Given the `Evidence` is registered with a corresponding `Handler`, it is processed
as follows:

```go
func SubmitEvidence(ctx Context, evidence Evidence) error {
  if _, err := GetEvidence(ctx, evidence.Hash()); err == nil {
    return errorsmod.Wrap(types.ErrEvidenceExists, strings.ToUpper(hex.EncodeToString(evidence.Hash())))
  }
  if !router.HasRoute(evidence.Route()) {
    return errorsmod.Wrap(types.ErrNoEvidenceHandlerExists, evidence.Route())
  }

  handler := router.GetRoute(evidence.Route())
  if err := handler(ctx, evidence); err != nil {
    return errorsmod.Wrap(types.ErrInvalidEvidence, err.Error())
  }

  ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			types.EventTypeSubmitEvidence,
			sdk.NewAttribute(types.AttributeKeyEvidenceHash, strings.ToUpper(hex.EncodeToString(evidence.Hash()))),
		),
	)

  SetEvidence(ctx, evidence)
  return nil
}
```

First, there must not already exist valid submitted `Evidence` of the exact same
type. Secondly, the `Evidence` is routed to the `Handler` and executed. Finally,
if there is no error in handling the `Evidence`, an event is emitted and it is persisted to state.


## Events

The `x/evidence` module emits the following events:

### Handlers

#### MsgSubmitEvidence

| Type            | Attribute Key | Attribute Value |
| --------------- | ------------- | --------------- |
| submit_evidence | evidence_hash | {evidenceHash}  |
| message         | module        | evidence        |
| message         | sender        | {senderAddress} |
| message         | action        | submit_evidence |


## Parameters

The evidence module does not contain any parameters.


## BeginBlock

### Evidence Handling

CometBFT blocks can include
[Evidence](https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_basic_concepts.md#evidence) that indicates if a validator committed malicious behavior. The relevant information is forwarded to the application as ABCI Evidence in `abci.RequestBeginBlock` so that the validator can be punished accordingly.

#### Equivocation

The Cosmos SDK handles two types of evidence inside the ABCI `BeginBlock`:

* `DuplicateVoteEvidence`,
* `LightClientAttackEvidence`.

The evidence module handles these two evidence types the same way. First, the Cosmos SDK converts the CometBFT concrete evidence type to an SDK `Evidence` interface using `Equivocation` as the concrete type.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/evidence/v1beta1/evidence.proto#L12-L32
```

For some `Equivocation` submitted in `block` to be valid, it must satisfy:

`Evidence.Timestamp >= block.Timestamp - MaxEvidenceAge`

Where:

* `Evidence.Timestamp` is the timestamp in the block at height `Evidence.Height`
* `block.Timestamp` is the current block timestamp.

If valid `Equivocation` evidence is included in a block, the validator's stake is
reduced (slashed) by `SlashFractionDoubleSign` as defined by the `x/slashing` module
of what their stake was when the infraction occurred, rather than when the evidence was discovered.
We want to "follow the stake", i.e., the stake that contributed to the infraction
should be slashed, even if it has since been redelegated or started unbonding.

In addition, the validator is permanently jailed and tombstoned to make it impossible for that
validator to ever re-enter the validator set.

The `Equivocation` evidence is handled as follows:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/evidence/keeper/infraction.go#L26-L140
```

**Note:** The slashing, jailing, and tombstoning calls are delegated through the `x/slashing` module
that emits informative events and finally delegates calls to the `x/staking` module. See documentation
on slashing and jailing in [State Transitions](../staking/README.md#state-transitions).

## Client

### CLI

A user can query and interact with the `evidence` module using the CLI.

#### Query

The `query` commands allows users to query `evidence` state.

```bash
simd query evidence --help
```

#### evidence

The `evidence` command allows users to list all evidence or evidence by hash.

Usage:

```bash
simd query evidence evidence [flags]
```

To query evidence by hash

Example:

```bash
simd query evidence evidence "DF0C23E8634E480F84B9D5674A7CDC9816466DEC28A3358F73260F68D28D7660"
```

Example Output:

```bash
evidence:
  consensus_address: cosmosvalcons1ntk8eualewuprz0gamh8hnvcem2nrcdsgz563h
  height: 11
  power: 100
  time: "2021-10-20T16:08:38.194017624Z"
```

To get all evidence

Example:

```bash
simd query evidence list
```

Example Output:

```bash
evidence:
  consensus_address: cosmosvalcons1ntk8eualewuprz0gamh8hnvcem2nrcdsgz563h
  height: 11
  power: 100
  time: "2021-10-20T16:08:38.194017624Z"
pagination:
  next_key: null
  total: "1"
```

### REST

A user can query the `evidence` module using REST endpoints.

#### Evidence

Get evidence by hash

```bash
/cosmos/evidence/v1beta1/evidence/{hash}
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/evidence/v1beta1/evidence/DF0C23E8634E480F84B9D5674A7CDC9816466DEC28A3358F73260F68D28D7660"
```

Example Output:

```bash
{
  "evidence": {
    "consensus_address": "cosmosvalcons1ntk8eualewuprz0gamh8hnvcem2nrcdsgz563h",
    "height": "11",
    "power": "100",
    "time": "2021-10-20T16:08:38.194017624Z"
  }
}
```

#### All evidence

Get all evidence

```bash
/cosmos/evidence/v1beta1/evidence
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/evidence/v1beta1/evidence"
```

Example Output:

```bash
{
  "evidence": [
    {
      "consensus_address": "cosmosvalcons1ntk8eualewuprz0gamh8hnvcem2nrcdsgz563h",
      "height": "11",
      "power": "100",
      "time": "2021-10-20T16:08:38.194017624Z"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

### gRPC

A user can query the `evidence` module using gRPC endpoints.

#### Evidence

Get evidence by hash

```bash
cosmos.evidence.v1beta1.Query/Evidence
```

Example:

```bash
grpcurl -plaintext -d '{"evidence_hash":"DF0C23E8634E480F84B9D5674A7CDC9816466DEC28A3358F73260F68D28D7660"}' localhost:9090 cosmos.evidence.v1beta1.Query/Evidence
```

Example Output:

```bash
{
  "evidence": {
    "consensus_address": "cosmosvalcons1ntk8eualewuprz0gamh8hnvcem2nrcdsgz563h",
    "height": "11",
    "power": "100",
    "time": "2021-10-20T16:08:38.194017624Z"
  }
}
```

#### All evidence

Get all evidence

```bash
cosmos.evidence.v1beta1.Query/AllEvidence
```

Example:

```bash
grpcurl -plaintext localhost:9090 cosmos.evidence.v1beta1.Query/AllEvidence
```

Example Output:

```bash
{
  "evidence": [
    {
      "consensus_address": "cosmosvalcons1ntk8eualewuprz0gamh8hnvcem2nrcdsgz563h",
      "height": "11",
      "power": "100",
      "time": "2021-10-20T16:08:38.194017624Z"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```
