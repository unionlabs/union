# ADR 029: Check block txs before prevote

## Changelog

04-10-2018: Update with link to issue
[#2384](https://github.com/tendermint/tendermint/issues/2384) and reason for rejection
19-09-2018: Initial Draft

## Context

We currently check a tx's validity through 2 ways.

1. Through checkTx in mempool connection.
2. Through deliverTx in consensus connection.

The 1st is called when external tx comes in, so the node should be a proposer this time. The 2nd is called when external block comes in and reach the commit phase, the node doesn't need to be the proposer of the block, however it should check the txs in that block.

In the 2nd situation, if there are many invalid txs in the block, it would be too late for all nodes to discover that most txs in the block are invalid, and we'd better not record invalid txs in the blockchain too.

## Proposed solution

Therefore, we should find a way to check the txs' validity before send out a prevote. Currently we have cs.isProposalComplete() to judge whether a block is complete. We can have

```
func (blockExec *BlockExecutor) CheckBlock(block *types.Block) error {
   // check txs of block.
   for _, tx := range block.Txs {
      reqRes := blockExec.proxyApp.CheckTxAsync(tx)
      reqRes.Wait()
      if reqRes.Response == nil || reqRes.Response.GetCheckTx() == nil || reqRes.Response.GetCheckTx().Code != abci.CodeTypeOK {
         return errors.Errorf("tx %v check failed. response: %v", tx, reqRes.Response)
      }
   }
   return nil
}
```

such a method in BlockExecutor to check all txs' validity in that block.

However, this method should not be implemented like that, because checkTx will share the same state used in mempool in the app.  So we should define a new interface method checkBlock in Application to indicate it to use the same state as deliverTx.

```
type Application interface {
   // Info/Query Connection
   Info(RequestInfo) ResponseInfo                // Return application info
   Query(RequestQuery) ResponseQuery             // Query for state

   // Mempool Connection
   CheckTx(tx []byte) ResponseCheckTx // Validate a tx for the mempool

   // Consensus Connection
   InitChain(RequestInitChain) ResponseInitChain // Initialize blockchain with validators and other info from TendermintCore
   CheckBlock(RequestCheckBlock) ResponseCheckBlock
   BeginBlock(RequestBeginBlock) ResponseBeginBlock // Signals the beginning of a block
   DeliverTx(tx []byte) ResponseDeliverTx           // Deliver a tx for full processing
   EndBlock(RequestEndBlock) ResponseEndBlock       // Signals the end of a block, returns changes to the validator set
   Commit() ResponseCommit                          // Commit the state and return the application Merkle root hash
}
```

All app should implement that method. For example, counter:

```
func (app *CounterApplication) CheckBlock(block types.Request_CheckBlock) types.ResponseCheckBlock {
   if app.serial {
   	  app.originalTxCount = app.txCount   //backup the txCount state
      for _, tx := range block.CheckBlock.Block.Txs {
         if len(tx) > 8 {
            return types.ResponseCheckBlock{
               Code: code.CodeTypeEncodingError,
               Log:  fmt.Sprintf("Max tx size is 8 bytes, got %d", len(tx))}
         }
         tx8 := make([]byte, 8)
         copy(tx8[len(tx8)-len(tx):], tx)
         txValue := binary.BigEndian.Uint64(tx8)
         if txValue < uint64(app.txCount) {
            return types.ResponseCheckBlock{
               Code: code.CodeTypeBadNonce,
               Log:  fmt.Sprintf("Invalid nonce. Expected >= %v, got %v", app.txCount, txValue)}
         }
         app.txCount++
      }
   }
   return types.ResponseCheckBlock{Code: code.CodeTypeOK}
}
```

In BeginBlock, the app should restore the state to the original state before checking the block:

```
func (app *CounterApplication) DeliverTx(tx []byte) types.ResponseDeliverTx {
   if app.serial {
      app.txCount = app.originalTxCount   //restore the txCount state
   }
   app.txCount++
   return types.ResponseDeliverTx{Code: code.CodeTypeOK}
}
```

The txCount is like the nonce in ethermint, it should be restored when entering the deliverTx phase. While some operation like checking the tx signature needs not to be done again. So the deliverTx can focus on how a tx can be applied, ignoring the checking of the tx, because all the checking has already been done in the checkBlock phase before.

An optional optimization is alter the deliverTx to deliverBlock. For the block has already been checked by checkBlock, so all the txs in it are valid. So the app can cache the block, and in the deliverBlock phase, it just needs to apply the block in the cache. This optimization can save network current in deliverTx.



## Status

Rejected

## Decision

Performance impact is considered too great. See [#2384](https://github.com/tendermint/tendermint/issues/2384)

## Consequences

### Positive

- more robust to defend the adversary to propose a block full of invalid txs.

### Negative

- add a new interface method. app logic needs to adjust to appeal to it.
- sending all the tx data over the ABCI twice
- potentially redundant validations (eg. signature checks in both CheckBlock and
  DeliverTx)

### Neutral
