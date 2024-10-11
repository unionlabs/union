# Upgrading CometBFT

This guide provides instructions for upgrading to specific versions of CometBFT.

## Unreleased

CometBFT `v1.0` includes some substantial breaking API changes that will hopefully
allow future changes to be rolled out quicker.

### Versioning

As of v1.0, the CometBFT team provides the following guarantees relating to
versioning:

- **Major version** bumps, such as v1.0.0 to v2.0.0, would generally involve
  changes that _force_ users to perform a coordinated upgrade in order to use
  the new version, such as protocol-breaking changes (e.g. changes to how block
  hashes are computed and thus what the network considers to be "valid blocks",
  or how the consensus protocol works, or changes that affect network-level
  compatibility between nodes, etc.).
- **Minor version** bumps, such as v1.1.0 to v1.2.0, are reserved for rolling
  out new features or substantial changes that do not force a coordinated
  upgrade (i.e. not protocol-breaking), but could potentially break Go APIs.
- **Patch version** bumps, such as v1.0.0 to v1.0.1, are reserved for
  bug/security fixes that are not protocol- or Go API-breaking.

### Building CometBFT

The minimum Go version has been bumped to [v1.22][go122].

### Proposer-Based Timestamps

CometBFT `v1.0` contains a new algorithm for generating and verifying block timestamps
called Proposer-Based Timestamps (PBTS).
The existing algorithm, called BFT-Time is kept for backwards compatibility.
Upgrading to `v1.0` does not automatically switch the chain from BFT-Time
to PBTS; rather a ConsensusParam called `PbtsEnableHeight` can be set to a future
height to transition from BFT-Time to PBTS.
This flexible mechanism allows chains disentagle the upgrade to `v1.0` from the transition
in the algorithm used for block times.
For further information, please check the [PBTS specification][pbts-spec].

### ABCI Mutex

CometBFT's existing ABCI local client is prevented from making
concurrent calls to ABCI implementations by virtue of a mutex taken
by the client's implementation.
In this release, two additional local ABCI clients have been added.
The first, supports one different mutex per ABCI connection
(consensus connection, mempool connection, etc.), allowing concurrency
in the application in different ABCI connections, but still serializing
ABCI calls within the same connection.
The second, totally removes mutexes from the ABCI client.
When using either of the new ABCI clients, the application is now
responsible to coordinate concurrent ABCI calls in order to prevent
race conditions or the possibility of non determinism.
If you are not sure how to ensure those guarantees in your application,
you are strongly advised to keep on using the existing ABCI client
containing one global mutex.

### Consensus

