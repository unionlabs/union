# ADR 067: Simulator v2

## Changelog

* June 01, 2023: Initial Draft (@alexanderbez)

## Status

DRAFT

## Abstract

The Cosmos SDK simulator is a tool that allows developers to test the entirety
of their application's state machine through the use of pseudo-randomized "operations",
which represent transactions. The simulator also provides primitives that ensures
there are no non-determinism issues and that the application's state machine can
be successfully exported and imported using randomized state.

The simulator has played an absolutely critical role in the development and testing
of the Cosmos Hub and all the releases of the Cosmos SDK after the launch of the
Cosmos Hub. Since the Hub, the simulator has relatively not changed much, so it's
overdue for a revamp.

## Context

The current simulator, `x/simulation`, acts as a semi-fuzz testing suite that takes
in an integer that represents a seed into a PRNG. The PRNG is used to generate a
sequence of "operations" that are meant to reflect transactions that an application's
state machine can process. Through the use of the PRNG, all aspects of block production
and consumption are randomized. This includes a block's proposer, the validators
who both sign and miss the block, along with the transaction operations themselves.

Each Cosmos SDK module defines a set of simulation operations that _attempt_ to
produce valid transactions, e.g. `x/distribution/simulation/operations.go`. These
operations can sometimes fail depending on the accumulated state of the application
within that simulation run. The simulator will continue to generate operations
until it has reached a certain number of operations or until it has reached a
fatal state, reporting results. This gives the ability for application developers
to reliably execute full range application simulation and fuzz testing against
their application.

However, there are a few major drawbacks. Namely, with the advent of ABCI++, specifically
`FinalizeBlock`, the internal workings of the simulator no longer comply with how
an application would actually perform. Specifically, operations are executed
_after_ `FinalizeBlock`, whereas they should be executed _within_ `FinalizeBlock`.

Additionally, the simulator is not very extensible. Developers should be able to
easily define and extend the following:

* Consistency or validity predicates (what are known as invariants today)
* Property tests of state before and after a block is simulated

In addition, we also want to achieve the following:

* Consolidated weight management, i.e. define weights within the simulator itself
  via a config and not defined in each module
* Observability of the simulator's execution, i.e. have easy to understand output/logs
  with the ability to pipe those logs into some external sink
* Smart replay, i.e. the ability to not only rerun a simulation from a seed, but
  also the ability to replay from an arbitrary breakpoint
* Run a simulation based off of real network state

## Decision

Instead of refactoring the existing simulator, `x/simulation`, we propose to create
a new package in the root of the Cosmos SDK, `simulator`, that will be the new
simulation framework. The simulator will more accurately reflect the complete
lifecycle of an ABCI application.

Specifically, we propose a similar implementation and use of a `simulator.Manager`,
that exists today, that is responsible for managing the execution of a simulation.
The manager will wrap an ABCI application and will be responsible for the following:

* Populating the application's mempool with a set of pseudo-random transactions
  before each block, some of which may contain invalid messages.
* Selecting transactions and a random proposer to execute `PrepareProposal`.
* Executing `ProcessProposal`, `FinalizeBlock` and `Commit`.
* Executing a set of validity predicates before and after each block.
* Maintaining a CPU and RAM profile of the simulation execution.
* Allowing a simulation to stop and resume from a given block height.
* Simulation liveness of each validator per-block.

From an application developer's perspective, they will only need to provide the
modules to be used in the simulator and the manager will take care of the rest.
In addition, they will not need to write their own simulation test(s), e.g.
non-determinism, multi-seed, etc..., as the manager will provide these as well.

```go
type Manager struct {
  app     sdk.Application
  mempool sdk.Mempool
  rng     rand.Rand
  // ...
}
```

### Configuration

The simulator's testing input will be driven by a configuration file, as opposed
to CLI arguments. This will allow for more extensibility and ease of use along with
the ability to have shared configuration files across multiple simulations.

### Execution

As alluded to previously, after the execution of each block, the manager will
generate a series of pseudo-random transactions and attempt to insert them into
the mempool via `BaseApp#CheckTx`. During the ABCI lifecycle of a block, this
mempool will be used to seed the transactions into a block proposal as it would
in a real network. This allows us to not only test the state machine, but also
test the ABCI lifecycle of a block.

Statistics, such as total blocks and total failed proposals, will be collected,
logged and written to output after the full or partial execution of a simulation.
The output destination of these statistics will be configurable.

```go
func (s *Simulator) SimulateBlock() {
  rProposer := s.SelectRandomProposer()
  rTxs := s.SelectTxs()

  prepareResp, err := s.app.PrepareProposal(&abci.PrepareProposalRequest{Txs: rTxs})
  // handle error

  processResp, err := s.app.ProcessProposal(&abci.ProcessProposalRequest{
    Txs: prepareResp.Txs,
    // ...
  })
  // handle error

  // execute liveness matrix...

  _, err = s.app.FinalizeBlock(...)
  // handle error
  
  _, err = s.app.Commit(...)
  // handle error
}
```

Note, some applications do not define or need their own app-side mempool, so we
propose that `SelectTxs` mimic CometBFT and just return FIFO-ordered transactions
from an ad-hoc simulator mempool. In the case where an application does define
its own mempool, it will simply ignore what is provided in `RequestPrepareProposal`.

### Profiling

The manager will be responsible for collecting CPU and RAM profiles of the simulation
execution. We propose to use [Pyroscope](https://pyroscope.io/docs/golang/) to
capture profiles and export them to a local file and via an HTTP endpoint. This
can be disabled or enabled by configuration.

### Breakpoints

Via configuration, a caller can express a height-based breakpoint that will allow
the simulation to stop and resume from a given height. This will allow for debugging
of CPU, RAM, and state.

### Validity Predicates

We propose to provide the ability for an application to provide the simulator a
set of validity predicates, i.e. invariant checkers, that will be executed before
and after each block. This will allow for the application to assert that certain
state invariants are held before and after each block. Note, as a consequence of
this, we propose to remove the existing notion of invariants from module production
execution paths and deprecate their usage altogether.

```go
type Manager struct {
  // ...
  preBlockValidator   func(sdk.Context) error
  postBlockValidator  func(sdk.Context) error
}
```

## Consequences

### Backwards Compatibility

The new simulator package will not naturally not be backwards compatible with the
existing `x/simulation` module. However, modules will still be responsible for
providing pseudo-random transactions to the simulator.

### Positive

* Providing more intuitive and cleaner APIs for application developers
* More closely resembling the true lifecycle of an ABCI application

### Negative

* Breaking current Cosmos SDK module APIs for transaction generation

## References

* [Osmosis Simulation ADR](https://github.com/osmosis-labs/osmosis/blob/main/simulation/ADR.md)
