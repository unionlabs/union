- the wasm module's implementation details leak into the ibc interface
  - ClientState envelope leaks the contract information (code hash) and latest height <- TODO: figure out why
    - a mapping of client-id to code_hash could be stored internally
    - issue then is how do you create a client? the answer is changing the MsgCreateClient to take a client_type, and registering a wasm contract with a client type (just an arbitrary string)
      - potential issues:
        - register wasm code as client_type `foo`, then migrate the contract. new contract code would be registered under `foo`
          also see: 
  - ConsensusState and ClientMessage don't provide anything other than arbitrary wrappers that are downcasted to
    - ConsensusState: https://github.com/cosmos/ibc-go/blob/feat/wasm-clients/modules/light-clients/08-wasm/types/client_state.go#L108-L112
      - why is this on ClientState? and not part of the keeper(?)
    - ClientMessage: https://github.com/cosmos/ibc-go/blob/feat/wasm-clients/modules/light-clients/08-wasm/types/client_message.go
      - this is _just_ a wrapper around bytes, all it does is ensure the bytes are non-empty (which seems useless)

# Preamble

The current implementation of `08-wasm` light clients leaks implementation details into it's ibc interface, when it would ideally be a completely opaque wrapper around the wasm contracts.

The `ClientState` envelope leaks information specific to the contract (`code_hash`), since the wasm module doesn't store any sort of mapping between client id and contract address. It also stores `LatestHeight` directly, instead of querying the contract, requiring duplication of information in a client state (a tendermint light client in wasm would have the `LatestHeight` stored both in `tendermint.ClientState` and `wasm.ClientState`)

`08-wasm` clients are always of type `08-wasm`, which makes routing very difficult - we have resorted to embedding a client type directly into the wasm byte code and parsing the blob in our relayer.

`ConsensusState` and `ClientMessage` have been whittled down to just being thin envelopes around the data passed to the contract, serving no purpose other than routing the message to the `08-wasm` module.

# Proposal

Add a field `client_type` to `MsgCreateClient`, and make the states arbitrary bytes:

```go
type MsgCreateClient struct {
	ClientType string // <- new
	ClientState []byte
	ConsensusState []byte
	Signer string
}
```

<!-- TODO: Elaborate -->
This is used to route to the various light client modules, instead of downcasting types.

For the wasm module, add a `MsgRegisterClient` message:

```go
type MsgRegisterClient struct {
    CodeHash []bytes
    ClientType string
}
```

This will register a client type in a global registry, that is "pre-seeded" with native modules:

<!-- NOTE: I am not familiar with go so not 100% sure the syntax required here -->
```go
// pseudocode
clientTypeGlobalRegistry["07-tendermint"] = *TendermintModule
clientTypeGlobalRegistry["09-solomachine"] = *SolomachineModule
```

## How it works

The flow for creating client would be as follows:

```json
{
  "client_type": "07-tendermint",
  "client_state": "0x...",
  "consensus_state": "0x..." 
}
```

{
  "client_type": "07-tendermint",
  "client_state": {
    "@type": "ibc.lightclients.tendermint.v1.ClientState",
    "value": "0x..."
  }, 
  "consensus_state": {
    "@type": "ibc.lightclients.tendermint.v1.ConsensusState",
    "value": "0x..."
  } 
}

this msg is received by `core.keeper.Keeper`, which then inspects the client type by checking it's registry:

```go
// pseudocode
func (k Keeper) CreateClient(goCtx context.Context, msg *clienttypes.MsgCreateClient) (*clienttypes.MsgCreateClientResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

  lightClientModule := keeper.registeredClientTypes[msg.clientType]
  // this call will do the necessary validations on the states - for tendermint, deserializing
  // Any<ibc.tendermint.[...]>, for wasm simply passing the bytes through to the contract to validate itself
  lightClientModule.CreateClient(clientType, clientState, consensusState)

	return &clienttypes.MsgCreateClientResponse{}, nil
}
```

If the client type points to an existing handler, the handler would handle the msg however it likes. for `07-tendermint`, this would be as simple as keeping the exact same code that exists now - verifying that the values contained in the states are the correct `Any<ibc.lightclients.tendermint.v1.*State>` types, and `08-wasm` passing the bytes through to the contract directly.

### `08-wasm` specific flow

- upload code, with hash `0xabcd`

- register client type with said code:

  ```json
  {
    "client_type": "foobar",
    "code_hash": "0xabcd"
  }
  ```

  this will be sent to the 08-wasm module, which will then register the client_type "foobar" under the global client type registry module, pointing back to the 08-wasm module with the code hash.

- create a client with the new type:

  ```json
  {
    "client_type": "foobar",
    "client_state": "0x...", 
    "consensus_state": "0x..." 
  }
  ```

  the keeper will look up the handler for the client type "foobar", and pass the message to it. the looked up module is `(08-wasm, "0xabcd")`, which instructs the wasm module to pass the client & consensus state to the contract as arbitrary bytes. this will require the following change to 08-wasm/types/contract_api.go#InstantiateMessage:

  ```go
  type InstantiateMessage struct {
    // no longer a type implementing Client/ConsensusState, instead just arbitary bytes that will be verified by the called contract
  	ClientState    []byte `json:"client_state"`
  	ConsensusState []byte `json:"consensus_state"`
  }
  ```

  the called contract will then do it's thing for instantiate (as specified in https://github.com/cosmos/ibc-go/issues/3956)

## Upgrading contracts

When upgrading a contract (https://github.com/cosmos/ibc-go/issues/3956), the existing mapping of clientType => codeHash will be updated to point to the new codeHash.

# Backwards Compatability 

Since this proposal break backwards compatabiility with existing light client implementations and relayers by changing the existing `MsgCreateClient` message, we propose deprecating `ibc.core.client.v1.MsgCreateClient` and create a new msg as follows: 

```protobuf
message MsgCreateClient {
  option (cosmos.msg.v1.signer) = "signer";

  option (gogoproto.goproto_getters) = false;

  // add this field
  string client_type = 1;

  // these are just bytes, that are decoded by the module being routed to
  bytes client_state = 2;
  bytes consensus_state = 3;

  // signer address
  string signer = 4;
}
```

This would allow for keeping the same interface for existing native light clients (using `ibc.core.client.v1.MsgCreateClient`), but without support for `08-wasm` clients - instead, introduce the above message as `ibc.core.client.v2.MsgCreateClient` that supports both native and non-native light clients via the routing system described above. Given that 08-wasm is still incomplete, this is the perfect time to make this change - the v1 messages could easily be routed to the v2 handler, simply by constructing the v2 message from the the fields in v1.