Removed the `consensus.State.ReplayFile` and `consensus.RunReplayFile` methods,
as these were exclusively used by the `replay` and `replay-console` subcommands,
which were also removed. (See
[\#1170](https://github.com/cometbft/cometbft/pull/1170))

### CLI Subcommands

- The `replay` and `replay-console` subcommands were removed
  ([\#1170](https://github.com/cometbft/cometbft/pull/1170)).

### Go API

As per [ADR 109](docs/references/architecture/adr-109-reduce-go-api-surface.md), the
following packages that were publicly accessible in CometBFT v0.38 were moved
into the `internal` directory:

- `blocksync`
- `consensus`
- `evidence`
- `inspect`
- `libs/async`
- `libs/autofile`
- `libs/bits`
- `libs/clist`
- `libs/cmap`
- `libs/events`
- `libs/fail`
- `libs/flowrate`
- `libs/net`
- `libs/os`
- `libs/progressbar`
- `libs/rand`
- `libs/strings`
- `libs/tempfile`
- `libs/timer`

If you rely on any of these packages and would like us to make them public
again, please [log an issue on
GitHub](https://github.com/cometbft/cometbft/issues/new/choose) describing your
use case and we will evaluate the best approach to helping you address it.

### Mempool

#### `nop` mempool

CometBFT v1.0.0 provides users with the option of a `nop` (no-op) mempool which,
if selected via configuration, turns off all mempool-related functionality in
Comet (e.g. ability to receive transactions, transaction gossip). Comet then
expects applications to provide their transactions when it calls
`PrepareProposal`, and that application developers will use some external means
of disseminating their transactions.

If you want to use it, change mempool's `type` to `nop` in your `config.toml`
file:

```toml
[mempool]

# The type of mempool for this node to use.
#
# Possible types:
# - "flood" : concurrent linked list mempool with flooding gossip protocol
# (default)
# - "nop"   : nop-mempool (short for no operation; the ABCI app is responsible
# for storing, disseminating and proposing txs). "create_empty_blocks=false"
# is not supported.
type = "nop"
```

#### Internal `CheckTx` Go API changes

The `Mempool` interface was modified on `CheckTx`. Note that this interface is
meant for internal use only, so you should be aware of these changes only if you
happen to call these methods directly.

`CheckTx`'s signature changed from
`CheckTx(tx types.Tx, cb func(*abci.ResponseCheckTx), txInfo TxInfo) error` to
`CheckTx(tx types.Tx, sender p2p.ID) (*abcicli.ReqRes, error)`.
The method used to take a callback function `cb` to be applied to the
ABCI `CheckTx` response and a `TxInfo` structure containing a sender.
Now the sender ID is passed directly and `CheckTx` returns the ABCI response
of type `*abcicli.ReqRes`, on which one can apply any callback manually.
For example:
```golang
reqRes, err := CheckTx(tx, sender)
// check `err` here
cb(reqRes.Response.GetCheckTx())
```

The `*abcicli.ReqRes` structure that `CheckTx` returns has a callback to
process the response already set (namely, the function `handleCheckTxResponse`).
The callback will be invoked internally when the response is ready. We need only 
to wait for it; for example:
```golang
reqRes, err := CheckTx(tx, sender)
// check `err` here
reqRes.Wait()
```

### Protobufs and Generated Go Code

Several major changes have been implemented relating to the Protobuf
definitions:

1. CometBFT now makes use of the `cometbft.*` Protobuf definitions in
   [`proto/cometbft`](./proto/cometbft/). This is a breaking change for all
   users who rely on serialization of the Protobuf type paths, such as
   integrators who serialize CometBFT's Protobuf data types into `Any`-typed
   fields. For example, the `tendermint.types.Block` type in CometBFT v0.38.x is
   now accessible as `cometbft.types.v1.Block` (see the next point in the list
   for details on versioning).

   See the CometBFT Protobufs [README](/proto/README.md) file for more details.

2. All CometBFT Protobuf packages include a version whose number will be
   independent of the CometBFT version. As mentioned in (1), the
   `tendermint.types.Block` type is now available under
   `cometbft.types.v1.Block` - the `v1` in the type path indicates the version
   of the `types` package used by this version of CometBFT.

   The Protobuf definitions that are wire-level compatible (but not type
   path-compatible) with CometBFT v0.34, v0.37 and v0.38, where breaking changes
   were introduced, are available under `v1beta*`-versioned types. For example:

   - The `tendermint.abci.Request` type from CometBFT v0.34 is now available as
     `cometbft.abci.v1beta1.Request`.
   - The `tendermint.abci.Request` type from CometBFT v0.37 is now available as
     `cometbft.abci.v1beta2.Request`.
   - The `tendermint.abci.Request` type from CometBFT v0.38 is now available as
     `cometbft.abci.v1beta3.Request`.

   See the CometBFT Protobufs [README](/proto/README.md) file for more details.

3. All Go code generated from the `cometbft.*` types is now available under the
   [`api`](./api/) directory. This directory is also an independently versioned
   Go module. This code is still generated using the Cosmos SDK's [gogoproto
   fork](https://github.com/cosmos/gogoproto) at present.

4. Several ABCI-related types were renamed in order to align with [Buf
   guidelines](https://buf.build/docs/best-practices/style-guide/). `Request*`
   and `Response*` were renamed to `*Request` and `*Response` (e.g.
   `RequestQuery` was renamed to `QueryRequest`). See the changelog for more
   details.

### RPC

- The RPC API is now versioned, with the existing RPC being available under both
  the `/` path (as in CometBFT v0.38) and a `/v1` path.

  Although invoking methods without specifying the version is still supported
  for now, support will be dropped in future releases and users are encouraged
  to use the versioned approach. For example, instead of
  `curl localhost:26657/block?height=5`, use `curl localhost:26657/v1/block?height=5`.

- The `/websocket` endpoint path is no longer configurable in the client or
  server. Creating an RPC client now takes the form:

  ```golang
  // The WebSocket endpoint in the following example is assumed to be available
  // at http://localhost:26657/v1/websocket
  rpcClient, err := client.New("http://localhost:26657/v1")
  ```

### Config Changes

- `consensus.skip_timeout_commit` was deprecated in favor of `consensus.timeout_commit=0s`.

## v0.38.0

This release introduces state machine-breaking changes, as well as substantial changes
on the ABCI interface and indexing. It therefore requires a
coordinated upgrade.

### Config Changes

- The field `Version` in the mempool section has been removed. The priority
  mempool (what was called version `v1`) has been removed (see below), thus
  there is only one implementation of the mempool available (what was called
  `v0`).
- Config fields `TTLDuration` and `TTLNumBlocks`, which were only used by the
  priority mempool, have been removed.

### Mempool Changes

- The priority mempool (what was referred in the code as version `v1`) has been
  removed. There is now only one mempool (what was called version `v0`), that
  is, the default implementation as a queue of transactions.
- In the protobuf message `ResponseCheckTx`, fields `sender`, `priority`, and
  `mempool_error`, which were only used by the priority mempool, were removed
  but still kept in the message as "reserved".

### ABCI Changes

- The `ABCIVersion` is now `2.0.0`.
- Added new ABCI methods `ExtendVote`, and `VerifyVoteExtension`.
  Applications upgrading to v0.38.0 must implement these methods as described
  [here](./spec/abci/abci%2B%2B_comet_expected_behavior.md#adapting-existing-applications-that-use-abci)
- Removed methods `BeginBlock`, `DeliverTx`, `EndBlock`, and replaced them by
  method `FinalizeBlock`. Applications upgrading to `v0.38.0` must refactor
  the logic handling the methods removed to handle `FinalizeBlock`.
- The Application's hash (or any data representing the Application's current state)
  is known by the time `FinalizeBlock` finishes its execution.
  Accordingly, the `app_hash` parameter has been moved from `ResponseCommit`
  to `ResponseFinalizeBlock`.
- Field `signed_last_block` in structure `VoteInfo` has been replaced by the
  more expressive `block_id_flag`. Applications willing to keep the semantics
  of `signed_last_block` can now use the following predicate
    - `voteInfo.block_id_flag != BlockIDFlagAbsent`
- For further details, please see the updated [specification](spec/abci/README.md)

### `block_results` RPC endpoint - query result display change (breaking)

- When returning a block, all block events are displayed within the `finalize_block_events` field.
 For blocks generated with older versions of CometBFT,  that means that block results that appeared
 as `begin_block_events` and `end_block_events` are merged into `finalize_block_events`.
 For users who rely on the events to be grouped by the function they were generated by, this change
 is breaking.

### kvindexer changes to indexing block events

The changes described here are internal to the implementation of the kvindexer, and they are transparent to the
user. However, if you own a fork with a modified version of the indexer, you should be aware of these changes.

- Indexer key for block events will not contain information about the function that returned the event.
The events were indexed by their attributes, event type, the function that returned them, the height and
event sequence. The functions returning events in old (pre `v0.38.0`) versions of CometBFT were `BeginBlock` or `EndBlock`.
As events are returned now only via `FinalizeBlock`, the value of this field has no use, and has been removed.
The main motivation is the reduction of the storage footprint.

Events indexed with previous CometBFT or Tendermint Core versions, will still be transparently processed.
There is no need to re-index the events. This function field is not exposed to queries, and was not
visible to users. However, if you forked CometBFT and changed the indexer code directly to accommodate for this,
this will impact your code.

## v0.37.0

This release introduces state machine-breaking changes, and therefore requires a
coordinated upgrade.

### Go API

When upgrading from the v0.34 release series, please note that the Go module has
now changed to `github.com/cometbft/cometbft`.

### ABCI Changes

- The `ABCIVersion` is now `1.0.0`.
- Added new ABCI methods `PrepareProposal` and `ProcessProposal`. For details,
  please see the [spec](spec/abci/README.md). Applications upgrading to
  v0.37.0 must implement these methods, at the very minimum, as described
  [here](./spec/abci/abci++_app_requirements.md)
- Deduplicated `ConsensusParams` and `BlockParams`.
  In the v0.34 branch they are defined both in `abci/types.proto` and `types/params.proto`.
  The definitions in `abci/types.proto` have been removed.
  In-process applications should make sure they are not using the deleted
  version of those structures.
- In v0.34, messages on the wire used to be length-delimited with `int64` varint
  values, which was inconsistent with the `uint64` varint length delimiters used
  in the P2P layer. Both now consistently use `uint64` varint length delimiters.
- Added `AbciVersion` to `RequestInfo`.
  Applications should check that CometBFT's ABCI version matches the one they expect
  in order to ensure compatibility.
- The `SetOption` method has been removed from the ABCI `Client` interface.
  The corresponding Protobuf types have been deprecated.
- The `key` and `value` fields in the `EventAttribute` type have been changed
  from type `bytes` to `string`. As per the [Protocol Buffers updating
  guidelines](https://developers.google.com/protocol-buffers/docs/proto3#updating),
  this should have no effect on the wire-level encoding for UTF8-encoded
  strings.

### RPC

If you rely on the `/tx_search` or `/block_search` endpoints for event querying,
please note that the default behaviour of these endpoints has changed in a way
that might break your queries. The original behaviour was poorly specified,
which did not respect event boundaries.

Please see
[tendermint/tendermint\#9712](https://github.com/tendermint/tendermint/issues/9712)
for context on the bug that was addressed that resulted in this behaviour
change.

## v0.34.27

This is the first official release of CometBFT, forked originally from
[Tendermint Core v0.34.24][v03424] and subsequently updated in Informal Systems'
public fork of Tendermint Core for [v0.34.25][v03425] and [v0.34.26][v03426].

### Upgrading from Tendermint Core

If you already make use of Tendermint Core (either the original Tendermint Core
v0.34.24, or Informal Systems' public fork), you can upgrade to CometBFT
v0.34.27 by replacing your dependency in your `go.mod` file:

```bash
go mod edit -replace github.com/tendermint/tendermint=github.com/cometbft/cometbft@v0.34.27
```

We make use of the original module URL in order to minimize the impact of
switching to CometBFT. This is only possible in our v0.34 release series, and we
will be switching our module URL to `github.com/cometbft/cometbft` in the next
major release.

### Home directory

CometBFT, by default, will consider its home directory in `~/.cometbft` from now
on instead of `~/.tendermint`.

### Environment variables

The environment variable prefixes have now changed from `TM` to `CMT`. For
example, `TMHOME` becomes `CMTHOME`.

We have implemented a fallback check in case `TMHOME` is still set and `CMTHOME`
is not, but you will start to see a warning message in the logs if the old
`TMHOME` variable is set. This fallback check will be removed entirely in a
subsequent major release of CometBFT.

### Building CometBFT

CometBFT must be compiled using Go 1.19 or higher. The use of Go 1.18 is not
supported, since this version has reached end-of-life with the release of [Go 1.20][go120].

### Troubleshooting

If you run into any trouble with this upgrade, please [contact us][discussions].

---

For historical upgrading instructions for Tendermint Core v0.34.24 and earlier,
please see the [Tendermint Core upgrading instructions][tmupgrade].

[v03424]: https://github.com/tendermint/tendermint/releases/tag/v0.34.24
[v03425]: https://github.com/informalsystems/tendermint/releases/tag/v0.34.25
[v03426]: https://github.com/informalsystems/tendermint/releases/tag/v0.34.26
[discussions]: https://github.com/cometbft/cometbft/discussions
[tmupgrade]: https://github.com/tendermint/tendermint/blob/35581cf54ec436b8c37fabb43fdaa3f48339a170/UPGRADING.md
[go120]: https://go.dev/blog/go1.20
[go122]: https://go.dev/blog/go1.22
[pbts-spec]: ./spec/consensus/proposer-based-timestamp/README.md
