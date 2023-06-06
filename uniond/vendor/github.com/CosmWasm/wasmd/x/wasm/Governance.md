# Governance

This document gives an overview of how the various governance
proposals interact with the CosmWasm contract lifecycle. It is
a high-level, technical introduction meant to provide context before
looking into the code, or constructing proposals. 

## Proposal Types
We have added 9 new wasm specific proposal types that cover the contract's live cycle and authorization:
 
* `StoreCodeProposal` - upload a wasm binary
* `InstantiateContractProposal` - instantiate a wasm contract
* `MigrateContractProposal` - migrate a wasm contract to a new code version
* `SudoContractProposal` - call into the protected `sudo` entry point of a contract
* `ExecuteContractProposal` - execute a wasm contract as an arbitrary user
* `UpdateAdminProposal` - set a new admin for a contract
* `ClearAdminProposal` - clear admin for a contract to prevent further migrations
* `PinCodes` - pin the given code ids in cache. This trades memory for reduced startup time and lowers gas cost
* `UnpinCodes` - unpin the given code ids from the cache. This frees up memory and returns to standard speed and gas cost
* `UpdateInstantiateConfigProposal` - update instantiate permissions to a list of given code ids.
* `StoreAndInstantiateContractProposal` - upload and instantiate a wasm contract.

For details see the proposal type [implementation](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/types/proposal.go)

### Unit tests
[Proposal type validations](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/types/proposal_test.go)

## Proposal Handler
The [wasmd proposal_handler](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/keeper/proposal_handler.go) implements the `gov.Handler` function
and executes the wasmd proposal types after a successful tally.
 
The proposal handler uses a [`GovAuthorizationPolicy`](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/keeper/authz_policy.go#L29) to bypass the existing contract's authorization policy.

### Tests
* [Integration: Submit and execute proposal](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/keeper/proposal_integration_test.go)

## Gov Integration
The wasmd proposal handler can be added to the gov router in the [abci app](https://github.com/CosmWasm/wasmd/blob/master/app/app.go#L306)
to receive proposal execution calls. 
```go
govRouter.AddRoute(wasm.RouterKey, wasm.NewWasmProposalHandler(app.wasmKeeper, enabledProposals))
```

## Wasmd Authorization Settings

Settings via sdk `params` module: 
- `code_upload_access` - who can upload a wasm binary: `Nobody`, `Everybody`, `OnlyAddress`
- `instantiate_default_permission` - platform default, who can instantiate a wasm binary when the code owner has not set it 

See [params.go](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/types/params.go)

### Init Params Via Genesis 

```json
    "wasm": {
      "params": {
        "code_upload_access": {
          "permission": "Everybody"
        },
        "instantiate_default_permission": "Everybody"
      }
    },  
```

The values can be updated via gov proposal implemented in the `params` module.

### Update Params Via [ParamChangeProposal](https://github.com/cosmos/cosmos-sdk/blob/v0.45.3/proto/cosmos/params/v1beta1/params.proto#L10)
Example to submit a parameter change gov proposal:
```sh
wasmd tx gov submit-proposal param-change <proposal-json-file> --from validator --chain-id=testing -b block
```
#### Content examples
* Disable wasm code uploads
```json
{
  "title": "Foo",
  "description": "Bar",
  "changes": [
    {
      "subspace": "wasm",
      "key": "uploadAccess",
      "value": {
        "permission": "Nobody"
      }
    }
  ],
  "deposit": ""
}
```
* Allow wasm code uploads for everybody
```json
{
  "title": "Foo",
  "description": "Bar",
  "changes": [
    {
      "subspace": "wasm",
      "key": "uploadAccess",
      "value": {
        "permission": "Everybody"
      }
    }
  ],
  "deposit": ""
}
```

* Restrict code uploads to a single address
```json
{
  "title": "Foo",
  "description": "Bar",
  "changes": [
    {
      "subspace": "wasm",
      "key": "uploadAccess",
      "value": {
        "permission": "OnlyAddress",
        "address": "cosmos1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq0fr2sh"
      }
    }
  ],
  "deposit": ""
}
```
* Set chain **default** instantiation settings to nobody
```json
{
  "title": "Foo",
  "description": "Bar",
  "changes": [
    {
      "subspace": "wasm",
      "key": "instantiateAccess",
      "value": "Nobody"
    }
  ],
  "deposit": ""
}
```
* Set chain **default** instantiation settings to everybody
```json
{
  "title": "Foo",
  "description": "Bar",
  "changes": [
    {
      "subspace": "wasm",
      "key": "instantiateAccess",
      "value": "Everybody"
    }
  ],
  "deposit": ""
}
```

### Enable gov proposals at **compile time**. 
As gov proposals bypass the existing authorization policy they are disabled and require to be enabled at compile time. 
```
-X github.com/CosmWasm/wasmd/app.ProposalsEnabled=true - enable all x/wasm governance proposals (default false)
-X github.com/CosmWasm/wasmd/app.EnableSpecificProposals=MigrateContract,UpdateAdmin,ClearAdmin - enable a subset of the x/wasm governance proposal types (overrides ProposalsEnabled)
```

The `ParamChangeProposal` is always enabled.

### Tests
* [params validation unit tests](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/types/params_test.go)
* [genesis validation tests](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/types/genesis_test.go)
* [policy integration tests](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/keeper/keeper_test.go)

## CLI

```shell script
  wasmd tx gov submit-proposal [command]

Available Commands:
  wasm-store           Submit a wasm binary proposal
  instantiate-contract Submit an instantiate wasm contract proposal
  migrate-contract     Submit a migrate wasm contract to a new code version proposal
  set-contract-admin   Submit a new admin for a contract proposal
  clear-contract-admin Submit a clear admin for a contract to prevent further migrations proposal
...
```
## Rest
New [`ProposalHandlers`](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/client/proposal_handler.go)

* Integration
```shell script
gov.NewAppModuleBasic(append(wasmclient.ProposalHandlers, paramsclient.ProposalHandler, distr.ProposalHandler, upgradeclient.ProposalHandler)...),
```
In [abci app](https://github.com/CosmWasm/wasmd/blob/master/app/app.go#L109)

### Tests
* [Rest Unit tests](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/client/proposal_handler_test.go)
* [Rest smoke LCD test](https://github.com/CosmWasm/wasmd/blob/master/lcd_test/wasm_test.go)



## Pull requests
* https://github.com/CosmWasm/wasmd/pull/190
* https://github.com/CosmWasm/wasmd/pull/186
* https://github.com/CosmWasm/wasmd/pull/183
* https://github.com/CosmWasm/wasmd/pull/180
* https://github.com/CosmWasm/wasmd/pull/179
* https://github.com/CosmWasm/wasmd/pull/173
