# RFC 013: ABCI++

## Changelog

- 2020-01-11: initialized
- 2022-02-11: Migrate RFC to tendermint repo (Originally [RFC 004](https://github.com/tendermint/spec/pull/254))

## Author(s)

- Dev (@valardragon)
- Sunny (@sunnya97)

## Context

ABCI is the interface between the consensus engine and the application.
It defines when the application can talk to consensus during the execution of a blockchain.
At the moment, the application can only act at one phase in consensus, immediately after a block has been finalized.

This restriction on the application prohibits numerous features for the application, including many scalability improvements that are now better understood than when ABCI was first written.
For example, many of the scalability proposals can be boiled down to "Make the miner / block proposers / validators do work, so the network does not have to".
This includes optimizations such as tx-level signature aggregation, state transition proofs, etc.
Furthermore, many new security properties cannot be achieved in the current paradigm, as the application cannot enforce validators do more than just finalize txs.
This includes features such as threshold cryptography, and guaranteed IBC connection attempts.
We propose introducing three new phases to ABCI to enable these new features, and renaming the existing methods for block execution.

#### Prepare Proposal phase

This phase aims to allow the block proposer to perform more computation, to reduce load on all other full nodes, and light clients in the network.
It is intended to enable features such as batch optimizations on the transaction data (e.g. signature aggregation, zk rollup style validity proofs, etc.), enabling stateless blockchains with validator provided authentication paths, etc.

This new phase will only be executed by the block proposer. The application will take in the block header and raw transaction data output by the consensus engine's mempool. It will then return block data that is prepared for gossip on the network, and additional fields to include into the block header.

#### Process Proposal Phase

This phase aims to allow applications to determine validity of a new block proposal, and execute computation on the block data, prior to the blocks finalization.
It is intended to enable applications to reject block proposals with invalid data, and to enable alternate pipelined execution models. (Such as Ethereum-style immediate execution)

This phase will be executed by all full nodes upon receiving a block, though on the application side it can do more work in the even that the current node is a validator.

#### Vote Extension Phase

This phase aims to allow applications to require their validators do more than just validate blocks.
Example usecases of this include validator determined price oracles, validator guaranteed IBC connection attempts, and validator based threshold crypto.

This adds an app-determined data field that every validator must include with their vote, and these will thus appear in the header.

#### Rename {BeginBlock, [DeliverTx], EndBlock} to FinalizeBlock

The prior phases gives the application more flexibility in their execution model for a block, and they obsolete the current methods for how the consensus engine relates the block data to the state machine. Thus we refactor the existing methods to better reflect what is happening in the new ABCI model.

This rename doesn't on its own enable anything new, but instead improves naming to clarify the expectations from the application in this new communication model. The existing ABCI methods  `BeginBlock, [DeliverTx], EndBlock` are renamed to a single method called `FinalizeBlock`.

#### Summary

We include a more detailed list of features / scaling improvements that are blocked, and which new phases resolve them at the end of this document.

 <image src="images/abci.png" style="float: left; width: 40%;" /> <image src="images/abci++.png" style="float: right; width: 40%;" />
On the top is the existing definition of ABCI, and on the bottom is the proposed ABCI++.

## Proposal

Below we suggest an API to add these three new phases.
In this document, sometimes the final round of voting is referred to as precommit for clarity in how it acts in the Tendermint case.

### Prepare Proposal

*Note, APIs in this section will change after Vote Extensions, we list the adjusted APIs further in the proposal.*

The Prepare Proposal phase allows the block proposer to perform application-dependent work in a block, to lower the amount of work the rest of the network must do. This enables batch optimizations to a block, which has been empirically demonstrated to be a key component for scaling. This phase introduces the following ABCI method

```rust
fn PrepareProposal(Block) -> BlockData
```

where `BlockData` is a type alias for however data is internally stored within the consensus engine. In Tendermint Core today, this is `[]Tx`.

The application may read the entire block proposal, and mutate the block data fields. Mutated transactions will still get removed from the mempool later on, as the mempool rechecks all transactions after a block is executed.

The `PrepareProposal` API will be modified in the vote extensions section, for allowing the application to modify the header.

### Process Proposal

The Process Proposal phase sends the block data to the state machine, prior to running the last round of votes on the state machine. This enables features such as allowing validators to reject a block according to whether state machine deems it valid, and changing block execution pipeline.

We introduce three new methods,

```rust
fn VerifyHeader(header: Header, isValidator: bool) -> ResponseVerifyHeader {...}
fn ProcessProposal(block: Block) -> ResponseProcessProposal {...}
fn RevertProposal(height: usize, round: usize) {...}
```

where

```rust
struct ResponseVerifyHeader {
    accept_header: bool,
    evidence: Vec<Evidence>
}
struct ResponseProcessProposal {
    accept_block: bool,
    evidence: Vec<Evidence>
}
```

Upon receiving a block header, every validator runs `VerifyHeader(header, isValidator)`. The reason for why `VerifyHeader` is split from `ProcessProposal` is due to the later sections for Preprocess Proposal and Vote Extensions, where there may be application dependent data in the header that must be verified before accepting the header.
If the returned `ResponseVerifyHeader.accept_header` is false, then the validator must precommit nil on this block, and reject all other precommits on this block. `ResponseVerifyHeader.evidence` is appended to the validators local `EvidencePool`.

Upon receiving an entire block proposal (in the current implementation, all "block parts"), every validator runs `ProcessProposal(block)`. If the returned `ResponseProcessProposal.accept_block` is false, then the validator must precommit nil on this block, and reject all other precommits on this block. `ResponseProcessProposal.evidence` is appended to the validators local `EvidencePool`.

Once a validator knows that consensus has failed to be achieved for a given block, it must run `RevertProposal(block.height, block.round)`, in order to signal to the application to revert any potentially mutative state changes it may have made. In Tendermint, this occurs when incrementing rounds.

**RFC**: How do we handle the scenario where honest node A finalized on round x, and honest node B finalized on round x + 1? (e.g. when 2f precommits are publicly known, and a validator precommits themself but doesn't broadcast, but they increment rounds) Is this a real concern? The state root derived could change if everyone finalizes on round x+1, not round x, as the state machine can depend non-uniformly on timestamp.

The application is expected to cache the block data for later execution.

The `isValidator` flag is set according to whether the current node is a validator or a full node. This is intended to allow for beginning validator-dependent computation that will be included later in vote extensions. (An example of this is threshold decryptions of ciphertexts.)

### DeliverTx rename to FinalizeBlock

After implementing `ProcessProposal`, txs no longer need to be delivered during the block execution phase. Instead, they are already in the state machine. Thus `BeginBlock, DeliverTx, EndBlock` can all be replaced with a single ABCI method for `ExecuteBlock`. Internally the application may still structure its method for executing the block as `BeginBlock, DeliverTx, EndBlock`. However, it is overly restrictive to enforce that the block be executed after it is finalized. There are multiple other, very reasonable pipelined execution models one can go for. So instead we suggest calling this succession of methods `FinalizeBlock`. We propose the following API

Replace the `BeginBlock, DeliverTx, EndBlock` ABCI methods with the following method

```rust
fn FinalizeBlock() -> ResponseFinalizeBlock
```

where `ResponseFinalizeBlock` has the following API, in terms of what already exists

```rust
struct ResponseFinalizeBlock {
    updates: ResponseEndBlock,
    tx_results: Vec<ResponseDeliverTx>
}
```

`ResponseEndBlock` should then be renamed to `ConsensusUpdates` and `ResponseDeliverTx` should be renamed to `ResponseTx`.

### Vote Extensions

The Vote Extensions phase allow applications to force their validators to do more than just validate within consensus. This is done by allowing the application to add more data to their votes, in the final round of voting. (Namely the precommit)
This additional application data will then appear in the block header.

First we discuss the API changes to the vote struct directly

```rust
fn ExtendVote(height: u64, round: u64) -> (UnsignedAppVoteData, SelfAuthenticatingAppData)
fn VerifyVoteExtension(signed_app_vote_data: Vec<u8>, self_authenticating_app_vote_data: Vec<u8>) -> bool
```

There are two types of data that the application can enforce validators to include with their vote.
There is data that the app needs the validator to sign over in their vote, and there can be self-authenticating vote data. Self-authenticating here means that the application upon seeing these bytes, knows its valid, came from the validator and is non-malleable. We give an example of each type of vote data here, to make their roles clearer.

- Unsigned app vote data: A use case of this is if you wanted validator backed oracles, where each validator independently signs some oracle data in their vote, and the median of these values is used on chain. Thus we leverage consensus' signing process for convenience, and use that same key to sign the oracle data.
- Self-authenticating vote data: A use case of this is in threshold random beacons. Every validator produces a threshold beacon share. This threshold beacon share can be verified by any node in the network, given the share and the validators public key (which is not the same as its consensus public key). However, this decryption share will not make it into the subsequent block's header. They will be aggregated by the subsequent block proposer to get a single random beacon value that will appear in the subsequent block's header. Everyone can then verify that this aggregated value came from the requisite threshold of the validator set, without increasing the bandwidth for full nodes or light clients. To achieve this goal, the self-authenticating vote data cannot be signed over by the consensus key along with the rest of the vote, as that would require all full nodes & light clients to know this data in order to verify the vote.

The `CanonicalVote` struct will accommodate the `UnsignedAppVoteData` field by adding another string to its encoding, after the `chain-id`. This should not interfere with existing hardware signing integrations, as it does not affect the constant offset for the `height` and `round`, and the vote size does not have an explicit upper bound. (So adding this unsigned app vote data field is equivalent from the HSM's perspective as having a superlong chain-ID)

**RFC**: Please comment if you think it will be fine to have elongate the message the HSM signs, or if we need to explore pre-hashing the app vote data.

The flow of these methods is that when a validator has to precommit, Tendermint will first produce a precommit canonical vote without the application vote data. It will then pass it to the application, which will return unsigned application vote data, and self authenticating application vote data. It will bundle the `unsigned_application_vote_data` into the canonical vote, and pass it to the HSM to sign. Finally it will package the self-authenticating app vote data, and the `signed_vote_data` together, into one final Vote struct to be passed around the network.

#### Changes to Prepare Proposal Phase

There are many use cases where the additional data from vote extensions can be batch optimized.
This is mainly of interest when the votes include self-authenticating app vote data that be batched together, or the unsigned app vote data is the same across all votes.
To allow for this, we change the PrepareProposal API to the following

```rust
fn PrepareProposal(Block, UnbatchedHeader) -> (BlockData, Header)
```

where `UnbatchedHeader` essentially contains a "RawCommit", the `Header` contains a batch-optimized `commit` and an additional "Application Data" field in its root. This will involve a number of changes to core data structures, which will be gone over in the ADR.
The `Unbatched` header and `rawcommit` will never be broadcasted, they will be completely internal to consensus.

#### Inter-process communication (IPC) effects

For brevity in exposition above, we did not discuss the trade-offs that may occur in interprocess communication delays that these changs will introduce.
These new ABCI methods add more locations where the application must communicate with the consensus engine.
In most configurations, we expect that the consensus engine and the application will be either statically or dynamically linked, so all communication is a matter of at most adjusting the memory model the data is laid out within.
This memory model conversion is typically considered negligible, as delay here is measured on the order of microseconds at most, whereas we face millisecond delays due to cryptography and network overheads.
Thus we ignore the overhead in the case of linked libraries.

In the case where the consensus engine and the application are ran in separate processes, and thus communicate with a form of Inter-process communication (IPC), the delays can easily become on the order of milliseconds based upon the data sent. Thus its important to consider what's happening here.
We go through this phase by phase.

##### Prepare proposal IPC overhead

This requires a round of IPC communication, where both directions are quite large. Namely the proposer communicating an entire block to the application.
However, this can be mitigated by splitting up `PrepareProposal` into two distinct, async methods, one for the block IPC communication, and one for the Header IPC communication.

Then for chains where the block data does not depend on the header data, the block data IPC communication can proceed in parallel to the prior block's voting phase. (As a node can know whether or not its the leader in the next round)

Furthermore, this IPC communication is expected to be quite low relative to the amount of p2p gossip time it takes to send the block data around the network, so this is perhaps a premature concern until more sophisticated block gossip protocols are implemented.

##### Process Proposal IPC overhead

This phase changes the amount of time available for the consensus engine to deliver a block's data to the state machine.
Before, the block data for block N would be delivered to the state machine upon receiving a commit for block N and then be executed.
The state machine would respond after executing the txs and before prevoting.
The time for block delivery from the consensus engine to the state machine after this change is the time of receiving block proposal N to the to time precommit on proposal N.
It is expected that this difference is unimportant in practice, as this time is in parallel to one round of p2p communication for prevoting, which is expected to be significantly less than the time for the consensus engine to deliver a block to the state machine.

##### Vote Extension IPC overhead

This has a small amount of data, but does incur an IPC round trip delay. This IPC round trip delay is pretty negligible as compared the variance in vote gossip time. (the IPC delay is typically on the order of 10 microseconds)

## Status

Proposed

## Consequences

### Positive

- Enables a large number of new features for applications
- Supports both immediate and delayed execution models
- Allows application specific data from each validator
- Allows for batch optimizations across txs, and votes

### Negative

- This is a breaking change to all existing ABCI clients, however the application should be able to have a thin wrapper to replicate existing ABCI behavior.
    - PrepareProposal - can be a no-op
    - Process Proposal - has to cache the block, but can otherwise be a no-op
    - Vote Extensions - can be a no-op
    - Finalize Block - Can black-box call BeginBlock, DeliverTx, EndBlock given the cached block data

- Vote Extensions adds more complexity to core Tendermint Data Structures
- Allowing alternate alternate execution models will lead to a proliferation of new ways for applications to violate expected guarantees.

### Neutral

- IPC overhead considerations change, but mostly for the better

## References

Reference for IPC delay constants: <http://pages.cs.wisc.edu/~adityav/Evaluation_of_Inter_Process_Communication_Mechanisms.pdf>

### Short list of blocked features / scaling improvements with required ABCI++ Phases

| Feature | PrepareProposal | ProcessProposal | Vote Extensions |
| :---         |     :---:      |     :---:     |     :---:     |
| Tx based signature aggregation   | X |   |   |
| SNARK proof of valid state transition     | X |   |   |
| Validator provided authentication paths in stateless blockchains | X |   |   |
| Immediate Execution     |   | X |   |
| Simple soft forks     |   | X |   |
| Validator guaranteed IBC connection attempts     |   |   | X |
| Validator based price oracles     |   |   | X |
| Immediate Execution with increased time for block execution     | X | X | X |
| Threshold Encrypted txs     | X | X | X |
