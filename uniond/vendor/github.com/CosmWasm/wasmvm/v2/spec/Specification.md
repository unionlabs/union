# Specification

This section attempts to codify the [architecture](./Architecture.md) with a number of concrete
implementation details, function signatures, naming choices, etc.

## Definitions

**Contract** is as some wasm code uploaded to the system, initialized at the creation of the contract. This has no state except that which is contained in the wasm code (eg. static constants)

**Instance** is one instantiation of the contract. This contains a reference to the contract, as well as some "local" state to this instance, initialized at the creation of the instance. This state is stored in the kvstore, meaning a reference to the code plus a reference to the (prefixed) data store uniquely defines the smart contract.

Example: we could upload a generic "ERC20 mintable" contract, and many people could create independent instances of the same bytecode, where the local data defines the token name, the issuer, the max issuance, etc.

- First you **create** a _contract_
- Then you **instantiate** an _instance_
- Finally users **invoke** the _instance_

_Contracts_ are immutible (code/logic is fixed), but _instances_ are mutible (state changes)

## Serialization Format

There are two pieces of data that must be considered here. **Message Data**, which is arbitrary binary data passed in the transaction by the end user signing it, and **Context Data**, which is passed in by the cosmos sdk runtime, providing some guaranteed context. Context data may include the signer's address, the instance's address, number of tokens sent, block height, and any other information a contract may need to control the internal logic.

**Message Data** comes from a binary transaction and must be serialized. The most standard and flexible codec is (unfortunately) JSON. This allows the contract to define any schema it wants, and the client can easily provide the proper data. We recommend using a `string` field in the `InvokeMsg`, to contain the user-defined _message data_.

**Contact Data** comes from the go runtime and can either be serialized by sdk and deserialized by the contract, or we can try to do some ffi magic and use the same memory layout for the struct in Go and Wasm and avoid any serialization overhead. Note that the context data struct will be well-defined at compile time and guaranteed not to change between invocations (the same cannot be said for _message data_).

In spite of possible performance gains or compiler guarantees with C-types, I would recommend using JSON for this as well. Or another well-defined binary format, like protobuf. However, I will document some links below for those who would like to research the shared struct approach.

