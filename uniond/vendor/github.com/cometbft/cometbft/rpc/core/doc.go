/*
Package core defines the CometBFT RPC endpoints.

CometBFT ships with its own JSONRPC library -
https://github.com/cometbft/cometbft/tree/v0.38.x/rpc/jsonrpc.

## Get the list

An HTTP Get request to the root RPC endpoint shows a list of available endpoints.

```bash
curl 'localhost:26657'
```

> Response:

```plain
Available endpoints:
/abci_info
/dump_consensus_state
/genesis
/net_info
/num_unconfirmed_txs
/status
/health
/unconfirmed_txs
/unsafe_flush_mempool
/validators

Endpoints that require arguments:
/abci_query?path=_&data=_&prove=_
/block?height=_
/blockchain?minHeight=_&maxHeight=_
/broadcast_tx_async?tx=_
/broadcast_tx_commit?tx=_
/broadcast_tx_sync?tx=_
/commit?height=_
/dial_seeds?seeds=_
/dial_persistent_peers?persistent_peers=_
/subscribe?event=_
/tx?hash=_&prove=_
/unsubscribe?event=_
```
*/
package core
