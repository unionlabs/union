- `[rpc]` The endpoints `broadcast_tx_*` now return an error when the node is
  performing block sync or state sync.
  ([\#785](https://github.com/cometbft/cometbft/issues/785))
- `[mempool]` When the node is performing block sync or state sync, the mempool
  reactor now discards incoming transactions from peers, and does not propagate
  transactions to peers.
  ([\#785](https://github.com/cometbft/cometbft/issues/785))
