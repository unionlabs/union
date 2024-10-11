---
sidebar_position: 1
---

# Transaction Lifecycle

:::note Synopsis
This document describes the lifecycle of a transaction from creation to committed state changes. Transaction definition is described in a [different doc](../advanced/01-transactions.md). The transaction is referred to as `Tx`.
:::

:::note Pre-requisite Readings

* [Anatomy of a Cosmos SDK Application](./00-app-anatomy.md)
:::

## Transaction Creation

One of the main application interfaces is the command-line interface. The transaction `Tx` can be created by the user inputting a command in the following format from the [command-line](../advanced/07-cli.md), providing the type of transaction in `[command]`, arguments in `[args]`, and configurations such as gas prices in `[flags]`:

```bash
[appname] tx [command] [args] [flags]
```

This command automatically **creates** the transaction, **signs** it using the account's private key, and **broadcasts** it to the specified peer node.

There are several required and optional flags for transaction creation. The `--from` flag specifies which [account](./03-accounts.md) the transaction is originating from. For example, if the transaction is sending coins, the funds are drawn from the specified `from` address.

### Gas and Fees

Additionally, there are several [flags](../advanced/07-cli.md) users can use to indicate how much they are willing to pay in [fees](./04-gas-fees.md):

* `--gas` refers to how much [gas](./04-gas-fees.md), which represents computational resources, `Tx` consumes. Gas is dependent on the transaction and is not precisely calculated until execution, but can be estimated by providing `auto` as the value for `--gas`.
* `--gas-adjustment` (optional) can be used to scale `gas` up in order to avoid underestimating. For example, users can specify their gas adjustment as 1.5 to use 1.5 times the estimated gas.
* `--gas-prices` specifies how much the user is willing to pay per unit of gas, which can be one or multiple denominations of tokens. For example, `--gas-prices=0.025uatom, 0.025upho` means the user is willing to pay 0.025uatom AND 0.025upho per unit of gas.
* `--fees` specifies how much in fees the user is willing to pay in total.
* `--timeout-height` specifies a block timeout height to prevent the tx from being committed past a certain height.

The ultimate value of the fees paid is equal to the gas multiplied by the gas prices. In other words, `fees = ceil(gas * gasPrices)`. Thus, since fees can be calculated using gas prices and vice versa, the users specify only one of the two.

Later, validators decide whether or not to include the transaction in their block by comparing the given or calculated `gas-prices` to their local `min-gas-prices`. `Tx` is rejected if its `gas-prices` is not high enough, so users are incentivized to pay more.

### CLI Example

Users of the application `app` can enter the following command into their CLI to generate a transaction to send 1000uatom from a `senderAddress` to a `recipientAddress`. The command specifies how much gas they are willing to pay: an automatic estimate scaled up by 1.5 times, with a gas price of 0.025uatom per unit gas.

```bash
appd tx send <recipientAddress> 1000uatom --from <senderAddress> --gas auto --gas-adjustment 1.5 --gas-prices 0.025uatom
```

### Other Transaction Creation Methods

The command-line is an easy way to interact with an application, but `Tx` can also be created using a [gRPC or REST interface](../advanced/06-grpc_rest.md) or some other entry point defined by the application developer. From the user's perspective, the interaction depends on the web interface or wallet they are using (e.g. creating `Tx` using [Keplr](https://www.keplr.app/) and signing it with a Ledger Nano S).

## Transaction Broadcasting

This is the next phase, where a transaction is sent from a client (such as a wallet or a command-line interface) to the network of nodes. This process is consensus-agnostic, meaning it can work with various consensus engines.

Below are the steps involved in transaction broadcasting:

1. **Transaction Creation and Signing:**
Transactions are created and signed using the client's private key to ensure authenticity and integrity.

2. **Broadcasting to the Network:**
The signed transaction is sent to the network. This is handled by the `BroadcastTx` function in the client context.

3. **Network Propagation:**
Once received by a node, the transaction is propagated to other nodes in the network. This ensures that all nodes have a copy of the transaction.

4. **Consensus Engine Interaction:**
The specific method of broadcasting may vary depending on the consensus engine used. The SDK's design allows for easy integration with any consensus engine by configuring the `clientCtx` appropriately.

The function `BroadcastTx` in `client/tx/tx.go` demonstrates how a transaction is prepared, signed, and broadcasted. Here's the relevant part of the function that handles the broadcasting:

```go
res, err := clientCtx.BroadcastTx(txBytes)
if err != nil {
 return err
}
```

**Configuration:**

To adapt this function for different consensus engines, ensure that the `clientCtx` is configured with the correct network settings and transaction handling mechanisms for your chosen engine. This might involve setting up specific encoders, decoders, and network endpoints that are compatible with the engine.

## Transaction Processing 

After a transaction is broadcasted to the network, it undergoes several processing steps to ensure its validity. These steps are managed by the application's core transaction processing layer, which is responsible for handling transactions. Within the SDK, this core layer is wrapped with a runtime layer defined in `runtime/app.go`. This layer extends the core functionality with additional features to handle transactions, allowing for more flexibility and customization, such as the ability to swap out different consensus engines. The key steps in transaction processing are:

