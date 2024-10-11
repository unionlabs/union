# CHANGELOG

## Unreleased

*July 1, 2024*

This is a major release of CometBFT that includes several substantial changes
that aim to reduce bandwidth consumption, enable modularity, improve
integrators' experience and increase the velocity of the CometBFT development
team, including:

1. Validators now proactively communicate the block parts they already have so
   others do not resend them, reducing amplification in the network and reducing
   bandwidth consumption.
2. An experimental feature in the mempool that allows limiting the number of
   peers to which transactions are forwarded, allowing operators to optimize
   gossip-related bandwidth consumption further.
3. An opt-in `nop` mempool, which allows application developers to turn off all
   mempool-related functionality in Comet such that they can build their own
   transaction dissemination mechanism, for example a standalone mempool-like
   process that can be scaled independently of the consensus engine/application.
   This requires application developers to implement their own gossip/networking
   mechanisms. See [ADR 111](./docs/architecture/adr-111-nop-mempool.md) for
   details.
4. The first officially supported release of the [data companion
   API](./docs/architecture/adr-101-data-companion-pull-api.md).
5. Versioning of both the Protobuf definitions _and_ RPC. By versioning our
   APIs, we aim to provide a level of commitment to API stability while
   simultaneously affording ourselves the ability to roll out substantial
   changes in non-breaking releases of CometBFT. See [ADR 103](./docs/architecture/adr-103-proto-versioning.md) and [ADR 107](./docs/architecture/adr-107-betaize-proto-versions.md).
6. Moving many Go packages that are currently publicly accessible into the
   `internal` directory such that the team can roll out substantial changes in
   future without needing to worry about causing breakages in users' codebases.
   The massive surface area of previous versions has in the past significantly
   hampered the team's ability to roll out impactful new changes to users, as
   previously such changes required a new breaking release (which currently
   takes 6 to 12 months to reach production use for many users). See [ADR 109](./docs/architecture/adr-109-reduce-go-api-surface.md) for more details.
7. Proposer-Based Timestamps (PBTS) support. PBTS is a Byzantine fault-tolerant
    algorithm used by CometBFT for computing block times.
    When activated on a chain, it replaces the pre-existing BFT-time algorithm.
    See [spec](./spec/consensus/proposer-based-timestamp) doc for PBTS.

None of these changes are state machine-breaking for CometBFT-based networks,
but could be breaking for some users who depend on the Protobuf definitions type
URLs. See the [upgrading guidelines](./UPGRADING.md) and specific changes below
for more details.

**NB: This version is still a release candidate, which means that
API-breaking changes, although very unlikely, might still be introduced
before the final release.** See [RELEASES.md](./RELEASES.md) for more information on
the stability guarantees we provide for pre-releases.

