---
order: 2
---
# Creating a proposal

A block consists of a header, transactions, votes (the commit),
and a list of evidence of misbehavior (e.g. duplicate vote).

Outstanding evidence items get priority over outstanding transactions in the mempool.
All in all, the block MUST NOT exceed  `ConsensusParams.Block.MaxBytes`,
or 100MB if `ConsensusParams.Block.MaxBytes == -1`.

## Reaping transactions from the mempool

When we reap transactions from the mempool, we calculate maximum data
size by subtracting maximum header size (`MaxHeaderBytes`), the maximum
protobuf overhead for a block (`MaxOverheadForBlock`), the size of
the last commit (if present) and evidence (if present). While reaping
we account for protobuf overhead for each transaction.

```go
func MaxDataBytes(maxBytes, evidenceBytes int64, valsCount int) int64 {
  return maxBytes -
  MaxOverheadForBlock -
  MaxHeaderBytes -
  MaxCommitBytes(valsCount) -
  evidenceBytes
}
```

If `ConsensusParams.Block.MaxBytes == -1`, we reap *all* outstanding transactions from the mempool

## Preparing the proposal

Once the transactions have been reaped from the mempool according to the rules described above,
CometBFT calls `PrepareProposal` to the application with the transaction list that has just been reaped.
As part of this call the application can remove, add, or reorder transactions in the transaction list.

The `PrepareProposalRequest` contains two important fields:

* `MaxTxBytes`, which contains the value returned by `MaxDataBytes` described above.
  The application MUST NOT return a list of transactions whose size exceeds this number.
* `Txs`, which contains the list of reaped transactions.

For more details on `PrepareProposal`, please see the
[relevant part of the spec](../abci/abci%2B%2B_methods.md#prepareproposal)

## Validating transactions in the mempool

Before we accept a transaction in the mempool, we check if its size is no more
than {MaxDataSize}. {MaxDataSize} is calculated using the same formula as
above, except we assume there is no evidence.

```go
func MaxDataBytesNoEvidence(maxBytes int64, valsCount int) int64 {
  return maxBytes -
    MaxOverheadForBlock -
    MaxHeaderBytes -
    MaxCommitBytes(valsCount)
}
```
