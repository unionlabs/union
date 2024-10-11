# ADR 009: Evidence Module

## Changelog

* 2019 July 31: Initial draft
* 2019 October 24: Initial implementation

## Status

Accepted

## Context

In order to support building highly secure, robust and interoperable blockchain
applications, it is vital for the Cosmos SDK to expose a mechanism in which arbitrary
evidence can be submitted, evaluated and verified resulting in some agreed upon
penalty for any misbehavior committed by a validator, such as equivocation (double-voting),
signing when unbonded, signing an incorrect state transition (in the future), etc.
Furthermore, such a mechanism is paramount for any
[IBC](https://github.com/cosmos/ibc) or
cross-chain validation protocol implementation in order to support the ability
for any misbehavior to be relayed back from a collateralized chain to a primary
chain so that the equivocating validator(s) can be slashed.

## Decision

We will implement an evidence module in the Cosmos SDK supporting the following
functionality:

* Provide developers with the abstractions and interfaces necessary to define
  custom evidence messages, message handlers, and methods to slash and penalize
  accordingly for misbehavior.
* Support the ability to route evidence messages to handlers in any module to
  determine the validity of submitted misbehavior.
* Support the ability, through governance, to modify slashing penalties of any
  evidence type.
* Querier implementation to support querying params, evidence types, params, and
  all submitted valid misbehavior.

### Types

First, we define the `Evidence` interface type. The `x/evidence` module may implement
its own types that can be used by many chains (e.g. `CounterFactualEvidence`).
In addition, other modules may implement their own `Evidence` types in a similar
manner in which governance is extensible. It is important to note any concrete
type implementing the `Evidence` interface may include arbitrary fields such as
an infraction time. We want the `Evidence` type to remain as flexible as possible.

When submitting evidence to the `x/evidence` module, the concrete type must provide
the validator's consensus address, which should be known by the `x/slashing`
module (assuming the infraction is valid), the height at which the infraction
occurred and the validator's power at same height in which the infraction occurred.

```go
type Evidence interface {
  Route() string
  Type() string
  String() string
  Hash() HexBytes
  ValidateBasic() error

  // The consensus address of the malicious validator at time of infraction
  GetConsensusAddress() ConsAddress

  // Height at which the infraction occurred
  GetHeight() int64

  // The total power of the malicious validator at time of infraction
  GetValidatorPower() int64

  // The total validator set power at time of infraction
  GetTotalPower() int64
}
```

### Routing & Handling

Each `Evidence` type must map to a specific unique route and be registered with
the `x/evidence` module. It accomplishes this through the `Router` implementation.

```go
type Router interface {
  AddRoute(r string, h Handler) Router
  HasRoute(r string) bool
  GetRoute(path string) Handler
  Seal()
}
```

Upon successful routing through the `x/evidence` module, the `Evidence` type
is passed through a `Handler`. This `Handler` is responsible for executing all
corresponding business logic necessary for verifying the evidence as valid. In
addition, the `Handler` may execute any necessary slashing and potential jailing.
Since slashing fractions will typically result from some form of static functions,
allow the `Handler` to do this provides the greatest flexibility. An example could
be `k * evidence.GetValidatorPower()` where `k` is an on-chain parameter controlled
by governance. The `Evidence` type should provide all the external information
necessary in order for the `Handler` to make the necessary state transitions.
If no error is returned, the `Evidence` is considered valid.

```go
type Handler func(Context, Evidence) error
```

### Submission

`Evidence` is submitted through a `MsgSubmitEvidence` message type which is internally
handled by the `x/evidence` module's `SubmitEvidence`.

```go
type MsgSubmitEvidence struct {
  Evidence
}

func handleMsgSubmitEvidence(ctx Context, keeper Keeper, msg MsgSubmitEvidence) Result {
  if err := keeper.SubmitEvidence(ctx, msg.Evidence); err != nil {
    return err.Result()
  }

  // emit events...

  return Result{
    // ...
  }
}
```

The `x/evidence` module's keeper is responsible for matching the `Evidence` against
the module's router and invoking the corresponding `Handler` which may include
slashing and jailing the validator. Upon success, the submitted evidence is persisted.

```go
func (k Keeper) SubmitEvidence(ctx Context, evidence Evidence) error {
  handler := keeper.router.GetRoute(evidence.Route())
  if err := handler(ctx, evidence); err != nil {
    return ErrInvalidEvidence(keeper.codespace, err)
  }

  keeper.setEvidence(ctx, evidence)
  return nil
}
```

### Genesis

Finally, we need to represent the genesis state of the `x/evidence` module. The
module only needs a list of all submitted valid infractions and any necessary params
for which the module needs in order to handle submitted evidence. The `x/evidence`
module will naturally define and route native evidence types for which it'll most
likely need slashing penalty constants for.

```go
type GenesisState struct {
  Params       Params
  Infractions  []Evidence
}
```

## Consequences

### Positive

* Allows the state machine to process misbehavior submitted on-chain and penalize
  validators based on agreed upon slashing parameters.
* Allows evidence types to be defined and handled by any module. This further allows
  slashing and jailing to be defined by more complex mechanisms.
* Does not solely rely on Tendermint to submit evidence.

### Negative

* No easy way to introduce new evidence types through governance on a live chain
  due to the inability to introduce the new evidence type's corresponding handler

### Neutral

* Should we persist infractions indefinitely? Or should we rather rely on events?

## References

* [ICS](https://github.com/cosmos/ics)
* [IBC Architecture](https://github.com/cosmos/ibc/blob/main/spec/ics-001-ics-standard/README.md)
* [Tendermint Fork Accountability](https://github.com/tendermint/spec/blob/7b3138e69490f410768d9b1ffc7a17abc23ea397/spec/consensus/fork-accountability.md)
