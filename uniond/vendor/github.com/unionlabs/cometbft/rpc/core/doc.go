/*
Package core defines the CometBFT RPC endpoints.

CometBFT ships with its own JSONRPC library -
https://github.com/cometbft/cometbft/tree/main/rpc/jsonrpc.

## Get the list

An HTTP Get request to the root RPC endpoint shows a list of available endpoints.

```bash
curl "http://localhost:26657"  | textutil -stdin -convert txt -stdout | sed 's/\/\/localhost:26657//g'
```

> Response:

```plain
Available endpoints:
/abci_info
/dump_consensus_state
/genesis
/health
/net_info
/num_unconfirmed_txs
/status
/unsafe_flush_mempool
/unsubscribe_all?

Endpoints that require arguments:
/abci_query?path=_&data=_&height=_&prove=_
/block?height=_
/block_by_hash?hash=_
/block_results?height=_
/block_search?query=_&page=_&per_page=_&order_by=_
/blockchain?minHeight=_&maxHeight=_
/broadcast_evidence?evidence=_
/broadcast_tx_async?tx=_
/broadcast_tx_commit?tx=_
/broadcast_tx_sync?tx=_
/check_tx?tx=_
/commit?height=_
/consensus_params?height=_
/consensus_state?
/genesis_chunked?chunk=_
/header?height=_
/header_by_hash?hash=_
/subscribe?query=_
/tx?hash=_&prove=_
/tx_search?query=_&prove=_&page=_&per_page=_&order_by=_
/unconfirmed_txs?limit=_
/unsubscribe?query=_
/validators?height=_&page=_&per_page=_
```
*/
package core
