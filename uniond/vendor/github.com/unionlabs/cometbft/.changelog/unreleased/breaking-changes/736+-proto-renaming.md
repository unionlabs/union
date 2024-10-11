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
