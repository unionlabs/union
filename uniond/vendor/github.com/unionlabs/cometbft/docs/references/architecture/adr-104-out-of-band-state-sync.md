# ADR 104: State sync from local application snapshot

## Changelog

- 2023-05-05: Initial Draft (@cason)
- 2023-24-05: Added description of SDK-based & a CometBFT based solution (@jmalicevic)

## Status

Accepted

## Context

1. What is state sync?

From CometBFT v0.34, synchronizing a fresh node with the rest of the network
can benefit from [state sync][state-sync], a protocol for discovering,
fetching, and installing application snapshots.
State sync is able to rapidly bootstrap a node by installing a relatively
recent state machine snapshot, instead of replaying all historical blocks.

2. What is the problem?

With the widespread adoption of state sync to bootstrap nodes, however,
what should be one of its strengths - the ability to discover and fetch
application snapshots from peers in the p2p network - has turned out to be one
of its weaknesses.
In fact, while downloading recent snapshots is very convenient for new nodes
(clients of the protocol), providing snapshots to multiple peers (as servers of
the protocol) is _bandwidth-consuming_, especially without a clear incentive for
node operators to provide this service.
As a result, the number of nodes in production CometBFT networks offering the
state sync service (i.e., servers offering snapshots) has been limited, which
has rendered the service _fragile_ (from the client's point of view). In other words, it is very
hard to find a node with _good_ snapshots, leading nodes to often block during sync up.

3. High level idea behind the solution

This ADR stems from the observation that state sync is more than a protocol to
discover and fetch snapshots from peers in the network.
In fact, in addition to installing a snapshot, state sync also checks the
consistency of the installed application state with the state (`appHash`)
recorded on the blockchain at the same height, and bootstraps CometBFT's block
store and state store accordingly.
As a result, once state sync is completed the node can safely switch to Block
sync and/or Consensus to catch up with the latest state of the network.

The purpose of this ADR is to provide node operators with more flexibility in
defining how or where state sync should look for application snapshots.
More precisely, it enables state sync to support the bootstrap of nodes from
application snapshots obtained _out-of-band_ by operators, available to the
node _locally_.
Applications dump snapshots into an exportable format, which can be then obtained
by the operators and placed on the syncing node. The node can then sync locally
without transferring snapshots via the network.
The goal is to provide an alternative to the mechanism currently adopted by
state sync, discovering and fetching application snapshots from peers in the
network, in order to address the above mentioned limitations, while preserving
most of state sync's operation.

The ADR presents two solutions:

1. The first solution is implemented by the application and was proposed and
implemented by the Cosmos SDK (PR [#16067][sdk-pr2] and [#16061][sdk-pr1] ). This ADR describes the solution
in a general way, proividing guidelines to non-SDK based applications if they wish
to implement their own local state sync.
2. The second part of the ADR proposes a more general solution, that uses ABCI
to achieve the same behavior provided by the SDK's solution.


## Alternative Approaches

### Strengthen p2p-based state sync

This ADR proposes a workaround for some limitations of the existing state sync
protocol, briefly summarized above. A more comprehensive and sustainable
long-term solution would require addressing such limitations, by improving the
current p2p based state sync solution.

From a protocol design point of view, probably the major limitation of state
sync is the lack of mechanisms to incentivize and reward established nodes that
provide "good" (updated and consistent) snapshots to peers.
The existing protocol is essentially altruistic: it assumes that established
nodes will support the bootstrap of fresh nodes without receiving anything in
return, other than having new peers joining their network.

From an operational point of view, the design of the p2p layer should take into
consideration the fact that not every node in the network is a "good" state
sync server. Fresh nodes, which need support for bootstrapping, should then be
able to discover peers in the network that are able or willing to provide
application snapshots.
The implementation of this feature would require a more complex content-based
peer discovery mechanism.

However, while relevant, strengthening p2p-based state sync should be seen as
_orthogonal_ to the solution proposed in this ADR, which should have relevant use
cases even for an enhanced version of state sync.

### Manual application state transfer

The proposed [ADR 083][adr083]/[ADR 103][adr103] inspired the discussions that
led to this ADR. At first glance, their proposal might look identical to the
solution discussed here. The main distinction is that these previous proposals do not
involve installing application _snapshots_, but rely on node operators to
manually synchronize the application _state_.

More precisely, their proposal is to
"_allow applications to start with a bootstrapped state_
(i.e., the application has already a state before the node is started)
_alongside an empty CometBFT instance_
(i.e., the node's block store is empty and the state store is at genesis state)
_using a subsystem of the state sync protocol_".

Starting from an "empty CometBFT instance" is a requirement for running state
sync, which is maintained in the current proposal.
The mentioned "subsystem of the state sync protocol" is the light client,
responsible for verifying the application state and for bootstrapping
CometBFT's block store and state store accordingly.

The distinction, therefore, lies in the way which the state of the application
from a running node is transferred to the new node to be bootstrapped.
Instead of relying on application snapshots, produced by the application but
handled by state sync, [ADR 083][adr083]/[ADR 103][adr103] assumes that node
operators are able to manually synchronize the application state from a running
node (it might be necessary to stop it) to a not-yet-started fresh node.

The main limitation of the approach in [ADR 083][adr083]/[ADR 103][adr103]
is that it relies on the ability of node
operators to properly synchronize the application state between two nodes.
While experienced node operators are likely able to perform this operation in a
proper way, we have to consider a broader set of users and emphasize that it is
an operation susceptible to errors. Operators need to know the format of the
application's stored state, export the state properly and guarantee consistency. That is,
they have to make sure that nobody is trying to read the exported state while they are
exporting it.
Furthermore, it is an operation that is, by definition, application-specific:
applications are free to manage their state as they see fit, and this includes
how and where the state is persisted.
Node operators would therefore need to know the specifics of each application
in order to adopt this solution.


## Decision

State sync should support the bootstrap of new nodes from application snapshots
available locally. Implementing this option does not mean that networked State sync
should be removed, but not both should be enabled at the same time.

In other words, the following operation should be, in general terms, possible:

1. When configuring a new node, operators should be able to define where (e.g.,
   a local file system location) the client side of state sync should look for
   application snapshots to install or restore;
2. Operators should be able to instruct the server side of State sync on a
   running node to export application snapshots, when they are available,
   to a given location (e.g., at the local file system);
3. It is up to node operators to transfer application snapshots produced by the
   running node (server side) to the new node (client side) to be bootstrapped
   using this new State sync feature.



As Cosmos SDK implemented a solution for all applications using it, and we do not have
non-SDK in production users requesting this feature, at the moment we will not implement the generally applicable
solution.

This ADR will outline both the solution implemented by the SDK and a design proposal of a
generally applicable solution, that can be later on picked up and implemented in CometBFT.

## Detailed Design

### Application-implemented local state sync

This section describes the solution to local state sync implemented by the SDK. An application can
chose to implement local state sync differently, or implement only a subset of the functionalities
implemented by the SDK.

This solution exposes a command line interface enabling a node to manipulate the snapshots
including dumping existing snapshots to an exportable format, loading, restoring and deleting exported snapshots,
as well as a command to bootstrap the node by resetting CometBFT's state and block store.

The SDK exposes the following commands for snapshot manipulation:

```script

delete      Delete a snapshot from the local snapshot store
dump        Dump the snapshot as portable archive format
export      Export app state to snapshot store
list        List local snapshots
load        Load a snapshot archive file into snapshot store
restore     Restore app state from a snapshot stored in the local snapshot store

```

and the following command to bootstrap the state of CometBFT upon installing a snapshot:

``` script

comet bootstrap-state          Bootstrap the state of CometBFT's state and block store from the
                               application's state

```

These commands enable the implementation of both the client and server side of statesync.
Namely, a statesync server can use `dump` to create a portable archive format out existing snapshots,
or trigger snapshot creation using `export`.

The client side, restores the application state from a local snapshot that was previously exported, using
`restore`. Before `restore` can be called, the client has to `load` an archived snapshot into its
local snapshot store.
Upon successful completion of the previous sequence of commands, the state of CometBFT is bootstrapped
using `bootstrap-state` and CometBFT can be launched.


There are three prerequisites for this solution to work when a node is syncing:

1. The application has access to the snapshot store (usually as a separate database used by applications)
2. CometBFT's state and block stores are empty or reset
3. CometBFT is not running while the node is state syncing

The server side of state sync (snapshot generation and dumping), can be performed while the node is running.
The application has to be careful not to interfere with normal node operations, and to use a snapshot store
and dumping mechanism that will mitigate the risk of requesting snapshots while they are being dumped to an archive format.

In order to be able to dump or export the snapshots, the application must have access to the snapshot store.

We describe the main interface expected from the snapshot database and used by the above mentioned CLI commands
for snapshot manipulation.
The interface was derived from the SDK's implementation of local State sync.

```golang

// Delete a snapshot for a certain height
func (s *Store) Delete(height uint64, format uint32) error

// Retrieves a snapshot for a certain height
func (s *Store) Get(height uint64, format uint32) (*Snapshot, error)

// List recent snapshots, in reverse order (newest first)
func (s *Store) List() ([]*Snapshot, error)

// Loads a snapshot (both metadata and binary chunks). The chunks must be consumed and closed.
// Returns nil if the snapshot does not exist.
func (s *Store) Load(height uint64, format uint32) (*Snapshot, <-chan io.ReadCloser, error)

// LoadChunk loads a chunk from disk, or returns nil if it does not exist. The caller must call
// Close() on it when done.
func (s *Store) LoadChunk(height uint64, format, chunk uint32) (io.ReadCloser, error)

// Save saves a snapshot to disk, returning it.
func (s *Store) Save(height uint64, format uint32, chunks <-chan io.ReadCloser) (*Snapshot, error)

// PathChunk generates a snapshot chunk path.
func (s *Store) PathChunk(height uint64, format, chunk uint32) string

```

In order to dump a snapshot, an application needs to retrieve all the chunks stored at a certain path.

#### CometBFT state bootstrap

In addition to managing snapshots, it is necessary to bootstrap (setup) the state and block store of CometBFT before starting up the node.
Upon a successful start, CometBFT performs block sync and consensus.
At the moment of writing this ADR, there is no command line in CometBFT that supports this, but an [issue][state-bootstrap]
has been opened to address this.
Until it has been resolved, the application developers have to, within their bootstrapping command:

- Create a state and block store
- Launch a light client to obtain and verify the block header for the snapshot's height.
- Use the light client's `State` to
verify that the `AppHash` on the retrieved block header matches the `AppHash` obtained via
the ABCI `Info` call to the application (the same procedure is performed by the node on startup).
- Use `Commit` to retrieve the last commit for the snapshot's height.
- Save the retrieved values into the state and block stores.

This code essentially mimics what CometBFT does as part of node setup, once state sync is complete.


### CometBFT based local state sync

Given that snapshot manipulation is entirely application defined, and to avoid pulling this functionality into
CometBFT, we propose a solution using ABCI, that mimics the behaviour described in the previous section.

On the client side, the main difference between local State sync done by the application and CometBFT is that the
application has to perform the sync offline, in order to properly set up CometBFT's initial state. Furthermore, the
application developer has to manually bootstrap CometBFTs state and block stores.
With support for local State sync, a node can simply load a snapshot from a predefined location and offer it to the application
as is currently done via networked State sync.

On the server side, without any support for local State sync, an operator has to manually instruct the application
to export the snapshots into a portable format (via `dump`).

Having support for this within CometBFT, the app can automatically perform this export when taking snapshots.

In order to support local State sync, the following changes to CometBFT are necessary:

1. Adding new configuration options to the config file.
2. Introduce a CLI command that can explicitly tell the application to create a snapshot export, in case
operators decide not to generate periodical exports.
3. Extract a snapshot from the exported format.
4. Potentially alter existing ABCI calls to signal to the application that we want to create a snapshot export periodically.
5. Allow reading a snapshot from a compressed format into CometBFT and offer it to the application via
the existing `OfferSnapshot` ABCI call.


At a very high level, there are two possible solutions and we will present both:

1. The application will export snapshots into an exportable format on the server side. When a node syncs up, CometBFT will
send this as a blob of bytes to the application to uncompress and apply the snapshot.

2. CometBFT creates a snapshot using existing ABCI calls and exports it into a format of our choosing. CometBFT is then in charge of
reading in and parsing the exported snapshot into a snapshot that can be offered to the application via the existing `OfferSnapshot`
and `ApplyChunk` methods. This option does not alter any current APIs and is a good candidate for an initial implementation.


#### Config file additions

```bash
[statesync]
# State syncing from a local snapshot
local_sync=false
# Path to snapshot, will be ignored if local_sync=false
snapshot_load_path=""
# Periodically dump snapshots into archive format (optional)
auto_snapshot_dump=false
# If dumping nodes into archive format, set the path to where the file is dumped
# and the file format
# This can be changed when using a CLI command
snapshot_dump_path=""
snapshot_dump_format=""
```

#### *CLI command for application snapshot management*

CometBFT exposes a CLI command to instruct the application to dump the existing snapshots into an exportable format:

```bash
dump      Dump existing snapshots to exportable file format.
```

We could expand the CLI interface to allow for additional operations, similar to the CLI designed for Cosmos SDK. However,
as snapshot generation, dumping and loading a local snapshot would be built into CometBFT,
 while the node is running, we do not need to rely on a CLI for this functionality.

The `dump` command can be implemented in two ways:

1. Rely on the existing ABCI functions `ListSnapshots` and `LoadChunks` to retrieve the snapshots and chunks from a peer.
This approach requires no change to the current API and is easy to implement. Furthermore, CometBFT has complete control
over the format of the exported snapshot. It does however involve more than one ABCI call and network data transfer.
2.  Extend `RequestListSnapshots` with a flag to indicate that we want an exportable snapshot format and extend `ResponseListSnapshot` to return a
path and format of the exported snapshots.

An improvement to the second option mentioned above would be to add path parameters to the command,
 and include the path into `RequestListSnapshots` instructing the application to generate a snapshot export
at a given location.

A third option is the introduction of a new ABCI call: `ExportSnapshots`, which will have the same effect as option 2 above.


#### *Automatic snapshot exporting*

Applications generate snapshots with a regular cadence (fixed time intervals or heights). The application itself measures the time or number of heights passed since the last snapshot,
and CometBFT has no role in instructing the application when to take snapshots.

The State sync reactor currently retrieves a list of snapshots from a peer, who obtains these snapshots from the local instance of the application using the ABCI `RequestListSnapshots` call.

Applications can thus themselves be in charge of dumping the snapshots into a given file format, the same way they generate snapshots.
 If `auto_snapshot_dump` is true,
CometBFT instructs the application to export the snapshots periodically.

An alternative solution is that CometBFT, itself, using the implementation of the `dump` command, whichever
is chosen at the time, creates or asks the application to create snapshot exports. This is not forcing the
application to create a snapshot at the time of he request, rather *dumps* existing snapshots into
an exportable file format.

**Export file consistency**

The same caveat of making sure the file into which the snapshot is exported is not corrupted by concurrent reads and writes applies here as well. If we decide to simply export
snapshots into a compressed file, we need to make sure that we do not return files that with ongoing writes.
An alternative is for CometBFT to provide a Snapshot export store with snapshot isolation. This store would essentially be pruned as soon as a new snapshot is written,
thus not taking too much more space.

If it is the application that exports the snapshots, it is something the application developer has to be aware of.

#### Syncing a node using local snapshots

On startup, if `local_sync` is set to `true`, CometBFT will look for an existing snapshot at the path
given by the operator. If a snapshot is available, it will be loaded and state restored as if it came from a peer in the current implementation.

Note that, if it is not CometBFT that created the snapshot export from the data retrieved via ABCI (a combination of `ListSnapshots` and `LoadChunks`),
CometBFT might not be aware of how the snapshot was exported, and needs to ask the application to restore the snapshot.

If a snapshot was created using option 1 (by CometBFT) from the previous section, or the export format is known to CometBFT (like `tar, gzip` etc.),
CometBFT can extract the snapshot itself, and offer it to the application via `RequestOfferSnapshot` without any API changes.

If CometBFT is **not** the one who created the exported file, we introduce a new ABCI call `UnpackSnapshot`
to send the exported snapshot as a blob of bytes to the application, which uncompresses it, installs it
and responds whether the snapshot is accepted (as in `OfferSnapshot`) and the chunks application has passed (as in `LoadChunk`).


* **Request**:

    | Name            | Type    | Description                                 | Field Number |
    |-----------------|---------|---------------------------------------------|--------------|
    | exportedSnapshot| [] byte | Snapshot export created by the application. | 1            |

    Commit signals the application to persist application state. It takes no parameters.

* **Response**:

    | Name               | Type                                                    | Description                            | Field Number |
    |--------------------|-------------------------------------------------------- |----------------------------------------|--------------|
    | result             | [Result](../../../spec/abci/abci++_methods.md#result)  | The result of the snapshot offer.      | 1            |
    | resultChunk        | ResultC                                                 | The result of applying the chunks.     | 2            |

```proto
  enum ResultC {
    UNKNOWN         = 0;  // Unknown result, abort all snapshot restoration
    ACCEPT          = 1;  // The chunks were accepted.
    ABORT           = 2;  // Abort snapshot restoration, and don't try any other snapshots.
    RETRY_SNAPSHOT  = 3;  // Restart this snapshot from `OfferSnapshot`, reusing chunks unless instructed otherwise.
    REJECT_SNAPSHOT = 4;  // Reject this snapshot, try a different one.
  }
```

Unlike networked state sync, we do not re-fetch individual chunks, thus if the application of a chunk fails, then the application of the whole snapshot fails.

## Consequences

Adding the support for a node to sync up using a local snapshot can speed up the syncing process, especially as
network based State sync has proven to be fragile.

### Positive

- There is a possibility to implement this in a non-breaking manner: providing an alternative and complementary
  implementation for State sync
- Node operators will not need to use workarounds to make State sync to download
  application snapshots from specific nodes in the network, in particular from
  nodes that are controlled by the same operators

### Negative

- Implementing additional ABCI functions is API breaking and might not be backwards compatible.

### Neutral

- Additional complexity, with additional parameters for State sync's
  configuration and the bootstrap of a node
- Additional complexity, with the possible addition of a CLI command to save
  application snapshots to the file system

## References

- [State sync][state-sync] description, as part of ABCI++ spec
- Original issue on Tendermint Core repository: [statesync: bootstrap node with state obtained out-of-band #4642][original-issue]
- Original solution on Tendermint Core repository: [ADR 083: Supporting out of band state sync #9651][adr083]
- Original proposal ported to CometBFT repository: [ADR 103: Local State Sync Support][adr103]
- SDK's implementation of local state sync: [local snapshot store management][sdk-pr2] and [CometBFT state bootstrap][sdk-pr1]

[state-sync]: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_app_requirements.md#state-sync
[original-issue]: https://github.com/tendermint/tendermint/issues/4642
[adr083]: https://github.com/tendermint/tendermint/pull/9651
[adr103]: https://github.com/cometbft/cometbft/pull/729
[state-bootstrap]:  https://github.com/cometbft/cometbft/issues/884
[sdk-pr1]: https://github.com/cosmos/cosmos-sdk/pull/16061
[sdk-pr2]: https://github.com/cosmos/cosmos-sdk/pull/16067
