# Event System

## Usage in the SDK

Events are an essential part of the Cosmos SDK. They are similar to "logs" in Ethereum and allow a blockchain
app to attach key-value pairs to a transaction that can later be used to search for it or extract some information
in human readable form. Events are not written to the application state, nor do they form part of the AppHash,
but mainly intended for client use (and become an essential API for any reactive app or app that searches for txs). 

In contrast, transactions also have a binary "data" field that is part of the AppHash (provable with light client proofs,
part of consensus). This data is not searchable, but given a tx hash, you can be guaranteed what the data returned is.
This is often empty, but sometimes custom protobuf formats to return essential information from an execution.

Every message in the SDK may add events to the EventManager and these are then added to the final ABCI result that is returned
to Tendermint. Events are exposed in 3 different ways over the Tendermint API (which is the only way a client can query).
First of all is the `events` field on the transaction result (when you query a transaction by hash, you can see all event emitted
by it). Secondly is the `log` field on the same transaction result. And third is the query interface to search or subscribe for
transactions. 

The `log` field actually has the best data. It contains an array of array of events. The first array is one entry per incoming message.
Transactions in the Cosmos SDK may consist of multiple messages that are executed atomically. Maybe we send tokens, then issue a swap
on a DEX. Each action would return it's own list of Events and in the logs, these are separated. For each message, it maintains a list
of Events, exactly in the order returned by the application. This is JSON encoded and can be parsed by a client. In fact this is
how [CosmJS](https://github.com/cosmos/cosmjs) gets the events it shows to the client.

In Tendermint 0.35, the `events` field will be one flattened list of events over all messages. Just as if we concatenated all
the per-message arrays contained in the `log` field. This fix was made as
[part of an event system refactoring](https://github.com/tendermint/tendermint/pull/6634). This refactoring is also giving us
[pluggable event indexing engines](https://github.com/tendermint/tendermint/pull/6411), so we can use eg. PostgreSQL to
store and query the events with more powerful indexes.

However, currently (until Tendermint 0.34 used in Cosmos SDK 0.40-0.43), all events of one transaction are "flat-mapped" on type. 
Meaning all events with type `wasm` get merged into one. This makes the API not very useful to understanding more complex events
currently. There are also a number of limitations of the power of queries in the search interface.

Given the state of affairs, and given that we seek to provide a stable API for contracts looking into the future, we consider the
`log` output and the Tendermint 0.35 event handling to be the standard that clients should adhere to. And we will expose a similar
API to the smart contracts internally (all events from the message appended, unmerged).
### Data Format

The event has a string type, and a list of attributes. Each of them being a key value pair. All of these maintain a
consistent order (and avoid dictionaries/hashes). Here is a simple Event in JSON:

```json
{ 
    "type": "wasm", 
    "attributes": [
        {"key": "_contract_address", "value": "cosmos1pkptre7fdkl6gfrzlesjjvhxhlc3r4gmmk8rs6"}, 
        {"key": "transfered", "value": "777000"}
    ]
}
```

And here is a sample log output for a transaction with one message, which emitted 2 events:

```json
[
    [
        { 
            "type": "message", 
            "attributes": [
                {"key": "module", "value": "bank"}, 
                {"key": "action", "value": "send"}
            ]
        },
        { 
            "type": "transfer", 
            "attributes": [
                {"key": "recipient", "value": "cosmos1pkptre7fdkl6gfrzlesjjvhxhlc3r4gmmk8rs6"}, 
                {"key": "amount", "value": "777000uatom"}
            ]
        }
    ]
]
```

### Default Events in the SDK

There are two places events that are emitted in every transaction regardless of the module which is executed.
[The first is `{"type": "message"}`](https://github.com/cosmos/cosmos-sdk/blob/6888de1d86026c25197c1227dae3d7da4d41a441/baseapp/baseapp.go#L746-L748)
defining an `action` attribute. This is emitted for each top-level (user-signed) message, but the action names have changed between
0.42 and 0.43. 

The other place is in the [signature verification AnteHandler](https://github.com/cosmos/cosmos-sdk/blob/v0.42.9/x/auth/ante/sigverify.go#L103-L120), where it emits information on the account sequences and signatures on the transaction.

These are all handled in BaseApp and the middleware *before* any module is called and thus not exposed to CosmWasm contracts at all.

### Standard Events in the SDK

The events that will actually make it to the contracts are the events that are emitted by the other modules / keepers. Let's look
at some good examples of what they look like:

The most basic one is `bank`, which emits two events on every send, a [custom "transfer" event](https://github.com/cosmos/cosmos-sdk/blob/v0.42.9/x/bank/keeper/send.go#L142-L147) as well as "sender" information under the [standard "message" type](https://github.com/cosmos/cosmos-sdk/blob/v0.42.9/x/bank/keeper/send.go#L148-L151). Replacing variables with string literals, they look like this:

```go
sdk.NewEvent(
    "transfer"
    sdk.NewAttribute("recipient", toAddr.String()),
    sdk.NewAttribute("sender", fromAddr.String()),
    sdk.NewAttribute("amount", amt.String()),  // eg 12456uatom
),
sdk.NewEvent(
    "message",
    sdk.NewAttribute("sender", fromAddr.String()),
),
```

The delegation module seems a bit more refined, emitting a generic "message" type event in [`msg_server.go`](https://github.com/cosmos/cosmos-sdk/blob/v0.42.9/x/distribution/keeper/msg_server.go#L42-L46) including the module name, **before** 
emitting some custom event types closer to the actual code logic in
[`keeper.go`](https://github.com/cosmos/cosmos-sdk/blob/v0.42.9/x/distribution/keeper/keeper.go#L74-L77).

This looks something like:

```go
sdk.NewEvent(
    "message",
    sdk.NewAttribute("module", "distribution"),
    sdk.NewAttribute("sender", msg.DelegatorAddress),
),
sdk.NewEvent(
    "set_withdraw_address",
    sdk.NewAttribute("withdraw_address", withdrawAddr.String()),
),
```

## Usage in wasmd

In `x/wasm` we also use Events system. On one hand, the Go implementation of `x/wasm` emits standard events for each 
message it processes, using the `distribution` module as an example. Furthermore, it allows contracts to
emit custom events based on their execution state, so they can for example say "dex swap, BTC-ATOM, in 0.23, out 512"
which require internal knowledge of the contract and is very useful for custom dApp UIs.

`x/wasm` is also a consumer of events, since when a smart contract executes a SubMsg and processes the reply, it receives
not only the `data` response from the message execution, but also the list of events. This makes it even more important for
us to document a standard event processing format.

### Standard Events in x/wasm

Following the model of `distribution`, we will split the emitted events into two parts. All calls to the message server, will receive
the following event:

```go
sdk.NewEvent(
    "message",
    sdk.NewAttribute("module", "wasm"),
    // Note: this was "signer" before 0.18
    sdk.NewAttribute("sender", msg.Sender),  
),
```

No further information will be added to the generic "message" type, but rather be contained in a more context-specific event type.
Here are some examples:

```go
// Store Code
sdk.NewEvent(
    "store_code",
    sdk.NewAttribute("code_id", fmt.Sprintf("%d", codeID)),
    // features required by the contract (new in 0.18)
    // see https://github.com/CosmWasm/wasmd/issues/574
    sdk.NewAttribute("feature", "stargate"),
    sdk.NewAttribute("feature", "staking"),
)

// Instantiate Contract
sdk.NewEvent(
    "instantiate",
    sdk.NewAttribute("code_id", fmt.Sprintf("%d", msg.CodeID)),
    sdk.NewAttribute("_contract_address", contractAddr.String()),
)

// Execute Contract
sdk.NewEvent(
    "execute",
    sdk.NewAttribute("_contract_address", contractAddr.String()),
)

// Migrate Contract
sdk.NewEvent(
    "migrate",
    // Note: this is the new code id that is being migrated to
    sdk.NewAttribute("code_id", fmt.Sprintf("%d", msg.CodeID)),
    sdk.NewAttribute("_contract_address", contractAddr.String()),
)

// Set new admin
sdk.NewEvent(
    "update_admin",
    sdk.NewAttribute("_contract_address", contractAddr.String()),
    sdk.NewAttribute("admin", msg.NewAdmin),
)

// Clear admin
sdk.NewEvent(
    "clear_admin",
    sdk.NewAttribute("_contract_address", contractAddr.String()),
)

// Pin Code
sdk.NewEvent(
    "pin_code",
    sdk.NewAttribute("code_id", strconv.FormatUint(msg.CodeID, 10)),
)

// Unpin Code
sdk.NewEvent(
    "unpin_code",
    sdk.NewAttribute("code_id", strconv.FormatUint(msg.CodeID, 10)),
)

// Emitted when processing a submessage reply
sdk.NewEvent(
    "reply",
    sdk.NewAttribute("_contract_address", contractAddr.String()),
    // If the submessage was successful, and reply is processing the success case
    sdk.NewAttribute("mode", "handle_success"),
    // If the submessage returned an error that was "caught" by the reply block
    sdk.NewAttribute("mode", "handle_failure"),
)

// Emitted when handling sudo
sdk.NewEvent(
    "sudo",
    sdk.NewAttribute("_contract_address", contractAddr.String()),
)
```

Note that every event that affects a contract (not store code, pin or unpin) will return the contract_address as
`_contract_address`. The events that are related to a particular wasm code (store code, instantiate, pin, unpin, and migrate)
will emit that as `code_id`. All attributes prefixed with `_` are reserved and may not be emitted by a smart contract,
so we use the underscore prefix consistently with attributes that may be injected into custom events.

### Emitted Custom Events from a Contract

When a CosmWasm contract returns a `Response` from one of the calls, it may return a list of attributes as well as a list
of events (in addition to data and a list of messages to dispatch). These are then processed in `x/wasm` to create events that
are emitted to the blockchain.

If the response contains a non-empty list of `attributes`, `x/wasm` will emit a `wasm` type event. This event will
always be tagged with `_contract_address` by the Go module, so this is trust-worthy. The contract itself cannot overwrite
this field. Beyond this, the `attributes` returned by the contract, these are appended to the same event.

A contact may also return custom `events`. These are multiple events, each with their own type as well as attributes.
When they are received, `x/wasm` prepends `wasm-` to the event type returned by the contact to avoid them trying to fake
an eg. `transfer` event from the bank module. The output here may look like:

```go
sdk.NewEvent(
    "wasm-promote"
    sdk.NewAttribute("_contract_address", contractAddr.String()),
    sdk.NewAttribute("batch_id", "6"),
    sdk.NewAttribute("address", "cosmos1234567"),
    sdk.NewAttribute("address", "cosmos1765432"),
),
sdk.NewEvent(
    "wasm-promote"
    sdk.NewAttribute("_contract_address", contractAddr.String()),
    sdk.NewAttribute("batch_id", "7"),
    sdk.NewAttribute("address", "cosmos19875632"),
)
```

Note that these custom events also have the `_contract_address` attribute automatically injected for easier attribution in the clients.
The multiple event API was designed to allow the contract to make logical groupings that are persisted in the event system,
more than flattening them all into one event like:

```go
sdk.NewEvent(
    "wasm"
    sdk.NewAttribute("_contract_address", contractAddr.String()),
    sdk.NewAttribute("action", "promote"),
    sdk.NewAttribute("batch_id", "6"),
    sdk.NewAttribute("address", "cosmos1234567"),
    sdk.NewAttribute("address", "cosmos1765432"),
    sdk.NewAttribute("batch_id", "7"),
    sdk.NewAttribute("address", "cosmos19875632"),
)
```

If the Response contains neither `event` nor `attributes`, not `wasm*` events will be emitted, just the standard `message`
type as well as the action-dependent event (like `execute` or `migrate`). This is a significant change from pre-0.18 versions
where one could count on the `wasm` event to always be emitted. Now it is recommended to search for `execute._contract_address="foo"`
to find all transactions related to the contract.

### Validation Rules

While the `wasm` and `wasm-*` namespacing does sandbox the smart contract events and limits malicious activity they could
undertake, we also perform a number of further validation checks on the contracts:

* No attribute key may start with `_`. This is currently used for `_contract_address` and is reserved for a 
  namespace for injecting more *trusted* attributes from the `x/wasm` module as opposed to the contract itself
* Event types are trimmed of whitespace, and must have at least two characters prior to prepending `wasm-`. If the contract returns
  "  hello\n", the event type will look like `wasm-hello`. If it emits "  a  ", this will be rejected with an error (aborting execution!)
* Attribute keys and values (both in `attributes` and under `events`) are trimmed of leading/trailing whitespace. If they are empty after
  trimming, they are rejected as above (aborting the execution). Otherwise, they are passed verbatim.

## Event Details for wasmd

Beyond the basic Event system and emitted events, we must handle more advanced cases in `x/wasm`
and thus add some more logic to the event processing. Remember that CosmWasm contracts dispatch other
messages themselves, so far from the flattened event structure, or even a list of list (separated
by message index in the tx), we actually have a tree of messages, each with their own events. And we must
flatten that in a meaningful way to feed it into the event system.

Furthermore, with the sub-message reply handlers, we end up with eg. "Contract A execute", "Contract B execute",
"Contract A reply". If we return all events by all of these, we may end up with many repeated event types and
a confusing results. However, we may use the standard "message" events to separate the sub-messages as it marks
where the next one starts. With careful analysis of the "sender" field on these "message" markers, we may be able
to reconstruct much of the tree execution path. We should ensure all this information is exposed in the most
consistent way possible.

### Combining Events from Sub-Messages

Each time a contract is executed, it not only returns the `message` event from its call, the `execute` event for the
contact and the `wasm` event with any custom fields from the contract itself. It will also return the same set of information
for all messages that it returned, which were later dispatched. The event system was really designed for one main
action emitting events, so we define a structure to flatten this event tree:

* We only emit one event of type `message`. This is the top-level call, just like the standard Go modules. For all
  dispatched submessages, we filter out this event type.
* All events are returned in execution order as [defined by CosmWasm docs](https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages)
* `x/wasm` keeper emits a custom event for each call to a contract entry point. Not just `execute`, `instantiate`,
  and `migrate`, but also `reply`, `sudo` and all ibc entry points.
* This means all `wasm*` events are preceeded by the cosmwasm entry point that returned them. 

To make this more clear, I will provide an example of executing a contract, which returns two messages, one to instantiate a new
contract and the other to set the withdrawl address, while also using `ReplyOnSuccess` for the instantiation (to get the
address). It will emit a series of events that looks something like this:

```go
/// original execution (top-level message is the only one that gets the message tag)
sdk.NewEvent(
    "message",
    sdk.NewAttribute("module", "wasm"),
    sdk.NewAttribute("sender", msg.Sender),  
),

// top-level exection call
sdk.NewEvent(
    "execute",
    sdk.NewAttribute("_contract_address", contractAddr.String()),
),
sdk.NewEvent(
    "wasm",
    sdk.NewAttribute("_contract_address", contractAddr.String()),
    sdk.NewAttribute("custom", "from contract"),
),

// instantiating contract (first dipatched message)
sdk.NewEvent(
    "instantiate",
    sdk.NewAttribute("code_id", fmt.Sprintf("%d", msg.CodeID)),
    sdk.NewAttribute("_contract_address", newContract.String()),
)
// didn't emit any attributes, but one event
sdk.NewEvent(
    "wasm-custom",
    sdk.NewAttribute("_contract_address", newContract.String()),
    sdk.NewAttribute("foobar", "baz"),
),

// handling the reply (this doesn't emit a message event as it never goes through the message server)
sdk.NewEvent(
    "reply",
    sdk.NewAttribute("_contract_address", contractAddr.String()),
    sdk.NewAttribute("mode", "handle_success"),
),
sdk.NewEvent(
    "wasm",
    sdk.NewAttribute("_contract_address", contractAddr.String()),
    sdk.NewAttribute("custom", "from contract"),
),

// calling the distribution module (second dispatched message)
sdk.NewEvent(
    "set_withdraw_address",
    sdk.NewAttribute("withdraw_address", withdrawAddr.String()),
),
```

### Exposing Events to Reply

When the `reply` clause in a contract is called, it will receive the data returned from the message it
applies to, as well as all events from that message. In the above case, when the `reply` function was called
on `contractAddr` in response to initializing a contact, it would get the binary-encoded `initData` in the `data`
field, and the following in the `events` field:

```go
sdk.NewEvent(
    "instantiate",
    sdk.NewAttribute("code_id", fmt.Sprintf("%d", msg.CodeID)),
    sdk.NewAttribute("_contract_address", newContract.String()),
)
sdk.NewEvent(
    "wasm-custom",
    sdk.NewAttribute("_contract_address", newContract.String()),
    sdk.NewAttribute("foobar", "baz"),
),
```

If the original contract execution example above was actually the result of a message returned by an eg. factory contract,
and it registered a ReplyOn clause, the `reply` function on that contract would receive the entire 11 events in the example
above, and would need to use the `message` markers to locate the segment of interest.

## Governance Events
The governance process is handled by the cosmos-sdk `gov` module. We do not emit any events of type "message" anymore in v0.18+.
Context-specific events are emitted as described above. `Execution` and `Migration` return some contract result though that are
emitted as:
```go
sdk.NewEvent(
    "gov_contract_result",
    sdk.NewAttribute("result", hex.EncodeToString(data)),
)
```

## IBC Events

All IBC entry points are only called by external accounts and not from contracts. They need to contain proofs of state of other blockchains and cannot be called by other contracts on the same chain. Therefore, the event emitted are not essential for cross-contract calls, and `x/wasm` does not emit custom events for these actions.

There are well-defined events emitted by the IBC base layer and are required for the relayer functionality. If you wish to subscribe to these, you can find them [defined in the `ibc-go` codebase](https://github.com/cosmos/ibc-go/blob/main/modules/core/04-channel/keeper/events.go).
