# ADR 037: Deliver Block

Author: Daniil Lashin (@danil-lashin)

## Changelog

13-03-2019: Initial draft

## Context

Initial conversation: https://github.com/tendermint/tendermint/issues/2901

Some applications can handle transactions in parallel, or at least some
part of tx processing can be parallelized. Now it is not possible for developer
to execute txs in parallel because Tendermint delivers them consequentially.

## Decision

Now Tendermint have `BeginBlock`, `EndBlock`, `Commit`, `DeliverTx` steps
while executing block. This doc proposes merging this steps into one `DeliverBlock`
step. It will allow developers of applications to decide how they want to
execute transactions (in parallel or consequentially). Also it will simplify and
speed up communications between application and Tendermint.

As @jaekwon [mentioned](https://github.com/tendermint/tendermint/issues/2901#issuecomment-477746128)
in discussion not all application will benefit from this solution. In some cases,
when application handles transaction consequentially, it way slow down the blockchain,
because it need to wait until full block is transmitted to application to start
processing it. Also, in the case of complete change of ABCI, we need to force all the apps
to change their implementation completely. That's why I propose to introduce one more ABCI
type.

# Implementation Changes

In addition to default application interface which now have this structure

```go
type Application interface {
    // Info and Mempool methods...

    // Consensus Connection
    InitChain(RequestInitChain) ResponseInitChain    // Initialize blockchain with validators and other info from TendermintCore
    BeginBlock(RequestBeginBlock) ResponseBeginBlock // Signals the beginning of a block
    DeliverTx(tx []byte) ResponseDeliverTx           // Deliver a tx for full processing
    EndBlock(RequestEndBlock) ResponseEndBlock       // Signals the end of a block, returns changes to the validator set
    Commit() ResponseCommit                          // Commit the state and return the application Merkle root hash
}
```

this doc proposes to add one more:

```go
type Application interface {
    // Info and Mempool methods...

    // Consensus Connection
    InitChain(RequestInitChain) ResponseInitChain           // Initialize blockchain with validators and other info from TendermintCore
    DeliverBlock(RequestDeliverBlock) ResponseDeliverBlock  // Deliver full block
    Commit() ResponseCommit                                 // Commit the state and return the application Merkle root hash
}

type RequestDeliverBlock struct {
    Hash                 []byte
    Header               Header
    Txs                  Txs
    LastCommitInfo       LastCommitInfo
    ByzantineValidators  []Evidence
}

type ResponseDeliverBlock struct {
    ValidatorUpdates      []ValidatorUpdate
    ConsensusParamUpdates *ConsensusParams
    Tags                  []kv.Pair
    TxResults             []ResponseDeliverTx
}

```

Also, we will need to add new config param, which will specify what kind of ABCI application uses.
For example, it can be `abci_type`. Then we will have 2 types:
- `advanced` - current ABCI
- `simple` - proposed implementation

## Status

In review

## Consequences

### Positive

- much simpler introduction and tutorials for new developers (instead of implementing 5 methods whey
will need to implement only 3)
- txs can be handled in parallel
- simpler interface
- faster communications between Tendermint and application

### Negative

- Tendermint should now support 2 kinds of ABCI
