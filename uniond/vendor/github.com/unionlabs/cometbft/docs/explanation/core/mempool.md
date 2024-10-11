---
order: 12
---

# Mempool

A mempool (a contraction of memory and pool) is a node’s data structure for
storing information on uncommitted transactions. It acts as a sort of waiting
room for transactions that have not yet been committed.

CometBFT currently supports two types of mempools: `flood` and `nop`.

## 1. Flood

The `flood` mempool stores transactions in a concurrent linked list. When a new
transaction is received, it first checks if there's a space for it (`size` and
`max_txs_bytes` config options) and that it's not too big (`max_tx_bytes` config
option). Then, it checks if this transaction has already been seen before by using
an LRU cache (`cache_size` regulates the cache's size). If all checks pass and
the transaction is not in the cache (meaning it's new), the ABCI
[`CheckTxAsync`][1] method is called. The ABCI application validates the
transaction using its own rules.

If the transaction is deemed valid by the ABCI application, it's added to the linked list.

The mempool's name (`flood`) comes from the dissemination mechanism. When a new
transaction is added to the linked list, the mempool sends it to all connected
peers. Peers themselves gossip this transaction to their peers and so on. One
can say that each transaction "floods" the network, hence the name `flood`.

Note there are experimental config options
`experimental_max_gossip_connections_to_persistent_peers` and
`experimental_max_gossip_connections_to_non_persistent_peers` to limit the
number of peers a transaction is broadcasted to. Also, you can turn off
broadcasting with `broadcast` config option.

After each committed block, CometBFT rechecks all uncommitted transactions (can
be disabled with the `recheck` config option) by repeatedly calling the ABCI
`CheckTxAsync`.

### Transaction ordering

Currently, there's no ordering of transactions other than the order they've
arrived (via RPC or from other nodes).

So the only way to specify the order is to send them to a single node.

valA:

- `tx1`
- `tx2`
- `tx3`

If the transactions are split up across different nodes, there's no way to
ensure they are processed in the expected order.

valA:

- `tx1`
- `tx2`

valB:

- `tx3`

If valB is the proposer, the order might be:

- `tx3`
- `tx1`
- `tx2`

If valA is the proposer, the order might be:

- `tx1`
- `tx2`
- `tx3`

That said, if the transactions contain some internal value, like an
order/nonce/sequence number, the application can reject transactions that are
out of order. So if a node receives `tx3`, then `tx1`, it can reject `tx3` and then
accept `tx1`. The sender can then retry sending `tx3`, which should probably be
rejected until the node has seen `tx2`.

## 2. Nop

`nop` (short for no operation) mempool is used when the ABCI application developer wants to
build their own mempool. When `type = "nop"`, transactions are not stored anywhere
and are not gossiped to other peers using the P2P network.

Submitting a transaction via the existing RPC methods (`BroadcastTxSync`,
`BroadcastTxAsync`, and `BroadcastTxCommit`) will always result in an error.

Because there's no way for the consensus to know if transactions are available
to be committed, the node will always create blocks, which can be empty
sometimes. Using `consensus.create_empty_blocks=false` is prohibited in such
cases.

The ABCI application becomes responsible for storing, disseminating, and
proposing transactions using [`PrepareProposal`][2]. The concrete design is up
to the ABCI application developers.

[1]: ../../../spec/abci/abci++_methods.md#checktx
[2]: ../../../spec/abci/abci++_methods.md#prepareproposal
