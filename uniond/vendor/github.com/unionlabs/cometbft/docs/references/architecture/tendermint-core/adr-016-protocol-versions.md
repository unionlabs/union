# ADR 016: Protocol Versions

## TODO

- How to / should we version the authenticated encryption handshake itself (ie.
  upfront protocol negotiation for the P2PVersion)
- How to / should we version ABCI itself? Should it just be absorbed by the
  BlockVersion?

## Changelog

- 18-09-2018: Updates after working a bit on implementation
    - ABCI Handshake needs to happen independently of starting the app
      conns so we can see the result
    - Add question about ABCI protocol version
- 16-08-2018: Updates after discussion with SDK team
    - Remove signalling for next version from Header/ABCI
- 03-08-2018: Updates from discussion with Jae:
  - ProtocolVersion contains Block/AppVersion, not Current/Next
  - signal upgrades to Tendermint using EndBlock fields
  - dont restrict peer compatibility by version to simplify syncing old nodes
- 28-07-2018: Updates from review
  - split into two ADRs - one for protocol, one for chains
  - include signalling for upgrades in header
- 16-07-2018: Initial draft - was originally joint ADR for protocol and chain
  versions

## Context

Here we focus on software-agnostic protocol versions.

The Software Version is covered by SemVer and described elsewhere.
It is not relevant to the protocol description, suffice to say that if any protocol version
changes, the software version changes, but not necessarily vice versa.

Software version should be included in NodeInfo for convenience/diagnostics.

We are also interested in versioning across different blockchains in a
meaningful way, for instance to differentiate branches of a contentious
hard-fork. We leave that for a later ADR.

## Requirements

We need to version components of the blockchain that may be independently upgraded.
We need to do it in a way that is scalable and maintainable - we can't just litter
the code with conditionals.

We can consider the complete version of the protocol to contain the following sub-versions:
BlockVersion, P2PVersion, AppVersion. These versions reflect the major sub-components
of the software that are likely to evolve together, at different rates, and in different ways,
as described below.

The BlockVersion defines the core of the blockchain data structures and
should change infrequently.

The P2PVersion defines how peers connect and communicate with each other - it's
not part of the blockchain data structures, but defines the protocols used to build the
blockchain. It may change gradually.

The AppVersion determines how we compute app specific information, like the
AppHash and the Results.

All of these versions may change over the life of a blockchain, and we need to
be able to help new nodes sync up across version changes. This means we must be willing
to connect to peers with older version.

### BlockVersion

- All tendermint hashed data-structures (headers, votes, txs, responses, etc.).
  - Note the semantic meaning of a transaction may change according to the AppVersion, but the way txs are merklized into the header is part of the BlockVersion
- It should be the least frequent/likely to change.
  - Tendermint should be stabilizing - it's just Atomic Broadcast.
  - We can start considering for Tendermint v2.0 in a year
- It's easy to determine the version of a block from its serialized form

### P2PVersion

- All p2p and reactor messaging (messages, detectable behavior)
- Will change gradually as reactors evolve to improve performance and support new features - eg proposed new message types BatchTx in the mempool and HasBlockPart in the consensus
- It's easy to determine the version of a peer from its first serialized message/s
- New versions must be compatible with at least one old version to allow gradual upgrades

### AppVersion

- The ABCI state machine (txs, begin/endblock behavior, commit hashing)
- Behaviour and message types will change abruptly in the course of the life of a chain
- Need to minimize complexity of the code for supporting different AppVersions at different heights
- Ideally, each version of the software supports only a _single_ AppVersion at one time
  - this means we checkout different versions of the software at different heights instead of littering the code
    with conditionals
  - minimize the number of data migrations required across AppVersion (ie. most AppVersion should be able to read the same state from disk as previous AppVersion).

## Ideal

Each component of the software is independently versioned in a modular way and its easy to mix and match and upgrade.

## Proposal

Each of BlockVersion, AppVersion, P2PVersion, is a monotonically increasing uint64.

To use these versions, we need to update the block Header, the p2p NodeInfo, and the ABCI.

### Header

Block Header should include a `Version` struct as its first field like:

```
type Version struct {
    Block uint64
    App uint64
}
```

Here, `Version.Block` defines the rules for the current block, while
`Version.App` defines the app version that processed the last block and computed
the `AppHash` in the current block. Together they provide a complete description
of the consensus-critical protocol.

Since we have settled on a proto3 header, the ability to read the BlockVersion out of the serialized header is unanimous.

Using a Version struct gives us more flexibility to add fields without breaking
the header.

The ProtocolVersion struct includes both the Block and App versions - it should
serve as a complete description of the consensus-critical protocol.

### NodeInfo

NodeInfo should include a Version struct as its first field like:

```
type Version struct {
    P2P uint64
    Block uint64
    App uint64

    Other []string
}
```

Note this effectively makes `Version.P2P` the first field in the NodeInfo, so it
should be easy to read this out of the serialized header if need be to facilitate an upgrade.

