# RFC 005: Optimistic Execution

## Changelog

- 2023-06-07: Refactor for Cosmos SDK (@facundomedica)
- 2022-08-16: Initial draft by Sei Network

## Background

Before ABCI++, the first and only time a CometBFT blockchain's application layer would know about a block proposal is after the voting period, at which point CometBFT would invoke `BeginBlock`, `DeliverTx`, `EndBlock`, and `Commit` ABCI methods of the application, with the block proposal contents passed in.

With the introduction of ABCI++, the application layer now receives the block proposal before the voting period commences. This can be used to optimistically execute the block proposal in parallel with the voting process, thus reducing the block time.

## Proposal

Given that the application receives the block proposal in an earlier stage (`ProcessProposal`), it can be executed in the background so when `FinalizeBlock` is called the response can returned instantly.

## Decision

The newly introduced ABCI method `ProcessProposal` is called after a node receives the full block proposal of the current height but before prevote starts. CometBFT states that preemptively executing the block proposal is a potential use case for it:

> - **Usage**:
>   - Contains all information on the proposed block needed to fully execute it.
>     - The Application may fully execute the block as though it was handling
>       `RequestFinalizeBlock`.
>     - However, any resulting state changes must be kept as _candidate state_,
>       and the Application should be ready to discard it in case another block is decided.
>   - The Application MAY fully execute the block &mdash; immediate execution

Nevertheless, synchronously executing the proposal preemptively would not improve block time because it would just change the order of events (so the time we would like to save will be spent at `ProcessProposal` instead of `FinalizeBlock`).

Instead, we need to make block execution asynchronous by starting a goroutine in `ProcessProposal` (whose termination signal is kept in the application context) and returning a response immediately. That way, the actual block execution would happen at the same time as voting. When voting finishes and `FinalizeBlock` is called, the application handler can wait for the previously started goroutine to finish, and commit the resulting cache store if the block hash matches.

Assuming average voting period takes `P` and average block execution takes `Q`, this would reduce the average block time by `P + Q - max(P, Q)`.

Sei Network reported `P=~600ms` and `Q=~300ms` during a load test, meaning that optimistic execution could cut the block time by ~300ms.

The following diagram illustrates the intended flow:

```mermaid
flowchart LR;
    Propose-->ProcessProposal;
    ProcessProposal-->Prevote;
    Precommit-->FinalizeBlock;
    FinalizeBlock-->Commit;
    subgraph CometBFT
    Propose
    Prevote-->Precommit;
    Commit
    end
    subgraph CosmosSDK
    OEG[Optimistic Execution Goroutine]-.->FinalizeBlock
    ProcessProposal-.->OEG
    FinalizeBlock
    end
```

Some considerations:

- In the case that a proposal is being rejected by the local node, this node won't attempt to execute the proposal.
- The app must drop any previous Optimistic Execution data if `ProcessProposal` is called, in other words, abort and restart.

## Implementation

The execution context needs to have the following information:

- The block proposal (`abci.RequestFinalizeBlock`)
- Termination and completion signal for the OE goroutine

The OE goroutine would run on top of a cached branch of the root multi-store (which is the default behavior for `FinalizeBlock` as we only write to the underlying store once we've reached the end).

The OE goroutine would periodically check if a termination signal has been sent to it, and stops if so. Once the OE goroutine finishes the execution it will set the completion signal.

The application developers will have the option to enable or disable Optimistic Execution, being disabled by default. To enable it, the application will need to pass the `SetOptimisticExecution` option to `NewBaseApp`. This is because this feature will be considered experimental until it's proven to be stable. Note that individual nodes should not enable/disable this feature on their own, as it could lead to inconsistent behavior.

Upon receiving a `ProcessProposal` call, the SDK would adopt the following procedure if OE is enabled:

```
abort any running OE goroutine and wait for goroutine exit
if height > initial height:
    set OE fields
    kick off an OP goroutine that optimistically process the proposal
else:
    do nothing
respond to CometBFT
```

OE won't be enabled during the first block of the chain, regardless of the configuration. This is because during the first block `InitChain` writes to `finalizeBlockState` so `FinalizeBlock` starts with an already initialized state. In the case that an abort would be needed it would mean we need to run `InitChain` again or discard/recover the state; either way complicates the implementation too much just for a single block.

Upon receiving a `FinalizeBlock` call, the SDK would adopt the following procedure:

```
if OE is enabled:
    abort if the currently executing OE's hash doesn't match the proposal's hash
    if aborted:
        process the proposal synchronously
    else:
        wait for the OE goroutine to finish
        respond to CometBFT
```

## Consequences

### Backwards Compatibility

This can only be implemented on SDK versions using ABCI++, meaning v0.50.x and above.

### Positive

- Shorter block times for the same amount of transactions

### Negative

- Adds some complexity to the SDK
- Multiple rounds on a block that contains long running transactions could be problematic

### Neutral

- Each node can decide whether to use this feature or not.

### References

- [Original RFC by Sei Network](https://github.com/sei-protocol/sei-chain/blob/81b8af7980df722a63a910cc35ff96e60a94cbfe/docs/rfc/rfc-000-optimistic-proposal-processing.md)
- [ABCI++ methods in CometBFT](https://github.com/cometbft/cometbft/blob/a09f5d33ecd8846369b93cae9063291eb8abc3a0/spec/abci/abci%2B%2B_methods.md)
