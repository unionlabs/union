# Introduction

## What is ABCI?

ABCI, Application Blockchain Interface is the interface between CometBFT and the application. More information about ABCI can be found in the specs [here](https://docs.cometbft.com/v1.0/spec/abci/). Within the release of ABCI 2.0 for the 0.38 CometBFT release there were additional methods introduced.

The 5 methods introduced during ABCI 2.0 (compared to ABCI v0) are:

* `PrepareProposal`
* `ProcessProposal`
* `ExtendVote`
* `VerifyVoteExtension`
* `FinalizeBlock`


## The Flow

## PrepareProposal

Based on validator voting power, CometBFT chooses a block proposer and calls `PrepareProposal` on the block proposer's application (Cosmos SDK). The selected block proposer is responsible for collecting outstanding transactions from the mempool, adhering to the application's specifications. The application can enforce custom transaction ordering and incorporate additional transactions, potentially generated from vote extensions in the previous block.

To perform this manipulation on the application side, a custom handler must be implemented. By default, the Cosmos SDK provides `PrepareProposalHandler`, used in conjunction with an application specific mempool. A custom handler can be written by application developer, if a noop handler provided, all transactions are considered valid.

Please note that vote extensions will only be available on the following height in which vote extensions are enabled. More information about vote extensions can be found [here](https://docs.cosmos.network/main/build/abci/vote-extensions).

After creating the proposal, the proposer returns it to CometBFT.

PrepareProposal CAN be non-deterministic.

## ProcessProposal

This method allows validators to perform application-specific checks on the block proposal and is called on all validators. This is an important step in the consensus process, as it ensures that the block is valid and meets the requirements of the application. For example, validators could check that the block contains all the required transactions or that the block does not create any invalid state transitions.

The implementation of `ProcessProposal` MUST be deterministic.

## ExtendVote and VerifyVoteExtensions

These methods allow applications to extend the voting process by requiring validators to perform additional actions beyond simply validating blocks.

If vote extensions are enabled, `ExtendVote` will be called on every validator and each one will return its vote extension which is in practice a bunch of bytes. As mentioned above this data (vote extension) can only be retrieved in the next block height during `PrepareProposal`. Additionally, this data can be arbitrary, but in the provided tutorials, it serves as an oracle or proof of transactions in the mempool. Essentially, vote extensions are processed and injected as transactions. Examples of use-cases for vote extensions include prices for a price oracle or encryption shares for an encrypted transaction mempool. `ExtendVote` CAN be non-deterministic.

`VerifyVoteExtensions` is performed on every validator multiple times in order to verify other validators' vote extensions. This check is submitted to validate the integrity and validity of the vote extensions preventing malicious or invalid vote extensions.

Additionally, applications must keep the vote extension data concise as it can degrade the performance of their chain, see testing results [here](https://docs.cometbft.com/v1.0/references/qa/cometbft-qa-38#vote-extensions-testbed).

`VerifyVoteExtensions` MUST be deterministic.


## FinalizeBlock

`FinalizeBlock` is then called and is responsible for updating the state of the blockchain and making the block available to users.
