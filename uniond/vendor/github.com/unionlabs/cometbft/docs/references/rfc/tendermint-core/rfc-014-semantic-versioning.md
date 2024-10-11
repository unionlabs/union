# RFC 014: Semantic Versioning

## Changelog

- 2021-11-19: Initial Draft
- 2021-02-11: Migrate RFC to tendermint repo (Originally [RFC 006](https://github.com/tendermint/spec/pull/365))

## Author(s)

- Callum Waters @cmwaters

## Context

We use versioning as an instrument to hold a set of promises to users and signal when such a set changes and how. In the conventional sense of a Go library, major versions signal that the public Go API’s have changed in a breaking way and thus require the users of such libraries to change their usage accordingly. Tendermint is a bit different in that there are multiple users: application developers (both in-process and out-of-process), node operators, and external clients. More importantly, both how these users interact with Tendermint and what's important to these users differs from how users interact and what they find important in a more conventional library.

This document attempts to encapsulate the discussions around versioning in Tendermint and draws upon them to propose a guide to how Tendermint uses versioning to make promises to its users.

For a versioning policy to make sense, we must also address the intended frequency of breaking changes. The strictest guarantees in the world will not help users if we plan to break them with every release.

Finally I would like to remark that this RFC only addresses the "what", as in what are the rules for versioning. The "how" of Tendermint implementing the versioning rules we choose, will be addressed in a later RFC on Soft Upgrades.

## Discussion

We first begin with a round up of the various users and a set of assumptions on what these users expect from Tendermint in regards to versioning:

1. **Application Developers**, those that use the ABCI to build applications on top of Tendermint, are chiefly concerned with that API. Breaking changes will force developers to modify large portions of their codebase to accommodate for the changes. Some ABCI changes such as introducing priority for the mempool don't require any effort and can be lazily adopted whilst changes like ABCI++ may force applications to redesign their entire execution system. It's also worth considering that the API's for go developers differ to developers of other languages. The former here can use the entire Tendermint library, most notably the local RPC methods, and so the team must be wary of all public Go API's.
2. **Node Operators**, those running node infrastructure, are predominantly concerned with downtime, complexity and frequency of upgrading, and avoiding data loss. They may be also concerned about changes that may break the scripts and tooling they use to supervise their nodes.
3. **External Clients** are those that perform any of the following:
     - consume the RPC endpoints of nodes like `/block`
     - subscribe to the event stream
     - make queries to the indexer

    This set are concerned with chain upgrades which will impact their ability to query state and block data as well as broadcast transactions. Examples include wallets and block explorers.

4. **IBC module and relayers**. The developers of IBC and consumers of their software are concerned about changes that may affect a chain's ability to send arbitrary messages to another chain. Specifically, these users are affected by any breaking changes to the light client verification algorithm.

Although we present them here as having different concerns, in a broader sense these user groups share a concern for the end users of applications. A crucial principle guiding this RFC is that **the ability for chains to provide continual service is more important than the actual upgrade burden put on the developers of these chains**. This means some extra burden for application developers is tolerable if it minimizes or substantially reduces downtime for the end user.

### Modes of Interprocess Communication

Tendermint has two primary mechanisms to communicate with other processes: RPC and P2P. The division marks the boundary between the internal and external components of the network:

- The P2P layer is used in all cases that nodes (of any type) need to communicate with one another.
- The RPC interface is for any outside process that wants to communicate with a node.

The design principle here is that **communication via RPC is to a trusted source** and thus the RPC service prioritizes inspection rather than verification. The P2P interface is the primary medium for verification.

As an example, an in-browser light client would verify headers (and perhaps application state) via the p2p layer, and then pass along information on to the client via RPC (or potentially directly via a separate API).

The main exceptions to this are the IBC module and relayers, which are external to the node but also require verifiable data. Breaking changes to the light client verification path mean that all neighbouring chains that are connected will no longer be able to verify state transitions and thus pass messages back and forward.

## Proposal

Tendermint version labels will follow the syntax of [Semantic Versions 2.0.0](https://semver.org/) with a major, minor and patch version. The version components will be interpreted according to these rules:

For the entire cycle of a **major version** in Tendermint:

- All blocks and state data in a blockchain can be queried. All headers can be verified even across minor version changes. Nodes can both block sync and state sync from genesis to the head of the chain.
- Nodes in a network are able to communicate and perform BFT state machine replication so long as the agreed network version is the lowest of all nodes in a network. For example, nodes using version 1.5.x and 1.2.x can operate together so long as the network version is 1.2 or lower (but still within the 1.x range). This rule essentially captures the concept of network backwards compatibility.
- Node RPC endpoints will remain compatible with existing external clients:
    - New endpoints may be added, but old endpoints may not be removed.
    - Old endpoints may be extended to add new request and response fields, but requests not using those fields must function as before the change.
- Migrations should be automatic. Upgrading of one node can happen asynchronously with respect to other nodes (although agreement of a network-wide upgrade must still occur synchronously via consensus).

For the entire cycle of a **minor version** in Tendermint:

- Public Go API's, for example in `node` or `abci` packages will not change in a way that requires any consumer (not just application developers) to modify their code.
- No breaking changes to the block protocol. This means that all block related data structures should not change in a way that breaks any of the hashes, the consensus engine or light client verification.
- Upgrades between minor versions may not result in any downtime (i.e., no migrations are required), nor require any changes to the config files to continue with the existing behavior. A minor version upgrade will require only stopping the existing process, swapping the binary, and starting the new process.

A new **patch version** of Tendermint will only contain bug fixes and updates that impact the security and stability of Tendermint.

These guarantees will come into effect at release 1.0.

## Status

Proposed

## Consequences

### Positive

- Clearer communication of what versioning means to us and the effect they have on our users.

### Negative

- Can potentially incur greater engineering effort to uphold and follow these guarantees.

### Neutral

## References

- [SemVer](https://semver.org/)
- [Tendermint Tracking Issue](https://github.com/tendermint/tendermint/issues/5680)