- [repr( c )](https://doc.rust-lang.org/nomicon/other-reprs.html) is a rust directive to produce cannonical C-style memory layouts. This is typically used in FFI (which wasm calls are).
- [wasm-ffi](https://github.com/DeMille/wasm-ffi) demos how to pass structs between wasm/rust and javascript painlessly. Not into golang, but it provides a nice explanation and design overview.
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen/) also tries to convert types and you can [read some success and limitations of the approach](https://github.com/rustwasm/wasm-bindgen/issues/111)
- [cgo](https://golang.org/cmd/cgo/#hdr-Go_references_to_C) has some documentation about accessing C structs from Go (which is what we get with the repr( c ) directive)

In short, go/cgo doesn't handle c-types very transparently, and these also don't support references to heap allocated data (eg. strings). All we get is a small performance gain for a lot of headaches... let's stick with json.

## State Access

**Instance State** is accessible only by one instance of the contract, with full read-write access. This can contain either a singleton (one key - simple contract or config) or a kvstore (subspace with many keys that can be accessed - like erc20 instance holding balances for many accounts). Sometimes the contract may want one or the other or even both (config + much data) access modes.

We can set the instance state upon _instantiation_. We can read and modify it upon _invocation_. This is a unique "prefixed db" subspace that can only be accessed by this instance. The read-only contract state should suffice for shared data between all instances. (Discuss this design in light of all use cases)

**Instance Account** is the sdk account controlled by this isntance. We pass in the address of the account, as well as it's current balance along with every invocation of the contract. This allows the contract to be somewhat self-aware of the external environment for the most common cases (eg. if it needs to release funds).

## Function Definitions

As discussed above, all data structures passed between web assembly and the cosmos-sdk will be sent in their JSON representation. For simplicity, I will show them as Go structs in this section, but only the json representation is used.

The actual call to create a new contract (upload code) is quite simple, and returns a `ContractID` to be used in all future calls:
`Create(contract WasmCode) (ContractID, error)`

Both Instantiating a contract, as well as invoking a contract (`Execute` method) have similar interfaces. The difference is that `Instantiate` requires the `store` to be empty, while `Execute` requires it to be non-empty:

- `Instantiate(contract ContractID, params Params, userMsg []byte, store KVStore, gasLimit int64) (res *Result, err error)`
- `Execute(contract ContractID, params Params, userMsg []byte, store KVStore, gasLimit int64) (res *Result, err error)`

We also expose a Query method to respond to abci.QueryRequests:

- `Query(contract ContractID, path []byte, data []byte, store KVStore, gasLimit int64) ([]byte, error)`

Here we pass in the remainder of the path (after directing to the contract) as well as a user-defined (json?) data from the query. We pass in the instances KVStore as above, and a gasLimit, as the computation takes some time. There is no gas for queries, but we should define some reasonable limit here to avoid any DoS vectors, such as a uploading a contract with an infinite loop in the query handler. QueryResult is JSON-encoded data in whatever format the contract decides.

Note that no `InstanceID` is ever used. The reason being is that the code and the data define the entire instance state. The calling logic is responsible for prefixing the `KVStore` with the instance-specific prefix and passing the proper `ContractInfo` in the parameters. This `InstanceID` is managed on the SDK side, but not exposed over the general interface to the Wasm engine.

### Parameters

This Read-Only info is available to every contract:

```go
// Params defines the state of the blockchain environment this contract is
// running in. This must contain only trusted data - nothing from the Tx itself
// that has not been verfied (like Signer).
//
// Params are json encoded to a byte slice before passing to the wasm contract.
type Params struct {
	Block    BlockInfo    `json:"block"`
	Message  MessageInfo  `json:"message"`
	Contract ContractInfo `json:"contract"`
}

type BlockInfo struct {
	// block height this transaction is executed
	Height uint64 `json:"height"`
	// time in nanoseconds since unix epoch. Uses Uint64 to ensure JavaScript compatibility.
	Time    Uint64 `json:"time"`
	ChainID string `json:"chain_id"`
}

type MessageInfo struct {
	// binary encoding of sdk.AccAddress executing the contract
	Sender HumanAddress `json:"sender"`
	// amount of funds send to the contract along with this message
	Funds Coins `json:"funds"`
}

type ContractInfo struct {
    // sdk.AccAddress of the contract, to be used when sending messages
    Address string       `json:"address"`
    // current balance of the account controlled by the contract
	Balance []Coin `json:"send_amount"`
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Denom  string `json:"denom"`  // string encoing of decimal value, eg. "12.3456"
	Amount string `json:"amount"`  // type, eg. "ATOM"
}
```

### Results

This is the information the contract can return:

```go
// Result defines the return value on a successful
type Result struct {
	// Messages comes directly from the contract and is it's request for action
	Messages []CosmosMsg `json:"msgs"`
	// base64-encoded bytes to return as ABCI.Data field
	Data string
	// attributes for a log event to return over abci interface
	Attributes []EventAttribute `json:"attributes"`
}
```

`CosmosMsg` is defined in the next section.

Note: I intentionally redefine a number of core types, rather than importing them from sdk/types. This is to guarantee immutibility. These types will be passed to and from the contract, and the contract adapter code (in go) can convert them to the go types used in the rest of the app. But these are decoupled, so they can remain constant while other parts of the sdk evolve.

I also consider adding Events to the return Result, but will delay that until there is a clear spec for how to use them

## Dispatched Messages

`CosmosMsg` is an abstraction of allowed message types that is designed to be consistent in spite of any changes to the underlying SDK. The "contract" module will maintain an adapter between these well-defined types and the current sdk implementation.

The following are allowed types for `CosmosMsg` return values. To be expanded later:

```go
// CosmosMsg is an rust enum and only (exactly) one of the fields should be set
// Should we do a cleaner approach in Go? (type/data?)
type CosmosMsg struct {
	Send SendMsg `json:"send"`
	Contract ContractMsg `json:"contract"`
	Opaque OpaqueMsg `json:"opaque"`
}

// SendMsg contains instructions for a Cosmos-SDK/SendMsg
// It has a fixed interface here and should be converted into the proper SDK format before dispatching
type SendMsg struct {
	ToAddress   string `json:"to_address"`
	Amount      []Coin `json:"amount"`
}

// ContractMsg is used to call another defined contract on this chain.
// The calling contract requires the callee to be defined beforehand,
// and the address should have been defined in initialization.
// And we assume the developer tested the ABIs and coded them together.
//
// Since a contract is immutable once it is deployed, we don't need to transform this.
// If it was properly coded and worked once, it will continue to work throughout upgrades.
type ContractMsg struct {
    // ContractAddr is the sdk.AccAddress of the contract, which uniquely defines
    // the contract ID and instance ID. The sdk module should maintain a reverse lookup table.
    ContractAddr string `json:"contract_addr"`
    // Msg is assumed to be a json-encoded message, which will be passed directly
    // as `userMsg` when calling `Execute` on the above-defined contract
    Msg string `json:"msg"`
}


// OpaqueMsg is some raw sdk-transaction that is passed in from a user and then relayed
// by the contract under some given conditions. These should never be created or
// inspected by the contract, but allows to build eg. multisig, governance in a contract
// and allow the end users to make use of all sdk functionality.
//
// An example is submitting a proposal for a vote. This is assumed to be correct (from the user)
// and if the contract determines the vote passed, the contract can then re-send it. If the chain
// updates, the client can submit a new proposal in the new format. Since this never comes from the
// contract itself, we don't need to worry about upgrading.
type OpaqueMsg struct {
	// Data is a custom msg that the sdk knows.
	// Generally the base64-encoded of go-amino binary encoding of an sdk.Msg implementation.
	// This should never be created by the contract, but allows for blindly passing through
	// temporary data.
	Data string `json:"data"`
}
```

## Exposed imports

### Local Storage

We expose a (sandboxed) `KVStore` to the contract that it can read and write to as it desires (with gas limits and perhaps absolute storage limits). This is then translated into a C struct and passed into rust to be adapted to the wasm contract. But in essence we expose the following:

```go
type KVStore interface {
	Get(key []byte) []byte
	Set(key, value []byte)
}
```

If desired, we can add an Iterate method, but that adds yet another level of complexity, a method that returns yet another object with custom callbacks. And then ensuring proper cleanup.

### Querying Other Modules

We also pass in a callback to the smart contracts to make some well-defined queries

```go
Query(query QueryRequest) (QueryModels, error)
```

Both of these are enums (interfaces) and there is a clear 1-to-1 relation between QueryRequest type to QueryModel type. We can not make any arbitrary queries, but only those well-specified below.

## Well-defined Queries

Here are request-model pairs that we can use in queries:

```go
// QueryRequest is an enum. Exactly one field should be non-empty
type QueryRequest struct {
	Account AccountRequest `json:"account"`
}

// QueryModels is an enum. Exactly one field should be non-empty: the same one corresponding to the Request
type QueryModels struct {
	Account []AccountModel `json:"account"`
}
```

**Account**

```go
// AccountRequest asks to read the state of a given account
type AccountRequest struct {
	// bech32 encoded sdk.AccAddres
	Address string `json:"address"`
}

// AccountModel is a basic description of an account
// (more fields may be added later, but none should change)
type AccountModel struct {
	Address string `json:"address"`
	Balance []Coin `json:"balance"`
	// pubkey may be empty
	PubKey struct {
		// hex-encoded bytes of the raw public key
		Data string `json:"data"`
		// the algorithm of the pubkey, currently "secp256k1", possibly others in the future
		Algo string `json:"algo"`
	} `json:"pub_key"`
}
```