The `Version.Other` here should include additional information like the name of the software client and
it's SemVer version - this is for convenience only. Eg.
`tendermint-core/v0.22.8`. It's a `[]string` so it can include information about
the version of Tendermint, of the app, of Tendermint libraries, etc.

### ABCI

Since the ABCI is responsible for keeping Tendermint and the App in sync, we
need to communicate version information through it.

On startup, we use Info to perform a basic handshake. It should include all the
version information.

We also need to be able to update versions in the life of a blockchain. The
natural place to do this is EndBlock.

Note that currently the result of the Handshake isn't exposed anywhere, as the
handshaking happens inside the `proxy.AppConns` abstraction. We will need to
remove the handshaking from the `proxy` package so we can call it independently
and get the result, which should contain the application version.

#### Info

RequestInfo should add support for protocol versions like:

```
message RequestInfo {
  string version
  uint64 block_version
  uint64 p2p_version
}
```

Similarly, ResponseInfo should return the versions:

```
message ResponseInfo {
  string data

  string version
  uint64 app_version

  int64 last_block_height
  bytes last_block_app_hash
}
```

The existing `version` fields should be called `software_version` but we leave
them for now to reduce the number of breaking changes.

#### EndBlock

Updating the version could be done either with new fields or by using the
existing `tags`. Since we're trying to communicate information that will be
included in Tendermint block Headers, it should be native to the ABCI, and not
something embedded through some scheme in the tags. Thus, version updates should
be communicated through EndBlock.

EndBlock already contains `ConsensusParams`. We can add version information to
the ConsensusParams as well:

```
message ConsensusParams {

  BlockSize block_size
  EvidenceParams evidence_params
  VersionParams version
}

message VersionParams {
    uint64 block_version
    uint64 app_version
}
```

For now, the `block_version` will be ignored, as we do not allow block version
to be updated live. If the `app_version` is set, it signals that the app's
protocol version has changed, and the new `app_version` will be included in the
`Block.Header.Version.App` for the next block.

### BlockVersion

BlockVersion is included in both the Header and the NodeInfo.

Changing BlockVersion should happen quite infrequently and ideally only for
critical upgrades. For now, it is not encoded in ABCI, though it's always
possible to use tags to signal an external process to coordinate an upgrade.

Note Ethereum has not had to make an upgrade like this (everything has been at state machine level, AFAIK).

### P2PVersion

P2PVersion is not included in the block Header, just the NodeInfo.

P2PVersion is the first field in the NodeInfo. NodeInfo is also proto3 so this is easy to read out.

Note we need the peer/reactor protocols to take the versions of peers into account when sending messages:

- don't send messages they don't understand
- don't send messages they don't expect

Doing this will be specific to the upgrades being made.

Note we also include the list of reactor channels in the NodeInfo and already don't send messages for channels the peer doesn't understand.
If upgrades always use new channels, this simplifies the development cost of backwards compatibility.

Note NodeInfo is only exchanged after the authenticated encryption handshake to ensure that it's private.
Doing any version exchange before encrypting could be considered information leakage, though I'm not sure
how much that matters compared to being able to upgrade the protocol.

XXX: if needed, can we change the meaning of the first byte of the first message to encode a handshake version?
this is the first byte of a 32-byte ed25519 pubkey.

### AppVersion

AppVersion is also included in the block Header and the NodeInfo.

AppVersion essentially defines how the AppHash and LastResults are computed.

### Peer Compatibility

Restricting peer compatibility based on version is complicated by the need to
help old peers, possibly on older versions, sync the blockchain.

We might be tempted to say that we only connect to peers with the same
AppVersion and BlockVersion (since these define the consensus critical
computations), and a select list of P2PVersions (ie. those compatible with
ours), but then we'd need to make accommodations for connecting to peers with the
right Block/AppVersion for the height they're on.

For now, we will connect to peers with any version and restrict compatibility
solely based on the ChainID. We leave more restrictive rules on peer
compatibility to a future proposal.

### Future Changes

It may be valuable to support an `/unsafe_stop?height=_` endpoint to tell Tendermint to shutdown at a given height.
This could be use by an external manager process that oversees upgrades by
checking out and installing new software versions and restarting the process. It
would subscribe to the relevant upgrade event (needs to be implemented) and call `/unsafe_stop` at
the correct height (of course only after getting approval from its user!)

## Consequences

### Positive

- Make tendermint and application versions native to the ABCI to more clearly
  communicate about them
- Distinguish clearly between protocol versions and software version to
  facilitate implementations in other languages
- Versions included in key data structures in easy to discern way
- Allows proposers to signal for upgrades and apps to decide when to actually change the
  version (and start signalling for a new version)

### Neutral

- Unclear how to version the initial P2P handshake itself
- Versions aren't being used (yet) to restrict peer compatibility
- Signalling for a new version happens through the proposer and must be
  tallied/tracked in the app.

### Negative

- Adds more fields to the ABCI
- Implies that a single codebase must be able to handle multiple versions
