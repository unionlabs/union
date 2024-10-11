# RFC 001: Transaction Validation

## Changelog

* 2023-03-12: Proposed

## Background

Transaction Validation is crucial to a functioning state machine. Within the Cosmos SDK there are two validation flows, one is outside the message server and the other within. The flow outside of the message server is the `ValidateBasic` function. It is called in the antehandler on both `CheckTx` and `DeliverTx`. There is an overhead and sometimes duplication of validation within these two flows. This extra validation provides an additional check before entering the mempool.

With the deprecation of [`GetSigners`](https://github.com/cosmos/cosmos-sdk/issues/11275) we have the optionality to remove [sdk.Msg](https://github.com/cosmos/cosmos-sdk/blob/16a5404f8e00ddcf8857c8a55dca2f7c109c29bc/types/tx_msg.go#L16) and the `ValidateBasic` function. 

With the separation of CometBFT and Cosmos-SDK, there is a lack of control of what transactions get broadcasted and included in a block. This extra validation in the antehandler is meant to help in this case. In most cases the transaction is or should be simulated against a node for validation. With this flow transactions will be treated the same. 

## Proposal

The acceptance of this RFC would move validation within `ValidateBasic` to the message server in modules, update tutorials and docs to remove mention of using `ValidateBasic` in favour of handling all validation for a message where it is executed.

We can and will still support the `Validatebasic` function for users and provide an extension interface of the function once `sdk.Msg` is deprecated. 

> Note: This is how messages are handled in VMs like Ethereum and CosmWasm. 

### Consequences

The consequence of updating the transaction flow is that transaction that may have failed before with the `ValidateBasic` flow will now be included in a block and fees charged. 