### Decoding

The transaction is decoded from its binary format into a structured format that the application can understand.

* **During Transaction Processing:** Transactions are received in the encoded `[]byte` form. Nodes first unmarshal the transaction using the configuration defined in the app, then proceed to execute the transaction, which includes state changes.

### Routing

How Routing Works:
The transaction is routed to the appropriate module based on the message type. Each message type is associated with a specific module, which is responsible for processing the message. The core transaction processing layer uses a `MsgServiceRouter` to direct the transaction to the correct module.

1. **Transaction Type Identification:** 
Each transaction contains one or more messages (`sdk.Msg`), and each message has a `Type()` method that identifies its type. This type is used to determine the appropriate module to handle the message.
2. **Module Routing:** 
The core transaction processing layer holds a `MsgServiceRouter` which maps each message type to a specific module's handler. When a transaction is processed, this router is used to direct the message to the correct module.

### Example of Routing

Let's say there is a transaction that involves transferring tokens. The message type might be `MsgSend`, and the `MsgServiceRouter` in `BaseApp` would route this message to the bank module's handler. The bank module would then validate the transaction details (like sender balance) and update the state to reflect the transfer if valid...

### Validation

Preliminary checks are performed. These include signature verification to ensure the transaction hasn't been tampered with and checking if the transaction meets the minimum fee requirements, which is handled by the `AnteHandler`. The `Antehandler` is invoked during the `runTx` method in `BaseApp`.

#### Types of Transaction Checks

During the transaction lifecycle, full-nodes perform a series of checks to validate transactions before they are finalized in a block. These checks are categorized into stateless and stateful checks.

**Stateless Checks**:
Stateless checks are validations that do not require access to the state of the blockchain. They are computationally inexpensive and can be performed by light clients or offline nodes. Examples include:

* Ensuring addresses are not empty.
* Enforcing nonnegative values for transaction fields.
* Validating the format of the data in the transaction.

**Stateful Checks**:
Stateful checks involve validating transactions against the current committed state of the blockchain. These checks are more computationally intensive as they require access to the state. Examples include:

* Verifying that the account has sufficient funds.
* Checking that the sender has the necessary permissions for the transaction.
* Ensuring that the transaction does not result in any state conflicts.

Full-nodes use these checks during the validation process to quickly reject invalid transactions, minimizing wasted computational resources. Further validation occurs during the transaction execution phase, where transactions are fully executed.

#### ValidateBasic (deprecated)

