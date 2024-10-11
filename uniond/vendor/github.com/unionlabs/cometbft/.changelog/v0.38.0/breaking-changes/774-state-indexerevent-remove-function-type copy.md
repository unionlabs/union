- `[rpc]` Removed `begin_block_events` and `end_block_events` from `BlockResultsResponse`.
  The events are merged into one field called `finalize_block_events`.
  ([\#9427](https://github.com/tendermint/tendermint/issues/9427))