### BREAKING CHANGES

 - `[abci/types]` Rename `UpdateValidator` to `NewValidatorUpdate`, remove
   `Ed25519ValidatorUpdate` ([\#2843](https://github.com/cometbft/cometbft/pull/2843))
- [`config`] deprecate boltdb and cleveldb. If you're using either of those,
  please reach out ([\#2775](https://github.com/cometbft/cometbft/pull/2775))
- `[abci/client]` Deprecate `SetResponseCallback(cb Callback)` in the `Client` interface as it is no
longer used. ([\#3084](https://github.com/cometbft/cometbft/issues/3084))
- `[abci/types]` Replace `ValidatorUpdate.PubKey` with `PubKeyType` and
  `PubKeyBytes` to allow applications to avoid implementing `PubKey` interface.
  ([\#2843](https://github.com/cometbft/cometbft/pull/2843))
- `[abci]` Changed the proto-derived enum type and constant aliases to the
  buf-recommended naming conventions adopted in the `abci/v1` proto package.
  For example, `ResponseProcessProposal_ACCEPT` is renamed to `PROCESS_PROPOSAL_STATUS_ACCEPT`
  ([\#736](https://github.com/cometbft/cometbft/issues/736)).
- `[abci]` The `Type` enum field is now required to be set to a value other
  than the default `CHECK_TX_TYPE_UNKNOWN` for a valid `CheckTxRequest`
  ([\#736](https://github.com/cometbft/cometbft/issues/736)).
- `[abci]` Deprecates `ABCIParams` field of `ConsensusParam` and
  introduces replacement in `FeatureParams` to enable Vote Extensions.
  ([\#2322](https://github.com/cometbft/cometbft/pull/2322))
- `[abci]` Renamed the alias types for gRPC requests, responses, and service
  instances to follow the naming changes in the proto-derived
  `api/cometbft/abci/v1` package
  ([\#1533](https://github.com/cometbft/cometbft/pull/1533)):
  * The prefixed naming pattern `RequestFoo`, `ReponseFoo` changed to
    suffixed `FooRequest`, `FooResponse`.
  * Each method gets its own unique request and response type to allow for
    independent evolution with backward compatibility.
  * `ABCIClient` renamed to `ABCIServiceClient`.
  * `ABCIServer` renamed to `ABCIServiceServer`.
- `[blocksync]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[cmd]` Remove `replay` and `replay-console` subcommands
  and corresponding consensus file replay code, such as
  `consensus.RunReplayFile`, and `consensus.State.ReplayFile`
  ([\#1170](https://github.com/cometbft/cometbft/pull/1170))
- `[comet]` Version variables, in `version/version.go`, have been renamed to reflect the CometBFT rebranding.
   ([\#1621](https://github.com/cometbft/cometbft/pull/1621))
- `[config]` Merge `timeout_prevote` and `timeout_precommit`,
  `timeout_prevote_delta` and `timeout_precommit_delta` into `timeout_round`
  and `timeout_round_delta` accordingly
  ([\#2895](https://github.com/cometbft/cometbft/pull/2895))
- `[config]` Remove `cleveldb` and `boltdb` ([\#2786](https://github.com/cometbft/cometbft/pull/2786))
- `[config]` Remove `skip_timeout_commit` in favor of `timeout_commit=0`
  ([\#2892](https://github.com/cometbft/cometbft/pull/2892))
- `[consensus]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[consensus]` `Handshaker.Handshake` now requires `context.Context` ([\#857](https://github.com/cometbft/cometbft/pull/857))
- `[node]` `NewNode` now requires `context.Context` as the first parameter ([\#857](https://github.com/cometbft/cometbft/pull/857))
- `[crypto/merkle]` The public `Proof.ComputeRootHash` function has been deleted.
   ([\#558](https://github.com/cometbft/cometbft/issues/558))
- `[crypto]` Remove unnecessary `Sha256` wrapper
  ([\#3248](https://github.com/cometbft/cometbft/pull/3248))
- `[crypto]` Remove unnecessary `xchacha20` and `xsalsa20` implementations
  ([\#3347](https://github.com/cometbft/cometbft/pull/3347))
- `[evidence]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[go/runtime]` Bump minimum Go version to v1.22
  ([\#2725](https://github.com/cometbft/cometbft/pull/2725))
- `[inspect]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[internal/state]` Moved function `MedianTime` to package `types`,
  and made it a method of `Commit` so it can be used by external packages.
  ([\#2397](https://github.com/cometbft/cometbft/pull/2397))
- `[libs/async]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/autofile]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/bits]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/clist]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/cmap]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/events]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/fail]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/flowrate]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/net]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/os]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/progressbar]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/rand]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/strings]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/tempfile]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[libs/timer]` Move to `internal`
  ([\#1485](https://github.com/cometbft/cometbft/pull/1485))
- `[mempool]` Add to the `Mempool` interface a new method `PreUpdate()`. This method should be
  called before acquiring the mempool lock, to signal that a new update is coming. Also add to
  `ErrMempoolIsFull` a new field `RecheckFull`.
  ([\#3314](https://github.com/cometbft/cometbft/pull/3314))
- `[mempool]` Change the signature of `CheckTx` in the `Mempool` interface to
`CheckTx(tx types.Tx, sender p2p.ID) (*abcicli.ReqRes, error)`.
([\#1010](https://github.com/cometbft/cometbft/issues/1010), [\#3084](https://github.com/cometbft/cometbft/issues/3084))
- `[mempool]` Remove `mempoolIDs` for internally storing peer ids as `p2p.ID`
  instead of `uint16`.
  ([\#1146](https://github.com/cometbft/cometbft/pull/1146))
- `[node]` Change the signature of `GenesisDocProvider` to
  return the checksum of JSON content alongside the parsed genesis data
  ([\#1287](https://github.com/cometbft/cometbft/issues/1287)).
- `[node]` Go-API breaking: Change the signature of `LoadStateFromDBOrGenesisDocProvider`
   to accept an optional operator provided hash of the genesis file
  ([\#1324](https://github.com/cometbft/cometbft/pull/1324)).
- `[p2p]` Rename `IPeerSet#List` to `Copy`, add `Random`, `ForEach` methods.
   Rename `PeerSet#List` to `Copy`, add `Random`, `ForEach` methods.
   ([\#2246](https://github.com/cometbft/cometbft/pull/2246))
- `[privval]` allow privval to sign arbitrary bytes
  ([\#2692](https://github.com/cometbft/cometbft/pull/2692))
- `[proto/api]` Made `/api` a standalone Go module with its own `go.mod`
  ([\#1561](https://github.com/cometbft/cometbft/issues/1561))
- `[proto/privval]`  Replace `pub_key` with `pub_key_type` and `pub_key_bytes` in
  `PubKeyResponse` ([\#2878](https://github.com/cometbft/cometbft/issues/2878))
- `[proto/types]` Deprecate `pub_key` in favor of `pub_key_type` and `pub_key_bytes` in
  `Validator` ([\#2878](https://github.com/cometbft/cometbft/issues/2878))
- `[proto]` Remove `abci.ValidatorUpdate.pub_key`, add `pub_key_type` and
  `pub_key_bytes` ([\#2843](https://github.com/cometbft/cometbft/pull/2843))
- `[proto]` Remove stateful block data retrieval methods from the
  data companion gRPC API as per
  [RFC 106](https://github.com/cometbft/cometbft/blob/main/docs/references/rfc/rfc-106-separate-stateful-methods.md)
  ([\#2230](https://github.com/cometbft/cometbft/issues/2230)):
  * `GetLatest` from `cometbft.services.block.v1.BlockService`;
  * `GetLatestBlockResults` from `cometbft.services.block_results.v1.BlockResultsService`.
- `[rpc/grpc]` Remove support for stateful block data retrieval methods from the
  data companion APIs as per [RFC 106](https://github.com/cometbft/cometbft/blob/main/docs/references/rfc/rfc-106-separate-stateful-methods.md)
  * `GetLatestBlock` method removed from the `BlockServiceClient` interface.
  * `GetLatestBlockResults` method removed from the `BlockResultServiceClient` interface.
  * `GetLatest` endpoint is no longer served by `BlockServiceServer` instances.
  * `GetLatestBlockResults` endpoint is no longer served by `BlockResultServiceServer` instances.
- `[proto]` Renamed the packages from `tendermint.*` to `cometbft.*`
  and introduced versioned packages to distinguish between proto definitions
  released in `0.34.x`, `0.37.x`, `0.38.x`, and `1.x` versions.
  Prior to the 1.0 release, the versioned packages are suffixed with
  `.v1beta1`, `.v1beta2`, and so on; all definitions describing the protocols
  as per the 1.0.0 release are in packages suffixed with `.v1`.
  Relocated generated Go code into a new `api` folder and changed the import
  paths accordingly.
  ([\#495](https://github.com/cometbft/cometbft/pull/495),
  [\#1504](https://github.com/cometbft/cometbft/issues/1504))
- `[proto]` The names in the `cometbft.abci.v1` versioned proto package
  are changed to satisfy the
  [buf guidelines](https://buf.build/docs/best-practices/style-guide/)
  ([#736](https://github.com/cometbft/cometbft/issues/736),
   [#1504](https://github.com/cometbft/cometbft/issues/1504),
   [#1530](https://github.com/cometbft/cometbft/issues/1530)):
  * Names of request and response types used in gRPC changed by making
    `Request`/`Response` the suffix instead of the prefix, e.g.
    `RequestCheckTx` ⭢ `CheckTxRequest`.
  * The `Request` and `Response` multiplex messages are redefined accordingly.
  * `CheckTxType` values renamed with the `CHECK_TX_TYPE_` prefix.
  * `MisbehaviorType` values renamed with the `MISBEHAVIOR_TYPE_` prefix.
  * `Result` enum formerly nested in `ResponseOfferSnapshot` replaced with the package-level
    `OfferSnapshotResult`, its values named with the
    `OFFER_SNAPSHOT_RESULT_` prefix.
  * `Result` enum formerly nested in `ResponseApplyShapshotChunk` replaced with the package-level
    `ApplySnapshotChunkResult`, its values named with the
    `APPLY_SNAPSHOT_CHUNK_RESULT_` prefix.
  * `Status` enum formerly nested in `ResponseProcessProposal` replaced with the package-level
    `ProcessProposalStatus`, its values named with the
    `PROCESS_PROPOSAL_STATUS_` prefix.
  * `Status` enum formerly nested in `ResponseVerifyVoteExtension` replaced with the package-level
    `VerifyVoteExtensionStatus`, its values named with the
    `VERIFY_VOTE_EXTENSION_STATUS_` prefix.
  * New definition of `Misbehavior` using the changed `MisbehaviorType`.
  * The gRPC service is renamed `ABCIService` and defined using the types listed above.
- `[proto]` In the `cometbft.state.v1` package, the definition for `ABCIResponsesInfo`
  is changed, renaming `response_finalize_block` field to `finalize_block`.
- `[proxy]` Expand `ClientCreator` interface to allow
  for per-"connection" control of client creation
  ([\#1141](https://github.com/cometbft/cometbft/pull/1141))
- `[rpc/client]` Hard-code the `/websocket` endpoint path such that it is
  no longer configurable, removing the related client constructor parameter
  ([\#1412](https://github.com/cometbft/cometbft/pull/1412))
- `[rpc/grpc]` Remove the deprecated gRPC broadcast API
  ([\#650](https://github.com/cometbft/cometbft/issues/650))
- `[rpc]` The endpoints `broadcast_tx_*` now return an error when the node is
  performing block sync or state sync.
  ([\#785](https://github.com/cometbft/cometbft/issues/785))
- `[mempool]` When the node is performing block sync or state sync, the mempool
  reactor now discards incoming transactions from peers, and does not propagate
  transactions to peers.
  ([\#785](https://github.com/cometbft/cometbft/issues/785))
- `[state/indexer/block]` BlockIndexer now has additional method `Prune`, `GetRetainHeight`, `SetRetainHeight` ([\#1176](https://github.com/cometbft/cometbft/pull/1176))
- `[state/txindex]` TxIndexer now has additional methods: `Prune`, `GetRetainHeight`, `SetRetainHeight` ([\#1176](https://github.com/cometbft/cometbft/pull/1176))
- `[state/store]` go-API breaking change in `PruneABCIResponses`: added parameter to force compaction. ([\#1972](https://github.com/cometbft/cometbft/pull/1972))
- `[state/store]` go-API breaking change in `PruneStates`: added parameter to pass the number of pruned states and return pruned entries in current pruning iteration. ([\#1972](https://github.com/cometbft/cometbft/pull/1972))
- `[state]` The `state.Store` interface has been expanded
  to accommodate the data pull companion API of ADR 101
  ([\#1096](https://github.com/cometbft/cometbft/issues/1096))
- `[store]` Make the `LoadBlock` method also return block metadata
  ([\#1556](https://github.com/cometbft/cometbft/issues/1556))
- `[version]` Bumped the P2P version from 8 to 9, as this release contains new P2P messages.
   ([\#1411](https://github.com/cometbft/cometbft/pull/1411))

### BUG FIXES

- `[bits]` prevent `BitArray.UnmarshalJSON` from crashing on 0 bits
  ([\#2774](https://github.com/cometbft/cometbft/pull/2774))
- `[blocksync]` Wait for `poolRoutine` to stop in `(*Reactor).OnStop`
  ([\#1879](https://github.com/cometbft/cometbft/pull/1879))
- `[consensus]` Consensus now prevotes `nil` when the proposed value does not
  match the value the local validator has locked on
  ([\#1203](https://github.com/cometbft/cometbft/pull/1203))
- `[consensus]` Fix a race condition in the consensus timeout ticker. Race is caused by two timeouts being scheduled at the same time.
  ([\#3092](https://github.com/cometbft/cometbft/pull/2136))
- `[consensus]` Fix for Security Advisory `ASA-2024-001`: Validation of `VoteExtensionsEnableHeight` can cause chain halt
  ([ASA-2024-001](https://github.com/cometbft/cometbft/security/advisories/GHSA-qr8r-m495-7hc4))
- `[consensus]` Remove logic to unlock block on +2/3 prevote for nil
  ([\#1175](https://github.com/cometbft/cometbft/pull/1175): @BrendanChou)
- `[crypto]` `SupportsBatchVerifier` returns false
  if public key is nil instead of dereferencing nil.
  ([\#1825](https://github.com/cometbft/cometbft/pull/1825))
- `[evidence]` When `VerifyCommitLight` & `VerifyCommitLightTrusting` are called as part
  of evidence verification, all signatures present in the evidence must be verified
  ([\#1749](https://github.com/cometbft/cometbft/pull/1749))
- `[mempool]` Fix data race when rechecking with async ABCI client
  ([\#1827](https://github.com/cometbft/cometbft/issues/1827))
- `[mempool]` Fix data races in `CListMempool` by making atomic the types of `height`, `txsBytes`, and
  `notifiedTxsAvailable`. ([\#642](https://github.com/cometbft/cometbft/pull/642))
- `[mempool]` Panic when a CheckTx request to the app returns an error
  ([\#2225](https://github.com/cometbft/cometbft/pull/2225))
- `[mempool]` The calculation method of tx size returned by calling proxyapp should be consistent with that of mempool
  ([\#1687](https://github.com/cometbft/cometbft/pull/1687))
- `[p2p/pex]` Gracefully shutdown Reactor ([\#2010](https://github.com/cometbft/cometbft/pull/2010))
- `[privval]` Retry accepting a connection ([\#2047](https://github.com/cometbft/cometbft/pull/2047))
- `[rpc]` Fix nil pointer error in `/tx` and `/tx_search` when block is
  absent ([\#3352](https://github.com/cometbft/cometbft/issues/3352))
- `[state/indexer]` Respect both height params while querying for events
   ([\#1529](https://github.com/cometbft/cometbft/pull/1529))
- `[state/pruning]` When no blocks are pruned, do not attempt to prune statestore
   ([\#1616](https://github.com/cometbft/cometbft/pull/1616))
- `[state]` Fix rollback to a specific height
  ([\#2136](https://github.com/cometbft/cometbft/pull/2136))
- `[types]` Do not batch verify a commit if the validator set keys have different
  types. ([\#3195](https://github.com/cometbft/cometbft/issues/3195)

### DEPENDENCIES

- Bump api to v1.0.0-rc.1 for v1.0.0 Release Candidate 1
  ([\#3191](https://github.com/cometbft/cometbft/pull/3191))
- Bump cometbft-db to v0.9.0, providing support for RocksDB v8
  ([\#1725](https://github.com/cometbft/cometbft/pull/1725))

### FEATURES

- `[config]` Add [`pebbledb`](https://github.com/cockroachdb/pebble). To use, build with
  `pebbledb` tag (`go build -tags pebbledb`) ([\#2132](https://github.com/cometbft/cometbft/pull/2132/))
- `[config]` Add `[grpc.block_results_service]` gRPC configuration `BlockResultsService`
  ([\#1095](https://github.com/cometbft/cometbft/issues/1095))
- `[config]` Add `[grpc.block_service]` section to configure gRPC `BlockService`
  ([\#1094](https://github.com/cometbft/cometbft/issues/1094))
- `[config]` Add configuration parameters to tweak forced compaction. ([\#1972](https://github.com/cometbft/cometbft/pull/1972))
- `[config]` Added `[grpc.version_service]` section for configuring the gRPC version service.
  ([\#816](https://github.com/cometbft/cometbft/issues/816))
- `[config]` Added `[grpc]` section to configure the gRPC server.
  ([\#816](https://github.com/cometbft/cometbft/issues/816))
- `[config]` Added `[storage.experimental_db_key_layout]` storage parameter, set to "v2"
  for order preserving representation
([\#2327](https://github.com/cometbft/cometbft/pull/2327/))
- `[config]` Removed unused `[mempool.max_batch_bytes]` mempool parameter.
 ([\#2056](https://github.com/cometbft/cometbft/pull/2056/))
- `[config]` Update the default value of `mempool.max_txs_bytes` to 64 MiB.
  ([\#2756](https://github.com/cometbft/cometbft/issues/2756))
- `[consensus]` Update block validation to no longer require the block timestamp
  to be the median of the timestamps of the previous commit. (@anca)
  ([tendermint/tendermint\#7382](https://github.com/tendermint/tendermint/pull/7382))
- `[consensus]` Update proposal validation logic to Prevote nil
  if a proposal does not meet the conditions for Timeliness
  per the proposer-based timestamp specification. (@anca)
  ([tendermint/tendermint\#7415](https://github.com/tendermint/tendermint/pull/7415))
- `[consensus]` Update the proposal logic per the Propose-based timestamps specification
  so that the proposer will wait for the previous block time to occur
  before proposing the next block. (@williambanfield)
  ([tendermint/tendermint\#7376](https://github.com/tendermint/tendermint/pull/7376))
- `[consensus]` Use the proposed block timestamp as the proposal timestamp.
  Update the block validation logic to ensure that the proposed block's timestamp
  matches the timestamp in the proposal message. (@williambanfield)
  ([tendermint/tendermint\#7391](https://github.com/tendermint/tendermint/pull/7391))
- `[consensus]` Use the proposer timestamp for the first height instead of the genesis time.
  Chains will still start consensus at the genesis time. (@anca)
  ([tendermint/tendermint\#7711](https://github.com/tendermint/tendermint/pull/7711))
- `[consensus]` add a new `synchrony` field to the `ConsensusParameter` struct
  for controlling the parameters of the proposer-based timestamp algorithm. (@williambanfield)
  ([tendermint/tendermint\#7354](https://github.com/tendermint/tendermint/pull/7354))
- `[crypto]` Add support for BLS12-381 keys. Since the implementation needs
  `cgo` and brings in new dependencies, we use the `bls12381` build flag to
  enable it ([\#2765](https://github.com/cometbft/cometbft/pull/2765))
- `[docs]` Add report on storage improvements and findings. ([\#2569](https://github.com/cometbft/cometbft/pull/2569))
- `[e2e]` Add `block_max_bytes` option to the manifest file.
  ([\#2362](https://github.com/cometbft/cometbft/pull/2362))
- `[e2e]` Add new `--testnet-dir` parameter to set a custom directory for the generated testnet files.
  ([\#2433](https://github.com/cometbft/cometbft/pull/2433))
- `[evidence/store]` Added support for a different DB key representation within the evidence store ([\#2327](https://github.com/cometbft/cometbft/pull/2327/))
- `[grpc]` Add `BlockResultsService` with client to fetch BlockResults
  for a given height, or latest.
  ([\#1095](https://github.com/cometbft/cometbft/issues/1095))
- `[grpc]` Add `BlockService` with client to facilitate fetching of blocks and
  streaming of the latest committed block height
  ([\#1094](https://github.com/cometbft/cometbft/issues/1094))
- `[light/store]` Added support for a different DB key representation within the light block store ([\#2327](https://github.com/cometbft/cometbft/pull/2327/))
- `[mempool]` Add `nop` mempool ([\#1643](https://github.com/cometbft/cometbft/pull/1643)). If you want to use it, change mempool's `type` to `nop`:
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
- `[metrics]` Add metric for mempool size in bytes `SizeBytes`.
  ([\#1512](https://github.com/cometbft/cometbft/pull/1512))
- `[metrics]` Add metrics to monitor pruning and current available data in stores: `PruningServiceBlockRetainHeight`, `PruningServiceBlockResultsRetainHeight`, `ApplicationBlockRetainHeight`, `BlockStoreBaseHeight`, `ABCIResultsBaseHeight`.
  ([\#1234](https://github.com/cometbft/cometbft/pull/1234))
- `[metrics]` Added metrics to monitor block store access. ([\#1974](https://github.com/cometbft/cometbft/pull/1974))
- `[metrics]` Added metrics to monitor state store access. ([\#1974](https://github.com/cometbft/cometbft/pull/1974))
- `[proto]` Add definitions and generated code for
  [ADR-101](./docs/architecture/adr-101-data-companion-pull-api.md)
  `PruningService` in the `cometbft.services.pruning.v1` proto package
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
- `[rpc/grpc]` Add privileged gRPC server and client facilities, in
  `server/privileged` and `client/privileged` packages respectively, to
  enable a separate API server within the node which serves trusted clients
  without authentication and should never be exposed to public internet
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
- `[rpc/grpc]` Add a pruning service adding on the privileged gRPC server API to
  give an [ADR-101](./docs/architecture/adr-101-data-companion-pull-api.md) data
  companion control over block data retained by the node. The
  `WithPruningService` option method in `server/privileged` is provided to
  configure the pruning service
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
- `[rpc/grpc]` Add `PruningServiceClient` interface
  for the gRPC client in `client/privileged` along with a configuration option
  to enable it
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
- `[config]` Add `[grpc.privileged]` section to configure the privileged
  gRPC server for the node, and `[grpc.privileged.pruning_service]` section
  to control the pruning service
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
- `[proto]` add `syncing_to_height` to `FinalizeBlockRequest` to let the ABCI app
  know if the node is syncing or not.
  ([\#1247](https://github.com/cometbft/cometbft/issues/1247))
- `[rpc/grpc]` Add gRPC client with support for version service
  ([\#816](https://github.com/cometbft/cometbft/issues/816))
- `[rpc/grpc]` Add gRPC endpoint for pruning the block and transaction indexes
([\#1327](https://github.com/cometbft/cometbft/pull/1327))
- `[rpc/grpc]` Add gRPC server to the node, configurable
  via a new `[grpc]` section in the configuration file
  ([\#816](https://github.com/cometbft/cometbft/issues/816))
- `[rpc/grpc]` Add gRPC version service to allow clients to
  establish the software and protocol versions of the node
  ([\#816](https://github.com/cometbft/cometbft/issues/816))
- `[state]` Add TxIndexer and BlockIndexer pruning metrics
  ([\#1334](https://github.com/cometbft/cometbft/issues/1334))
- `[store]` Added support for a different DB key representation to state and block store ([\#2327](https://github.com/cometbft/cometbft/pull/2327/))
- `[store]` When pruning force compaction of the database. ([\#1972](https://github.com/cometbft/cometbft/pull/1972))
- `[test]` Added monitoring tools and dashboards for local testing with `localnet`. ([\#2107](https://github.com/cometbft/cometbft/issues/2107))

### IMPROVEMENTS

- `[abci/client]` Add consensus-synchronized local client creator,
  which only imposes a mutex on the consensus "connection", leaving
  the concurrency of all other "connections" up to the application
  ([\#1141](https://github.com/cometbft/cometbft/pull/1141))
- `[abci/client]` Add fully unsynchronized local client creator, which
  imposes no mutexes on the application, leaving all handling of concurrency up
  to the application ([\#1141](https://github.com/cometbft/cometbft/pull/1141))
- `[abci]` Increase ABCI socket message size limit to 2GB ([\#1730](https://github.com/cometbft/cometbft/pull/1730): @troykessler)
- `[blockstore]` Remove a redundant `Header.ValidateBasic` call in `LoadBlockMeta`, 75% reducing this time.
  ([\#2964](https://github.com/cometbft/cometbft/pull/2964))
- `[blockstore]` Use LRU caches for LoadBlockPart. Make the LoadBlockPart and LoadBlockCommit APIs 
    return mutative copies, that the caller is expected to not modify. This saves on memory copying.
  ([\#3342](https://github.com/cometbft/cometbft/issues/3342))
- `[blockstore]` Use LRU caches in blockstore, significiantly improving consensus gossip routine performance
  ([\#3003](https://github.com/cometbft/cometbft/issues/3003))
- `[blocksync]` Avoid double-calling `types.BlockFromProto` for performance
  reasons ([\#2016](https://github.com/cometbft/cometbft/pull/2016))
- `[blocksync]` Request a block from peer B if we are approaching pool's height
  (less than 50 blocks) and the current peer A is slow in sending us the
  block ([\#2475](https://github.com/cometbft/cometbft/pull/2475))
- `[blocksync]` Request the block N from peer B immediately after getting
  `NoBlockResponse` from peer A
  ([\#2475](https://github.com/cometbft/cometbft/pull/2475))
- `[blocksync]` Sort peers by download rate (the fastest peer is picked first)
  ([\#2475](https://github.com/cometbft/cometbft/pull/2475))
- `[blocksync]` make the max number of downloaded blocks dynamic.
  Previously it was a const 600. Now it's `peersCount * maxPendingRequestsPerPeer (20)`
  ([\#2467](https://github.com/cometbft/cometbft/pull/2467))
- `[cli/node]` The genesis hash provided with the `--genesis-hash` is now
   forwarded to the node, instead of reading the file.
  ([\#1324](https://github.com/cometbft/cometbft/pull/1324))
- `[cmd]` Add support for all key types in `gen-validator` command. Use
 `--key-type=` (or `-k`) to specify the key type (e.g., `-k secp256k1`).
  ([\#1757](https://github.com/cometbft/cometbft/issues/1757))
- `[config]` Added `[storage.pruning]` and `[storage.pruning.data_companion]`
  sections to facilitate background pruning and data companion (ADR 101)
  operations ([\#1096](https://github.com/cometbft/cometbft/issues/1096))
- `[config]` Added `genesis_hash` storage parameter, which when set it is checked
 on node startup
 ([\#1324](https://github.com/cometbft/cometbft/pull/1324/))
- `[config]` Added `recheck_timeout` mempool parameter to set how much time to wait for recheck
 responses from the app (only applies to non-local ABCI clients).
 ([\#1827](https://github.com/cometbft/cometbft/issues/1827/))
- `[config]` Use embed pkg for the default template
  ([\#3057](https://github.com/cometbft/cometbft/pull/3057))
- `[consensus/state]` Remove a redundant `VerifyBlock` call in `FinalizeCommit`
  ([\#2928](https://github.com/cometbft/cometbft/pull/2928))
- `[consensus]` Add `chain_size_bytes` metric for measuring the size of the blockchain in bytes
  ([\#2093](https://github.com/cometbft/cometbft/pull/2093))
- `[consensus]` Fix some reactor messages taking write locks instead of read locks.
  ([\#3159](https://github.com/cometbft/cometbft/issues/3159))
- `[consensus]` Improve performance of consensus metrics by lowering string operations
  ([\#3017](https://github.com/cometbft/cometbft/issues/3017))
- `[consensus]` Log vote validation failures at info level
  ([\#1022](https://github.com/cometbft/cometbft/pull/1022))
- `[consensus]` Lower the consensus blocking overhead of broadcasts from `num_peers * process_creation_time` to `process_creation_time`.
  ([\#3180](https://github.com/cometbft/cometbft/issues/3180))
- `[consensus]` Make Vote messages only take one peerstate mutex
  ([\#3156](https://github.com/cometbft/cometbft/issues/3156))
- `[consensus]` New metrics (counters) to track duplicate votes and block parts.
  ([\#896](https://github.com/cometbft/cometbft/pull/896))
- `[consensus]` Optimize vote and block part gossip with new message `HasProposalBlockPartMessage`,
  which is similar to `HasVoteMessage`; and random sleep in the loop broadcasting those messages.
  The sleep can be configured with new config `peer_gossip_intraloop_sleep_duration`, which is set to 0
  by default as this is experimental.
  Our scale tests show substantial bandwidth improvement with a value of 50 ms.
  ([\#904](https://github.com/cometbft/cometbft/pull/904))
- `[consensus]` Reduce the default MaxBytes to 4MB and increase MaxGas to 10 million
  ([\#1518](https://github.com/cometbft/cometbft/pull/1518))
- `[consensus]` Reuse an internal buffer for block building to reduce memory allocation overhead.
  ([\#3162](https://github.com/cometbft/cometbft/issues/3162))
- `[consensus]` Use an independent rng for gossip threads, reducing mutex contention.
  ([\#3005](https://github.com/cometbft/cometbft/issues/3005))
- `[consensus]` When prevoting, avoid calling PropocessProposal when we know the
  proposal was already validated by correct nodes.
  ([\#1230](https://github.com/cometbft/cometbft/pull/1230))
- `[crypto/merkle]` faster calculation of hashes ([#1921](https://github.com/cometbft/cometbft/pull/1921))
- `[docs]` Merge configuration doc in explanation section with the config.toml document in references.
  ([\#2769](https://github.com/cometbft/cometbft/pull/2769))
- `[e2e]` Add manifest option `VoteExtensionsUpdateHeight` to test
  vote extension activation via `InitChain` and `FinalizeBlock`.
  Also, extend the manifest generator to produce different values
  of this new option
  ([\#2065](https://github.com/cometbft/cometbft/pull/2065))
- `[e2e]` Add manifest option `load_max_txs` to limit the number of transactions generated by the
  `load` command. ([\#2094](https://github.com/cometbft/cometbft/pull/2094))
- `[e2e]` Add new targets `fast` and `clean` to Makefile.
  ([\#2192](https://github.com/cometbft/cometbft/pull/2192))
- `[e2e]` Allow disabling the PEX reactor on all nodes in the testnet
  ([\#1579](https://github.com/cometbft/cometbft/pull/1579))
- `[e2e]` Allow latency emulation between nodes.
  ([\#1560](https://github.com/cometbft/cometbft/pull/1560))
- `[e2e]` Introduce the possibility in the manifest for some nodes
  to run in a preconfigured clock skew.
  ([\#2453](https://github.com/cometbft/cometbft/pull/2453))
- `[e2e]` Log the number of transactions that were sent successfully or failed.
  ([\#2328](https://github.com/cometbft/cometbft/pull/2328))
- `[e2e]` add option to the 'runner logs' command to output logs separately.
  ([\#3353](https://github.com/cometbft/cometbft/pull/3353))
- `[event-bus]` Remove the debug logs in PublishEventTx, which were noticed production slowdowns.
  ([\#2911](https://github.com/cometbft/cometbft/pull/2911))
- `[flowrate]` Remove expensive time.Now() calls from flowrate calls.
  Changes clock updates to happen in a separate goroutine.
  ([\#3016](https://github.com/cometbft/cometbft/issues/3016))
- `[indexer]` Optimized the PSQL indexer
  ([\#2142](https://github.com/cometbft/cometbft/pull/2142)) thanks to external contributor @k0marov !
- `[internal/bits]` 10x speedup and remove heap overhead of `bitArray.PickRandom` (used extensively in consensus gossip)
  ([\#2841](https://github.com/cometbft/cometbft/pull/2841)).
- `[internal/bits]` 10x speedup creating initialized bitArrays, which speedsup extendedCommit.BitArray(). This is used in consensus vote gossip.
  ([\#2959](https://github.com/cometbft/cometbft/pull/2841)).
- `[jsonrpc]` enable HTTP basic auth in websocket client ([#2434](https://github.com/cometbft/cometbft/pull/2434))
- `[libs/json]` Lower the memory overhead of JSON encoding by using JSON encoders internally.
  ([\#2846](https://github.com/cometbft/cometbft/pull/2846))
- `[linting]` Removed undesired linting from `Makefile` and added dependency check for `codespell`.
  ([\#1958](https://github.com/cometbft/cometbft/pull/1958/))
- `[mempool]` Add a metric (a counter) to measure whether a tx was received more than once.
  ([\#634](https://github.com/cometbft/cometbft/pull/634))
- `[mempool]` Add experimental feature to limit the number of persistent peers and non-persistent
  peers to which the node gossip transactions.
  ([\#1558](https://github.com/cometbft/cometbft/pull/1558))
  ([\#1584](https://github.com/cometbft/cometbft/pull/1584))
- `[config]` Add mempool parameters `experimental_max_gossip_connections_to_persistent_peers` and
  `experimental_max_gossip_connections_to_non_persistent_peers` for limiting the number of peers to
  which the node gossip transactions.
  ([\#1558](https://github.com/cometbft/cometbft/pull/1558))
  ([\#1584](https://github.com/cometbft/cometbft/pull/1584))
- `[mempool]` Before updating the mempool, consider it as full if rechecking is still in progress.
  This will stop accepting transactions in the mempool if the node can't keep up with re-CheckTx.
  ([\#3314](https://github.com/cometbft/cometbft/pull/3314))
- `[node]` On upgrade, after [\#1296](https://github.com/cometbft/cometbft/pull/1296), delete the genesis file existing in the DB.
  ([\#1297](https://github.com/cometbft/cometbft/pull/1297))
- `[node]` Remove genesis persistence in state db, replaced by a hash
  ([\#1017](https://github.com/cometbft/cometbft/pull/1017),
  [\#1295](https://github.com/cometbft/cometbft/pull/1295))
- `[node]` The `node.Node` struct now manages a
  `state.Pruner` service to facilitate background pruning
  ([\#1096](https://github.com/cometbft/cometbft/issues/1096))
- `[node]` export node package errors
  ([\#3056](https://github.com/cometbft/cometbft/pull/3056))
- `[p2p/channel]` Speedup `ProtoIO` writer creation time, and thereby speedup channel writing by 5%.
  ([\#2949](https://github.com/cometbft/cometbft/pull/2949))
- `[p2p/conn]` Minor speedup (3%) to connection.WritePacketMsgTo, by removing MinInt calls.
  ([\#2952](https://github.com/cometbft/cometbft/pull/2952))
- `[p2p/conn]` Speedup connection.WritePacketMsgTo, by reusing internal buffers rather than re-allocating.
  ([\#2986](https://github.com/cometbft/cometbft/pull/2986))
- `[p2p]` Lower `flush_throttle_timeout` to 10ms
  ([\#2988](https://github.com/cometbft/cometbft/issues/2988))
- `[p2p]` make `PeerSet.Remove` more efficient (Author: @odeke-em) [\#2246](https://github.com/cometbft/cometbft/pull/2246)
- `[privval]` DO NOT require extension signature from privval if vote
  extensions are disabled. Remote signers can skip signing the extension if
  `skip_extension_signing` flag in `SignVoteRequest` is true.
  ([\#2496](https://github.com/cometbft/cometbft/pull/2496))
- `[proto]` Add `skip_extension_signing` field to the `SignVoteRequest` message
  in `cometbft.privval.v1` ([\#2522](https://github.com/cometbft/cometbft/pull/2522)).
  The `cometbft.privval.v1beta2` package is added to capture the protocol as it was
  released in CometBFT 0.38.x
  ([\#2529](https://github.com/cometbft/cometbft/pull/2529)).
- `[protoio]` Remove one allocation and new object call from `ReadMsg`,
  leading to a 4% p2p message reading performance gain.
  ([\#3018](https://github.com/cometbft/cometbft/issues/3018))
- `[rpc]` Add a configurable maximum batch size for RPC requests.
  ([\#2867](https://github.com/cometbft/cometbft/pull/2867)).
- `[rpc]` Export `MakeHTTPDialer` to allow HTTP client constructors more flexibility.
  ([\#1594](https://github.com/cometbft/cometbft/pull/1594))
- `[rpc]` Move the websockets info log for successful replies to debug.
  ([\#2788](https://github.com/cometbft/cometbft/pull/2788))
- `[rpc]` Support setting proxy from env to `DefaultHttpClient`.
  ([\#1900](https://github.com/cometbft/cometbft/pull/1900))
- `[rpc]` The RPC API is now versioned, with all existing endpoints accessible
  via `/v1/*` as well as `/*`
  ([\#1412](https://github.com/cometbft/cometbft/pull/1412))
- `[rpc]` Use default port for HTTP(S) URLs when there is no explicit port ([\#1903](https://github.com/cometbft/cometbft/pull/1903))
- `[spec]` Update Apalache type annotations in the light client spec ([#955](https://github.com/cometbft/cometbft/pull/955))
- `[state/execution]` Cache the block hash computation inside of the Block Type, so we only compute it once.
  ([\#2924](https://github.com/cometbft/cometbft/pull/2924))
- `[state/indexer]` Add transaction and block index pruning
  ([\#1176](https://github.com/cometbft/cometbft/pull/1176))
- `[state/indexer]` Fix txSearch performance issue
  ([\#2855](https://github.com/cometbft/cometbft/pull/2855))
- `[state/indexer]` Lower the heap allocation of transaction searches
  ([\#2839](https://github.com/cometbft/cometbft/pull/2839))
- `[state]` ABCI response pruning has been added for use by the data companion
  ([\#1096](https://github.com/cometbft/cometbft/issues/1096))
- `[state]` Block pruning has been moved from the block executor into a
  background process ([\#1096](https://github.com/cometbft/cometbft/issues/1096))
- `[state]` Save the state using a single DB batch ([\#1735](https://github.com/cometbft/cometbft/pull/1735))
- `[state]` avoid double-saving `FinalizeBlockResponse` for performance reasons
  ([\#2017](https://github.com/cometbft/cometbft/pull/2017))
- `[store]` Save block using a single DB batch if block is less than 640kB, otherwise each block part is saved individually
  ([\#1755](https://github.com/cometbft/cometbft/pull/1755))
- `[types]` Make a new method `GetByAddressMut` for `ValSet`, which does not copy the returned validator.
  ([\#3119](https://github.com/cometbft/cometbft/issues/3119))
- `[types]` Significantly speedup types.MakePartSet and types.AddPart, which are used in creating a block proposal
  ([\#3117](https://github.com/cometbft/cometbft/issues/3117))
- `[types]` Validate `Validator#Address` in `ValidateBasic` ([\#1715](https://github.com/cometbft/cometbft/pull/1715))

## v0.38.0

*September 12, 2023*

This release includes the second part of ABCI++, called ABCI 2.0.
ABCI 2.0 introduces ABCI methods `ExtendVote` and `VerifyVoteExtension`.
These new methods allow the application to add data (opaque to CometBFT),
called _vote extensions_ to precommit votes sent by validators.
These vote extensions are made available to the proposer(s) of the next height.
Additionally, ABCI 2.0 coalesces `BeginBlock`, `DeliverTx`, and `EndBlock`
into one method, `FinalizeBlock`, whose `Request*` and `Response*`
data structures contain the sum of all data previously contained
in the respective `Request*` and `Response*` data structures in
`BeginBlock`, `DeliverTx`, and `EndBlock`.
See the [specification](./spec/abci/) for more details on ABCI 2.0.

### BREAKING CHANGES

- `[abci]` Introduce `FinalizeBlock` which condenses `BeginBlock`, `DeliverTx`
  and `EndBlock` into a single method call
  ([\#9468](https://github.com/tendermint/tendermint/pull/9468))
- `[abci]` Move `app_hash` parameter from `Commit` to `FinalizeBlock`
  ([\#8664](https://github.com/tendermint/tendermint/pull/8664))
- `[config]` Remove `Version` field from `MempoolConfig`.
  ([\#260](https://github.com/cometbft/cometbft/issues/260))
- `[crypto/merkle]` Do not allow verification of Merkle Proofs against empty trees (`nil` root). `Proof.ComputeRootHash` now panics when it encounters an error, but `Proof.Verify` does not panic
  ([\#558](https://github.com/cometbft/cometbft/issues/558))
- `[inspect]` Add a new `inspect` command for introspecting
  the state and block store of a crashed tendermint node.
  ([\#9655](https://github.com/tendermint/tendermint/pull/9655))
- `[kvindexer]` Added support for big integers and big floats in the kvindexer.
  Breaking changes: function `Number` in package `libs/pubsub/query/syntax` changed its return value.
  ([\#797](https://github.com/cometbft/cometbft/pull/797))
- `[mempool]` Application can now set `ConsensusParams.Block.MaxBytes` to -1
  to have visibility on all transactions in the
  mempool at `PrepareProposal` time.
  This means that the total size of transactions sent via `RequestPrepareProposal`
  might exceed `RequestPrepareProposal.max_tx_bytes`.
  If that is the case, the application MUST make sure that the total size of transactions
  returned in `ResponsePrepareProposal.txs` does not exceed `RequestPrepareProposal.max_tx_bytes`,
  otherwise CometBFT will panic.
  ([\#980](https://github.com/cometbft/cometbft/issues/980))
- `[mempool]` Remove priority mempool.
  ([\#260](https://github.com/cometbft/cometbft/issues/260))
- `[metrics]` Move state-syncing and block-syncing metrics to
  their respective packages. Move labels from block_syncing
  -> blocksync_syncing and state_syncing -> statesync_syncing
  ([\#9682](https://github.com/tendermint/tendermint/pull/9682))
- `[node/state]` Add Go API to bootstrap block store and state store to a height. Make sure block sync starts syncing from bootstrapped height.
  ([\#1057](https://github.com/tendermint/tendermint/pull/#1057)) (@yihuang)
- `[state/store]` Added Go functions to save height at which offline state sync is performed.
  ([\#1057](https://github.com/tendermint/tendermint/pull/#1057)) (@jmalicevic)
- `[node]` Move DBContext and DBProvider from the node package to the config
  package. ([\#9655](https://github.com/tendermint/tendermint/pull/9655))
- `[node]` Removed `ConsensusState()` accessor from `Node`
  struct - all access to consensus state should go via the reactor
  ([\#1120](https://github.com/cometbft/cometbft/pull/1120))
- `[p2p]` Remove UPnP functionality
  ([\#1113](https://github.com/cometbft/cometbft/issues/1113))
- `[p2p]` Remove unused p2p/trust package
  ([\#9625](https://github.com/tendermint/tendermint/pull/9625))
- `[protobuf]` Remove fields `sender`, `priority`, and `mempool_error` from
  `ResponseCheckTx`. ([\#260](https://github.com/cometbft/cometbft/issues/260))
- `[pubsub]` Added support for big integers and big floats in the pubsub event query system.
  Breaking changes: function `Number` in package `libs/pubsub/query/syntax` changed its return value.
  ([\#797](https://github.com/cometbft/cometbft/pull/797))
- `[rpc]` Remove global environment and replace with constructor
  ([\#9655](https://github.com/tendermint/tendermint/pull/9655))
- `[rpc]` Removed `begin_block_events` and `end_block_events` from `BlockResultsResponse`.
  The events are merged into one field called `finalize_block_events`.
  ([\#9427](https://github.com/tendermint/tendermint/issues/9427))
- `[state/kvindexer]` Remove the function type from the event key stored in the database. This should be breaking only
for people who forked CometBFT and interact directly with the indexers kvstore.
  ([\#774](https://github.com/cometbft/cometbft/pull/774))
- `[state]` Move pruneBlocks from node/state to state/execution.
  ([\#6541](https://github.com/tendermint/tendermint/pull/6541))
- `[state]` Signature of `ExtendVote` changed in `BlockExecutor`.
  It now includes the block whose precommit will be extended, an the state object.
  ([\#1270](https://github.com/cometbft/cometbft/pull/1270))

### BUG FIXES

- `[abci-cli]` Fix broken abci-cli help command.
  ([\#9717](https://github.com/tendermint/tendermint/pull/9717))
- `[abci]` Restore the snake_case naming in JSON serialization of
  `ExecTxResult` ([\#855](https://github.com/cometbft/cometbft/issues/855)).
- `[consensus]` Avoid recursive call after rename to (*PeerState).MarshalJSON
  ([\#863](https://github.com/cometbft/cometbft/pull/863))
- `[consensus]` Rename `(*PeerState).ToJSON` to `MarshalJSON` to fix a logging data race
  ([\#524](https://github.com/cometbft/cometbft/pull/524))
- `[consensus]` Unexpected error conditions in `ApplyBlock` are non-recoverable, so ignoring the error and carrying on is a bug. We replaced a `return` that disregarded the error by a `panic`.
  ([\#496](https://github.com/cometbft/cometbft/pull/496))
- `[docker]` Ensure Docker image uses consistent version of Go.
  ([\#9462](https://github.com/tendermint/tendermint/pull/9462))
- `[kvindexer]` Forward porting the fixes done to the kvindexer in 0.37 in PR \#77
  ([\#423](https://github.com/cometbft/cometbft/pull/423))
- `[light]` Fixed an edge case where a light client would panic when attempting
  to query a node that (1) has started from a non-zero height and (2) does
  not yet have any data. The light client will now, correctly, not panic
  _and_ keep the node in its list of providers in the same way it would if
  it queried a node starting from height zero that does not yet have data
  ([\#575](https://github.com/cometbft/cometbft/issues/575))
- `[mempool/clist_mempool]` Prevent a transaction to appear twice in the mempool
  ([\#890](https://github.com/cometbft/cometbft/pull/890): @otrack)

### DEPRECATIONS

- `[rpc/grpc]` Mark the gRPC broadcast API as deprecated.
  It will be superseded by a broader API as part of
  [\#81](https://github.com/cometbft/cometbft/issues/81)
  ([\#650](https://github.com/cometbft/cometbft/issues/650))

### FEATURES

- `[abci]` New ABCI methods `VerifyVoteExtension` and `ExtendVote` allow validators to validate the vote extension data attached to a pre-commit message and allow applications to let their validators do more than just validate within consensus ([\#9836](https://github.com/tendermint/tendermint/pull/9836))
- `[node/state]` Add Go API to bootstrap block store and state store to a height
  ([\#1057](https://github.com/tendermint/tendermint/pull/#1057)) (@yihuang)
- `[proxy]` Introduce `NewConnSyncLocalClientCreator`, which allows local ABCI
  clients to have the same concurrency model as remote clients (i.e. one mutex
  per client "connection", for each of the four ABCI "connections").
  ([tendermint/tendermint\#9830](https://github.com/tendermint/tendermint/pull/9830)
  and [\#1145](https://github.com/cometbft/cometbft/pull/1145))
- `[proxy]` Introduce `NewUnsyncLocalClientCreator`, which allows local ABCI
  clients to have the same concurrency model as remote clients (i.e. one
  mutex per client "connection", for each of the four ABCI "connections").
  ([\#9830](https://github.com/tendermint/tendermint/pull/9830))

### IMPROVEMENTS

- `[blocksync]` Generate new metrics during BlockSync
  ([\#543](https://github.com/cometbft/cometbft/pull/543))
- `[crypto/merkle]` Improve HashAlternatives performance
  ([\#6443](https://github.com/tendermint/tendermint/pull/6443))
- `[crypto/merkle]` Improve HashAlternatives performance
  ([\#6513](https://github.com/tendermint/tendermint/pull/6513))
- `[jsonrpc/client]` Improve the error message for client errors stemming from
  bad HTTP responses.
  ([cometbft/cometbft\#638](https://github.com/cometbft/cometbft/pull/638))
- `[mempool]` Application can now set `ConsensusParams.Block.MaxBytes` to -1
  to gain more control on the max size of transactions in a block.
  It also allows the application to have visibility on all transactions in the
  mempool at `PrepareProposal` time.
  ([\#980](https://github.com/cometbft/cometbft/pull/980))
- `[node]` Close evidence.db OnStop ([cometbft/cometbft\#1210](https://github.com/cometbft/cometbft/pull/1210): @chillyvee)
- `[node]` Make handshake cancelable ([cometbft/cometbft\#857](https://github.com/cometbft/cometbft/pull/857))
- `[p2p/pex]` Improve addrBook.hash performance
  ([\#6509](https://github.com/tendermint/tendermint/pull/6509))
- `[pubsub/kvindexer]` Numeric query conditions and event values are represented as big floats with default precision of 125.
  Integers are read as "big ints" and represented with as many bits as they need when converting to floats.
  ([\#797](https://github.com/cometbft/cometbft/pull/797))
- `[pubsub]` Performance improvements for the event query API
  ([\#7319](https://github.com/tendermint/tendermint/pull/7319))
- `[rpc]` Remove response data from response failure logs in order
  to prevent large quantities of log data from being produced
  ([\#654](https://github.com/cometbft/cometbft/issues/654))
- `[state]` Make logging `block_app_hash` and `app_hash` consistent by logging them both as hex.
  ([\#1264](https://github.com/cometbft/cometbft/pull/1264))

## v0.37.0

*March 6, 2023*

This is the first CometBFT release with ABCI 1.0, which introduces the
`PrepareProposal` and `ProcessProposal` methods, with the aim of expanding the
range of use cases that application developers can address. This is the first
change to ABCI towards ABCI++, and the full range of ABCI++ functionality will
only become available in the next major release with ABCI 2.0. See the
[specification](./spec/abci/) for more details.

In the v0.34.27 release, the CometBFT Go module is still
`github.com/tendermint/tendermint` to facilitate ease of upgrading for users,
but in this release we have changed this to `github.com/cometbft/cometbft`.

Please also see our [upgrading guidelines](./UPGRADING.md) for more details on
upgrading from the v0.34 release series.

Also see our [QA results](https://docs.cometbft.com/v0.37/qa/v037/cometbft) for
the v0.37 release.

We'd love your feedback on this release! Please reach out to us via one of our
communication channels, such as [GitHub
Discussions](https://github.com/cometbft/cometbft/discussions), with any of your
questions, comments and/or concerns.

See below for more details.

### BREAKING CHANGES

- Bump minimum Go version to 1.20
  ([\#385](https://github.com/cometbft/cometbft/issues/385))
- Change spelling from British English to American. Rename
  `Subscription.Cancelled()` to `Subscription.Canceled()` in `libs/pubsub`
  ([\#9144](https://github.com/tendermint/tendermint/pull/9144))
- The `TMHOME` environment variable was renamed to `CMTHOME`, and all environment variables starting with `TM_` are instead prefixed with `CMT_`
  ([\#211](https://github.com/cometbft/cometbft/issues/211))
- [config] The boolean key `fastsync` is deprecated and replaced by
    `block_sync`. ([\#9259](https://github.com/tendermint/tendermint/pull/9259))
    At the same time, `block_sync` is also deprecated. In the next release,
    BlocSync will always be enabled and `block_sync` will be removed.
    ([\#409](https://github.com/cometbft/cometbft/issues/409))
- `[abci/counter]` Delete counter example app
  ([\#6684](https://github.com/tendermint/tendermint/pull/6684))
- `[abci/params]` Deduplicate `ConsensusParams` and `BlockParams` so
  only `types` proto definitions are use. Remove `TimeIotaMs` and use
  a hard-coded 1 millisecond value to ensure monotonically increasing
  block times. Rename `AppVersion` to `App` so as to not stutter.
  ([\#9287](https://github.com/tendermint/tendermint/pull/9287))
- `[abci]` Added cli commands for `PrepareProposal` and `ProcessProposal`.
  ([\#8656](https://github.com/tendermint/tendermint/pull/8656))
- `[abci]` Added cli commands for `PrepareProposal` and `ProcessProposal`.
  ([\#8901](https://github.com/tendermint/tendermint/pull/8901))
- `[abci]` Change the `key` and `value` fields from
  `[]byte` to `string` in the `EventAttribute` type.
  ([\#6403](https://github.com/tendermint/tendermint/pull/6403))
- `[abci]` Make length delimiter encoding consistent
  (`uint64`) between ABCI and P2P wire-level protocols
  ([\#5783](https://github.com/tendermint/tendermint/pull/5783))
- `[abci]` New ABCI methods `PrepareProposal` and `ProcessProposal` which give
  the app control over transactions proposed and allows for verification of
  proposed blocks. ([\#9301](https://github.com/tendermint/tendermint/pull/9301))
- `[abci]` Removes unused Response/Request `SetOption` from ABCI
  ([\#9145](https://github.com/tendermint/tendermint/pull/9145))
- `[abci]` Renamed `EvidenceType` to `MisbehaviorType` and `Evidence`
  to `Misbehavior` as a more accurate label of their contents.
  ([\#8216](https://github.com/tendermint/tendermint/pull/8216))
- `[abci]` Renamed `LastCommitInfo` to `CommitInfo` in preparation for vote
  extensions. ([\#9122](https://github.com/tendermint/tendermint/pull/9122))
- `[config]` Rename the fastsync section and the
  fast\_sync key blocksync and block\_sync respectively
  ([\#9259](https://github.com/tendermint/tendermint/pull/9259))
- `[p2p]` Reactor `Send`, `TrySend` and `Receive` renamed to `SendEnvelope`,
  `TrySendEnvelope` and `ReceiveEnvelope` to allow metrics to be appended to
  messages and measure bytes sent/received.
  ([\#230](https://github.com/cometbft/cometbft/pull/230))
- `[types]` Reduce the use of protobuf types in core logic. `ConsensusParams`,
  `BlockParams`, `ValidatorParams`, `EvidenceParams`, `VersionParams` have
  become native types.  They still utilize protobuf when being sent over
  the wire or written to disk.  Moved `ValidateConsensusParams` inside
  (now native type) `ConsensusParams`, and renamed it to `ValidateBasic`.
  ([\#9287](https://github.com/tendermint/tendermint/pull/9287))

### BUG FIXES

- `[blocksync]` handle the case when the sending
  queue is full: retry block request after a timeout
  ([\#9518](https://github.com/tendermint/tendermint/pull/9518))
- `[consensus]` ([\#386](https://github.com/cometbft/cometbft/pull/386)) Short-term fix for the case when `needProofBlock` cannot find previous block meta by defaulting to the creation of a new proof block. (@adizere)
  - Special thanks to the [Vega.xyz](https://vega.xyz/) team, and in particular to Zohar (@ze97286), for reporting the problem and working with us to get to a fix.
- `[consensus]` Fixed a busy loop that happened when sending of a block part failed by sleeping in case of error.
  ([\#4](https://github.com/informalsystems/tendermint/pull/4))
- `[consensus]` fix round number of `enterPropose`
  when handling `RoundStepNewRound` timeout.
  ([\#9229](https://github.com/tendermint/tendermint/pull/9229))
- `[docker]` enable cross platform build using docker buildx
  ([\#9073](https://github.com/tendermint/tendermint/pull/9073))
- `[docker]` ensure Docker image uses consistent version of Go
  ([\#9462](https://github.com/tendermint/tendermint/pull/9462))
- `[p2p]` prevent peers who have errored from being added to `peer_set`
  ([\#9500](https://github.com/tendermint/tendermint/pull/9500))
- `[state/kvindexer]` Fixed the default behaviour of the kvindexer to index and
  query attributes by events in which they occur. In 0.34.25 this was mitigated
  by a separated RPC flag. @jmalicevic
  ([\#77](https://github.com/cometbft/cometbft/pull/77))
- `[state/kvindexer]` Resolved crashes when event values contained slashes,
  introduced after adding event sequences in
  [\#77](https://github.com/cometbft/cometbft/pull/77). @jmalicevic
  ([\#382](https://github.com/cometbft/cometbft/pull/382))

### FEATURES

- `[abci]` New ABCI methods `PrepareProposal` and `ProcessProposal` which give
  the app control over transactions proposed and allows for verification of
  proposed blocks. ([\#9301](https://github.com/tendermint/tendermint/pull/9301))

### IMPROVEMENTS

- `[abci]` Added `AbciVersion` to `RequestInfo` allowing
  applications to check ABCI version when connecting to CometBFT.
  ([\#5706](https://github.com/tendermint/tendermint/pull/5706))
- `[cli]` add `--hard` flag to rollback command (and a boolean to the `RollbackState` method). This will rollback
   state and remove the last block. This command can be triggered multiple times. The application must also rollback
   state to the same height.
  ([\#9171](https://github.com/tendermint/tendermint/pull/9171))
- `[consensus]` Save peer LastCommit correctly to achieve 50% reduction in gossiped precommits.
  ([\#9760](https://github.com/tendermint/tendermint/pull/9760))
- `[crypto]` Update to use btcec v2 and the latest btcutil.
  ([\#9250](https://github.com/tendermint/tendermint/pull/9250))
- `[e2e]` Add functionality for uncoordinated (minor) upgrades
  ([\#56](https://github.com/tendermint/tendermint/pull/56))
- `[p2p]` Reactor `Send`, `TrySend` and `Receive` renamed to `SendEnvelope`,
  `TrySendEnvelope` and `ReceiveEnvelope` to allow metrics to be appended to
  messages and measure bytes sent/received.
  ([\#230](https://github.com/cometbft/cometbft/pull/230))
- `[proto]` Migrate from `gogo/protobuf` to `cosmos/gogoproto`
  ([\#9356](https://github.com/tendermint/tendermint/pull/9356))
- `[rpc]` Added `header` and `header_by_hash` queries to the RPC client
  ([\#9276](https://github.com/tendermint/tendermint/pull/9276))
- `[rpc]` Enable caching of RPC responses
  ([\#9650](https://github.com/tendermint/tendermint/pull/9650))
- `[tools/tm-signer-harness]` Remove the folder as it is unused
  ([\#136](https://github.com/cometbft/cometbft/issues/136))

## v0.34.27

*Feb 27, 2023*

This is the first official release of CometBFT - a fork of [Tendermint
Core](https://github.com/tendermint/tendermint). This particular release is
intended to be compatible with the Tendermint Core v0.34 release series.

For details as to how to upgrade to CometBFT from Tendermint Core, please see
our [upgrading guidelines](./UPGRADING.md).

If you have any questions, comments, concerns or feedback on this release, we
would love to hear from you! Please contact us via [GitHub
Discussions](https://github.com/cometbft/cometbft/discussions),
[Discord](https://discord.gg/cosmosnetwork) (in the `#cometbft` channel) or
[Telegram](https://t.me/CometBFT).

Special thanks to @wcsiu, @ze97286, @faddat and @JayT106 for their contributions
to this release!

### BREAKING CHANGES

- Rename binary to `cometbft` and Docker image to `cometbft/cometbft`
  ([\#152](https://github.com/cometbft/cometbft/pull/152))
- The `TMHOME` environment variable was renamed to `CMTHOME`, and all
  environment variables starting with `TM_` are instead prefixed with `CMT_`
  ([\#211](https://github.com/cometbft/cometbft/issues/211))
- Use Go 1.19 to build CometBFT, since Go 1.18 has reached end-of-life.
  ([\#360](https://github.com/cometbft/cometbft/issues/360))

### BUG FIXES

- `[consensus]` Fixed a busy loop that happened when sending of a block part
  failed by sleeping in case of error.
  ([\#4](https://github.com/informalsystems/tendermint/pull/4))
- `[consensus]` Short-term fix for the case when `needProofBlock` cannot find
  previous block meta by defaulting to the creation of a new proof block.
  ([\#386](https://github.com/cometbft/cometbft/pull/386): @adizere)
  - Special thanks to the [Vega.xyz](https://vega.xyz/) team, and in particular
    to Zohar (@ze97286), for reporting the problem and working with us to get to
    a fix.
- `[p2p]` Correctly use non-blocking `TrySendEnvelope` method when attempting to
  send messages, as opposed to the blocking `SendEnvelope` method. It is unclear
  whether this has a meaningful impact on P2P performance, but this patch does
  correct the underlying behaviour to what it should be
  ([tendermint/tendermint\#9936](https://github.com/tendermint/tendermint/pull/9936))
- `[state/kvindexer]` Resolved crashes when event values contained slashes,
  introduced after adding event sequences.
  (\#[383](https://github.com/cometbft/cometbft/pull/383): @jmalicevic)

### DEPENDENCIES

- Bump tm-load-test to v1.3.0 to remove implicit dependency on Tendermint Core
  ([\#165](https://github.com/cometbft/cometbft/pull/165))
- Replace [tm-db](https://github.com/tendermint/tm-db) with
  [cometbft-db](https://github.com/cometbft/cometbft-db)
  ([\#160](https://github.com/cometbft/cometbft/pull/160))
- `[crypto]` Update to use btcec v2 and the latest btcutil
  ([tendermint/tendermint\#9787](https://github.com/tendermint/tendermint/pull/9787):
  @wcsiu)

### FEATURES

- `[rpc]` Add `match_event` query parameter to indicate to the RPC that it
  should match events _within_ attributes, not only within a height
  ([tendermint/tendermint\#9759](https://github.com/tendermint/tendermint/pull/9759))

### IMPROVEMENTS

- Append the commit hash to the version of CometBFT being built
  ([\#204](https://github.com/cometbft/cometbft/pull/204))
- `[consensus]` Add `consensus_block_gossip_parts_received` and
  `consensus_step_duration_seconds` metrics in order to aid in investigating the
  impact of database compaction on consensus performance
  ([tendermint/tendermint\#9733](https://github.com/tendermint/tendermint/pull/9733))
- `[consensus]` Reduce bandwidth consumption of consensus votes by roughly 50%
  through fixing a small logic bug
  ([tendermint/tendermint\#9776](https://github.com/tendermint/tendermint/pull/9776))
- `[e2e]` Add functionality for uncoordinated (minor) upgrades
  ([\#56](https://github.com/tendermint/tendermint/pull/56))
- `[mempool/v1]` Suppress "rejected bad transaction" in priority mempool logs by
  reducing log level from info to debug
  ([\#314](https://github.com/cometbft/cometbft/pull/314): @JayT106)
- `[p2p]` Reduce log spam through reducing log level of "Dialing peer" and
  "Added peer" messages from info to debug
  ([tendermint/tendermint\#9764](https://github.com/tendermint/tendermint/pull/9764):
  @faddat)
- `[state/kvindexer]` Add `match.event` keyword to support condition evaluation
  based on the event the attributes belong to
  ([tendermint/tendermint\#9759](https://github.com/tendermint/tendermint/pull/9759))
- `[tools/tm-signer-harness]` Remove the folder as it is unused
  ([\#136](https://github.com/cometbft/cometbft/issues/136))

---

CometBFT is a fork of [Tendermint Core](https://github.com/tendermint/tendermint) as of late December 2022.

## Bug bounty

Friendly reminder, we have a [bug bounty program](https://hackerone.com/cosmos).

## Previous changes

For changes released before the creation of CometBFT, please refer to the Tendermint Core [CHANGELOG.md](https://github.com/tendermint/tendermint/blob/a9feb1c023e172b542c972605311af83b777855b/CHANGELOG.md).

