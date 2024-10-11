# ADR 66: End-to-End Testing

## Changelog

- 2020-09-07: Initial draft (@erikgrinaker)
- 2020-09-08: Minor improvements (@erikgrinaker)
- 2021-04-12: Renamed from RFC 001 (@tessr)

## Authors

- Erik Grinaker (@erikgrinaker)

## Context

The current set of end-to-end tests under `test/` are very limited, mostly focusing on P2P testing in a standard configuration. They do not test various configurations (e.g. fast sync reactor versions, state sync, block pruning, genesis vs InitChain setup), nor do they test various network topologies (e.g. sentry node architecture). This leads to poor test coverage, which has caused several serious bugs to go unnoticed.

We need an end-to-end test suite that can run a large number of combinations of configuration options, genesis settings, network topologies, ABCI interactions, and failure scenarios and check that the network is still functional. This ADR outlines the basic requirements and design for such a system.

This ADR will not cover comprehensive chaos testing, only a few simple scenarios (e.g. abrupt process termination and network partitioning). Chaos testing of the core consensus algorithm should be implemented e.g. via Jepsen tests or a similar framework, or alternatively be added to these end-to-end tests at a later time. Similarly, malicious or adversarial behavior is out of scope for the first implementation, but may be added later.

## Proposal

### Functional Coverage

The following lists the functionality we would like to test:

#### Environments

- **Topology:** single node, 4 nodes (seeds and persistent), sentry architecture, NAT (UPnP)
- **Networking:** IPv4, IPv6
- **ABCI connection:** UNIX socket, TCP, gRPC
- **PrivVal:** file, UNIX socket, TCP

#### Node/App Configurations

- **Database:** goleveldb, cleveldb, boltdb, rocksdb, badgerdb
- **Fast sync:** disabled, v0, v2
- **State sync:** disabled, enabled
- **Block pruning:** none, keep 20, keep 1, keep random
- **Role:** validator, full node
- **App persistence:** enabled, disabled
- **Node modes:** validator, full, light, seed

#### Geneses

- **Validators:** none (InitChain), given
- **Initial height:** 1, 1000
- **App state:** none, given

#### Behaviors

- **Recovery:** stop/start, power cycling, validator outage, network partition, total network loss
- **Validators:** add, remove, change power
- **Evidence:** injection of DuplicateVoteEvidence and LightClientAttackEvidence

### Functional Combinations

Running separate tests for all combinations of the above functionality is not feasible, as there are millions of them. However, the functionality can be grouped into three broad classes:

- **Global:** affects the entire network, needing a separate testnet for each combination (e.g. topology, network protocol, genesis settings)

- **Local:** affects a single node, and can be varied per node in a testnet (e.g. ABCI/privval connections, database backend, block pruning)

- **Temporal:** can be run after each other in the same testnet (e.g. recovery and validator changes)

Thus, we can run separate testnets for all combinations of global options (on the order of 100). In each testnet, we run nodes with randomly generated node configurations optimized for broad coverage (i.e. if one node is using GoLevelDB, then no other node should use it if possible). And in each testnet, we sequentially and randomly pick nodes to stop/start, power cycle, add/remove, disconnect, and so on.

All of the settings should be specified in a testnet configuration (or alternatively the seed that generated it) such that it can be retrieved from CI and debugged locally.

A custom ABCI application will have to be built that can exhibit the necessary behavior (e.g. make validator changes, prune blocks, enable/disable persistence, and so on).

### Test Stages

Given a test configuration, the test runner has the following stages:

- **Setup:** configures the Docker containers and networks, but does not start them.

- **Initialization:** starts the Docker containers, performs fast sync/state sync. Accommodates for different start heights.

- **Perturbation:** adds/removes validators, restarts nodes, perturbs networking, etc - liveness and readiness checked between each operation.

- **Testing:** runs RPC tests independently against all network nodes, making sure data matches expectations and invariants hold.

### Tests

The general approach will be to put the network through a sequence of operations (see stages above), check basic liveness and readiness after each operation, and then once the network stabilizes run an RPC test suite against each node in the network.

The test suite will do black-box testing against a single node's RPC service. We will be testing the behavior of the network as a whole, e.g. that a fast synced node correctly catches up to the chain head and serves basic block data via RPC. Thus the tests will not send e.g. P2P messages or examine the node database, as these are considered internal implementation details - if the network behaves correctly, presumably the internal components function correctly. Comprehensive component testing (e.g. each and every RPC method parameter) should be done via unit/integration tests.

The tests must take into account the node configuration (e.g. some nodes may be pruned, others may not be validators), and should somehow be provided access to expected data (i.e. complete block headers for the entire chain).

The test suite should use the Tendermint RPC client and the Tendermint light client, to exercise the client code as well.

### Implementation Considerations

The testnets should run in Docker Compose, both locally and in CI. This makes it easier to reproduce test failures locally. Supporting multiple test-runners (e.g. on VMs or Kubernetes) is out of scope. The same image should be used for all tests, with configuration passed via a mounted volume.

There does not appear to be any off-the-shelf solutions that would do this for us, so we will have to roll our own on top of Docker Compose. This gives us more flexibility, but is estimated to be a few weeks of work.

Testnets should be configured via a YAML file. These are used as inputs for the test runner, which e.g. generates Docker Compose configurations from them. An additional layer on top should generate these testnet configurations from a YAML file that specifies all the option combinations to test.

Comprehensive testnets should run against master nightly. However, a small subset of representative testnets should run for each pull request, e.g. a four-node IPv4 network with state sync and fast sync.

Tests should be written using the standard Go test framework (and e.g. Testify), with a helper function to fetch info from the test configuration. The test runner will run the tests separately for each network node, and the test must vary its expectations based on the node's configuration.

It should be possible to launch a specific testnet and run individual test cases from the IDE or local terminal against a it.

If possible, the existing `testnet` command should be extended to set up the network topologies needed by the end-to-end tests.

## Status

Implemented

## Consequences

### Positive

- Comprehensive end-to-end test coverage of basic Tendermint functionality, exercising common code paths in the same way that users would

- Test environments can easily be reproduced locally and debugged via standard tooling

### Negative

- Limited coverage of consensus correctness testing (e.g. Jepsen)

- No coverage of malicious or adversarial behavior

- Have to roll our own test framework, which takes engineering resources

- Possibly slower CI times, depending on which tests are run

- Operational costs and overhead, e.g. infrastructure costs and system maintenance

### Neutral

- No support for alternative infrastructure platforms, e.g. Kubernetes or VMs

## References

- [#5291: new end-to-end test suite](https://github.com/tendermint/tendermint/issues/5291)