* Messages ([`sdk.Msg`](../advanced/01-transactions.md#messages)) are extracted from transactions (`Tx`). The `ValidateBasic` method of the `sdk.Msg` interface implemented by the module developer is run for each transaction.
* To discard obviously invalid messages, the `BaseApp` type calls the `ValidateBasic` method very early in the processing of the message in the [`CheckTx`](../advanced/00-baseapp.md#checktx) and [`DeliverTx`](../advanced/00-baseapp.md#delivertx) transactions.
`ValidateBasic` can include only **stateless** checks (the checks that do not require access to the state). 

:::warning
The `ValidateBasic` method on messages has been deprecated in favor of validating messages directly in their respective [`Msg` services](../../build/building-modules/03-msg-services.md#Validation).

Read [RFC 001](https://docs.cosmos.network/main/rfc/rfc-001-tx-validation) for more details.
:::

### Discard or Addition to Mempool

If at any point during the initial transaction validation the transaction (`Tx`) fails, it is discarded, and the transaction lifecycle ends there. Otherwise, if it passes this preliminary check successfully, the general protocol is to relay it to peer nodes and add it to the node's transaction pool (often referred to as the mempool). This makes the `Tx` a candidate for inclusion in the next block, pending further consensus processes.

The **app-side mempool**, serves the purpose of keeping track of transactions seen by all full-nodes. Full-nodes maintain a **mempool cache** of the last `mempool.cache_size` transactions they have seen, serving as a first line of defense to prevent replay attacks. Ideally, `mempool.cache_size` should be large enough to encompass all transactions in the full mempool. If the mempool cache is too small to track all transactions, the initial transaction validation process is responsible for identifying and rejecting replayed transactions.

Currently existing preventative measures include fees and a `sequence` (nonce) counter to distinguish replayed transactions from identical but valid ones. If an attacker tries to spam nodes with many copies of a `Tx`, full-nodes maintaining a transaction cache reject all identical copies. Even if the copies have incremented sequence numbers, attackers are disincentivized by the need to pay fees.

Validator nodes maintain a transaction pool to prevent replay attacks, similar to full-nodes, but also use it to hold unconfirmed transactions in preparation for block inclusion. It's important to note that even if a `Tx` passes all preliminary checks, it can still be found invalid later on, as these initial checks do not fully execute the transaction's logic.

### Module Execution

After the transaction has been appropriately routed to the correct module by the `MsgServiceRouter` and passed all necessary validations, the execution phase begins:

* **Handler Activation**: Each module's handler processes the routed message, applying the necessary business logic such as updating account balances or transferring tokens.
* **State Changes**: Handlers may modify the state as required by the business logic, which could involve writing to the module's portion of the state store. This can be seen in the next subsection.
* **Event Emission and Logging**: During execution, modules can emit events and log information, which are crucial for monitoring and querying transaction outcomes.

For messages that adhere to older standards or specific formats, a routing function retrieves the route name from the message, identifying the corresponding module. The message is then processed by the designated handler within that module, ensuring accurate and consistent application of the transaction's logic.

4. During the execution, the module's handler will modify the state as required by the business logic. This could involve writing to the module's portion of the state store.

5. Modules can emit events and log information during execution, which are used for monitoring and querying transaction outcomes.

During the module execution phase, each message that has been routed to the appropriate module is processed according to the module-specific business logic. For example, the `handleMsgSend` function in the bank module processes `MsgSend` messages by checking balances, transferring tokens, and emitting events:

```go 
func handleMsgSend(ctx sdk.Context, keeper BankKeeper, msg MsgSend) error {
    if keeper.GetBalance(ctx, msg.Sender).Amount < msg.Amount {
        return sdkerrors.Wrap(sdkerrors.ErrInsufficientFunds, "sender does not have enough tokens")
    }
    keeper.SendCoins(ctx, msg.Sender, msg.Receiver, msg.Amount)
    ctx.EventManager().EmitEvent(
        sdk.NewEvent("transfer", sdk.NewAttribute("from", msg.Sender.String()), sdk.NewAttribute("to", msg.Receiver.String()), sdk.NewAttribute("amount", msg.Amount.String())),
    )
    return nil
}
```

This function exemplifies how a module's handler executes the transaction logic, modifies the state, and logs the transaction events, which are essential aspects of module execution.

### State Changes During Consensus

Before finalizing the transactions within a block, full-nodes perform a second round of checks using `validateBasicMsgs` and `AnteHandler`. This is crucial to ensure that all transactions are valid, especially since a malicious proposer might include invalid transactions. Unlike the checks during the transaction addition to the Mempool, the `AnteHandler` in this phase does not compare the transaction's `gas-prices` to the node's `min-gas-prices`. This is because `min-gas-prices` can vary between nodes, and using them here would lead to nondeterministic results across the network.

* After module execution, the transactions are included in a block proposal by the proposer.

* All full-nodes that receive this block proposal execute the transactions to ensure that the state changes are applied consistently across all nodes, maintaining the deterministic nature of the blockchain. This includes the execution of initial, transaction-specific, and finalizing operations.

## Inclusion in a Block

Consensus is the process through which nodes in a blockchain network agree on which transactions to include in the blockchain. This process typically occurs in rounds, starting with a designated node (often called a proposer) compiling a block from transactions in its transaction pool (mempool). The block is then proposed to other nodes (validators) in the network.

Each validator independently verifies the proposed block against the blockchain's rules. If the block is accepted by a sufficient number of validators according to the network's consensus rules, it is added to the blockchain. If not, the process may repeat, potentially with a different proposer or even resulting in a block that contains no transactions (a nil block).

The specific mechanisms of choosing a proposer, the criteria for a valid block, and the method of achieving agreement among validators can vary depending on the consensus algorithm used by the blockchain.


## Post-Transaction Handling

After execution, any additional actions that need to be taken are processed. This could include updating logs, sending events, or handling errors.

These steps are managed by `BaseApp` in the Cosmos SDK, which routes transactions to the appropriate handlers and manages state transitions.

After a transaction is executed in the Cosmos SDK, several steps are taken to finalise the process:

1. Event Emission: Modules emit events that can be used for logging, monitoring, or triggering other workflows. These events are collected during the transaction execution.

2. Logging: Information about the transaction execution, such as success or failure, and any significant state changes, are logged for audit and diagnostic purposes.

3. Error Handling: If any errors occur during transaction execution, they are handled appropriately, which may include rolling back certain operations to maintain state consistency.

4. State Commitment: Changes made to the state during the transaction are finalised and written to the blockchain. This step is crucial as it ensures that all state transitions are permanently recorded.

5. PostHandlers: After the execution of the message, `PostHandlers` are run. If they fail, the state changes made during `runMsgs` and by the `PostHandlers` themselves are both reverted. This ensures that only successful transactions affect the state.


After post-transaction handling, the exact sequence of the transaction lifecycle is dependent on the consensus mechanism used. This includes how transactions are grouped into blocks, how blocks are validated, and how consensus is achieved among validators to commit the block to the blockchain. Each consensus protocol may implement these steps differently to ensure network agreement and maintain the integrity of the blockchain state.

## Learn More

For a deeper dive into the underlying mechanisms of transaction processing and block commitment in the Cosmos SDK, consider exploring the [BaseApp documentation](../advanced/00-baseapp.md). This advanced documentation provides detailed insights into the internal workings and state management of the Cosmos SDK.

