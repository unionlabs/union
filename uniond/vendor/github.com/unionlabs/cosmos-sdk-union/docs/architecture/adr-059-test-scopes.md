# ADR 059: Test Scopes

## Changelog

* 2022-08-02: Initial Draft
* 2023-03-02: Add precision for integration tests
* 2023-03-23: Add precision for E2E tests

## Status

PROPOSED Partially Implemented

## Abstract

Recent work in the SDK aimed at breaking apart the monolithic root go module has highlighted
shortcomings and inconsistencies in our testing paradigm. This ADR clarifies a common
language for talking about test scopes and proposes an ideal state of tests at each scope.

## Context

[ADR-053: Go Module Refactoring](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-053-go-module-refactoring.md) expresses our desire for an SDK composed of many
independently versioned Go modules, and [ADR-057: App Wiring](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-057-app-wiring.md) offers a methodology
for breaking apart inter-module dependencies through the use of dependency injection. As
described in [EPIC: Separate all SDK modules into standalone go modules](https://github.com/cosmos/cosmos-sdk/issues/11899), module
dependencies are particularly complected in the test phase, where simapp is used as
the key test fixture in setting up and running tests. It is clear that the successful
completion of Phases 3 and 4 in that EPIC require the resolution of this dependency problem.

In [EPIC: Unit Testing of Modules via Mocks](https://github.com/cosmos/cosmos-sdk/issues/12398) it was thought this Gordian knot could be
unwound by mocking all dependencies in the test phase for each module, but seeing how these
refactors were complete rewrites of test suites discussions began around the fate of the
existing integration tests. One perspective is that they ought to be thrown out, another is
that integration tests have some utility of their own and a place in the SDK's testing story.

Another point of confusion has been the current state of CLI test suites, [x/auth](https://github.com/cosmos/cosmos-sdk/blob/0f7e56c6f9102cda0ca9aba5b6f091dbca976b5a/x/auth/client/testutil/suite.go#L44-L49) for
example. In code these are called integration tests, but in reality function as end to end
tests by starting up a tendermint node and full application. [EPIC: Rewrite and simplify
CLI tests](https://github.com/cosmos/cosmos-sdk/issues/12696) identifies the ideal state of CLI tests using mocks, but does not address the
place end to end tests may have in the SDK.

From here we identify three scopes of testing, **unit**, **integration**, **e2e** (end to
end), seek to define the boundaries of each, their shortcomings (real and imposed), and their
ideal state in the SDK.

### Unit tests

Unit tests exercise the code contained in a single module (e.g. `/x/bank`) or package
(e.g. `/client`) in isolation from the rest of the code base. Within this we identify two
levels of unit tests, *illustrative* and *journey*. The definitions below lean heavily on
[The BDD Books - Formulation](https://leanpub.com/bddbooks-formulation) section 1.3.

*Illustrative* tests exercise an atomic part of a module in isolation - in this case we
might do fixture setup/mocking of other parts of the module.

Tests which exercise a whole module's function with dependencies mocked, are *journeys*.
These are almost like integration tests in that they exercise many things together but still
use mocks.

Example 1 journey vs illustrative tests - depinject's BDD style tests, show how we can
rapidly build up many illustrative cases demonstrating behavioral rules without [very much code](https://github.com/cosmos/cosmos-sdk/blob/main/depinject/binding_test.go) while maintaining high level readability.

Example 2 [depinject table driven tests](https://github.com/cosmos/cosmos-sdk/blob/main/depinject/provider_desc_test.go)

Example 3 [Bank keeper tests](https://github.com/cosmos/cosmos-sdk/blob/2bec9d2021918650d3938c3ab242f84289daef80/x/bank/keeper/keeper_test.go#L94-L105) - A mock implementation of `AccountKeeper` is supplied to the keeper constructor.

#### Limitations

Certain modules are tightly coupled beyond the test phase. A recent dependency report for
`bank -> auth` found 274 total usages of `auth` in `bank`, 50 of which are in
production code and 224 in test. This tight coupling may suggest that either the modules
should be merged, or refactoring is required to abstract references to the core types tying
the modules together. It could also indicate that these modules should be tested together
in integration tests beyond mocked unit tests.

In some cases setting up a test case for a module with many mocked dependencies can be quite
cumbersome and the resulting test may only show that the mocking framework works as expected
rather than working as a functional test of interdependent module behavior.

### Integration tests

Integration tests define and exercise relationships between an arbitrary number of modules
and/or application subsystems.

Wiring for integration tests is provided by `depinject` and some [helper code](https://github.com/cosmos/cosmos-sdk/blob/2bec9d2021918650d3938c3ab242f84289daef80/testutil/sims/app_helpers.go#L95) starts up
a running application. A section of the running application may then be tested. Certain
inputs during different phases of the application life cycle are expected to produce
invariant outputs without too much concern for component internals. This type of black box
testing has a larger scope than unit testing.

Example 1 [client/grpc_query_test/TestGRPCQuery](https://github.com/cosmos/cosmos-sdk/blob/2bec9d2021918650d3938c3ab242f84289daef80/client/grpc_query_test.go#L111-L129) - This test is misplaced in `/client`,
but tests the life cycle of (at least) `runtime` and `bank` as they progress through
startup, genesis and query time. It also exercises the fitness of the client and query
server without putting bytes on the wire through the use of [QueryServiceTestHelper](https://github.com/cosmos/cosmos-sdk/blob/2bec9d2021918650d3938c3ab242f84289daef80/baseapp/grpcrouter_helpers.go#L31).

Example 2 `x/evidence` Keeper integration tests - Starts up an application composed of [8
modules](https://github.com/cosmos/cosmos-sdk/blob/2bec9d2021918650d3938c3ab242f84289daef80/x/evidence/testutil/app.yaml#L1) with [5 keepers](https://github.com/cosmos/cosmos-sdk/blob/2bec9d2021918650d3938c3ab242f84289daef80/x/evidence/keeper/keeper_test.go#L101-L106) used in the integration test suite. One test in the suite
exercises [HandleEquivocationEvidence](https://github.com/cosmos/cosmos-sdk/blob/2bec9d2021918650d3938c3ab242f84289daef80/x/evidence/keeper/infraction_test.go#L42) which contains many interactions with the staking
keeper.

Example 3 - Integration suite app configurations may also be specified via golang (not
YAML as above) [statically](https://github.com/cosmos/cosmos-sdk/blob/release/v0.47.x/x/nft/testutil/app_config.go) or [dynamically](https://github.com/cosmos/cosmos-sdk/blob/8c23f6f957d1c0bedd314806d1ac65bea59b084c/tests/integration/bank/keeper/keeper_test.go#L129-L134).

#### Limitations

Setting up a particular input state may be more challenging since the application is
starting from a zero state. Some of this may be addressed by good test fixture
abstractions with testing of their own. Tests may also be more brittle, and larger
refactors could impact application initialization in unexpected ways with harder to
understand errors. This could also be seen as a benefit, and indeed the SDK's current
integration tests were helpful in tracking down logic errors during earlier stages
of app-wiring refactors.

### Simulations

Simulations (also called generative testing) are a special case of integration tests where
deterministically random module operations are executed against a running simapp, building
blocks on the chain until a specified height is reached. No *specific* assertions are
made for the state transitions resulting from module operations but any error will halt and
fail the simulation. Since `crisis` is included in simapp and the simulation runs
EndBlockers at the end of each block any module invariant violations will also fail
the simulation.

Modules must implement [AppModuleSimulation.WeightedOperations](https://github.com/cosmos/cosmos-sdk/blob/2bec9d2021918650d3938c3ab242f84289daef80/types/module/simulation.go#L31) to define their
simulation operations. Note that not all modules implement this which may indicate a
gap in current simulation test coverage.

Modules not returning simulation operations:

* `auth`
* `evidence`
* `mint`
* `params`


#### Limitations

* May take a long time to run, 7-10 minutes per simulation in CI.
* Timeouts sometimes occur on apparent successes without any indication why.
* Useful error messages not provided on from CI, requiring a developer to run
  the simulation locally to reproduce.

### E2E tests

End to end tests exercise the entire system as we understand it in as close an approximation
to a production environment as is practical. Presently these tests are located at
[tests/e2e](https://github.com/cosmos/cosmos-sdk/tree/main/tests/e2e) and rely on [testutil/network](https://github.com/cosmos/cosmos-sdk/tree/main/testutil/network) to start up an in-process Tendermint node.

An application should be built as minimally as possible to exercise the desired functionality.
The SDK uses an application will only the required modules for the tests. The application developer is advised to use its own application for e2e tests.

#### Limitations

In general the limitations of end to end tests are orchestration and compute cost.
Scaffolding is required to start up and run a prod-like environment and the this
process takes much longer to start and run than unit or integration tests.

Global locks present in Tendermint code cause stateful starting/stopping to sometimes hang
or fail intermittently when run in a CI environment.

The scope of e2e tests has been complected with command line interface testing.

## Decision

We accept these test scopes and identify the following decisions points for each.

| Scope       | App Type            | Mocks? |
| ----------- | ------------------- | ------ |
| Unit        | None                | Yes    |
| Integration | integration helpers | Some   |
| Simulation  | minimal app         | No     |
| E2E         | minimal app         | No     |

The decision above is valid for the SDK. An application developer should test their application with their full application instead of the minimal app.

### Unit Tests

All modules must have mocked unit test coverage.

Illustrative tests should outnumber journeys in unit tests.

Unit tests should outnumber integration tests.

Unit tests must not introduce additional dependencies beyond those already present in
production code.

When module unit test introduction as per [EPIC: Unit testing of modules via mocks](https://github.com/cosmos/cosmos-sdk/issues/12398)
results in a near complete rewrite of an integration test suite the test suite should be
retained and moved to `/tests/integration`. We accept the resulting test logic
duplication but recommend improving the unit test suite through the addition of
illustrative tests.

### Integration Tests

All integration tests shall be located in `/tests/integration`, even those which do not
introduce extra module dependencies.

To help limit scope and complexity, it is recommended to use the smallest possible number of
modules in application startup, i.e. don't depend on simapp.

Integration tests should outnumber e2e tests.

### Simulations

Simulations shall use a minimal application (usually via app wiring). They are located under `/x/{moduleName}/simulation`.

### E2E Tests

Existing e2e tests shall be migrated to integration tests by removing the dependency on the
test network and in-process Tendermint node to ensure we do not lose test coverage.

The e2e rest runner shall transition from in process Tendermint to a runner powered by
Docker via [dockertest](https://github.com/ory/dockertest).

E2E tests exercising a full network upgrade shall be written.

The CLI testing aspect of existing e2e tests shall be rewritten using the network mocking
demonstrated in [PR#12706](https://github.com/cosmos/cosmos-sdk/pull/12706).

## Consequences

### Positive

* test coverage is increased
* test organization is improved
* reduced dependency graph size in modules
* simapp removed as a dependency from modules
* inter-module dependencies introduced in test code are removed
* reduced CI run time after transitioning away from in process Tendermint

### Negative

* some test logic duplication between unit and integration tests during transition
* test written using dockertest DX may be a bit worse

### Neutral

* some discovery required for e2e transition to dockertest

## Further Discussions

It may be useful if test suites could be run in integration mode (with mocked tendermint) or
with e2e fixtures (with real tendermint and many nodes). Integration fixtures could be used
for quicker runs, e2e fixures could be used for more battle hardening.

A PoC `x/gov` was completed in PR [#12847](https://github.com/cosmos/cosmos-sdk/pull/12847)
is in progress for unit tests demonstrating BDD [Rejected].
Observing that a strength of BDD specifications is their readability, and a con is the
cognitive load while writing and maintaining, current consensus is to reserve BDD use
for places in the SDK where complex rules and module interactions are demonstrated.
More straightforward or low level test cases will continue to rely on go table tests.

Levels are network mocking in integration and e2e tests are still being worked on and formalized.
