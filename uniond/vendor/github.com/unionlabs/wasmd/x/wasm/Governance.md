# Governance

This document gives an overview of how the various governance
proposals interact with the CosmWasm contract lifecycle. It is
a high-level, technical introduction meant to provide context before
looking into the code, or constructing proposals. 

## Proposal Types
We have added 15 new wasm specific proposal messages that cover the contract's live cycle and authorization:
 
* `MsgStoreCode` - upload a wasm binary
* `MsgInstantiateContract` - instantiate a wasm contract
* `MsgInstantiateContract2` - instantiate a wasm contract with a predictable address
* `MsgMigrateContract` - migrate a wasm contract to a new code version
* `MsgSudoContract` - call into the protected `sudo` entry point of a contract
* `MsgExecuteContract` - execute a wasm contract as an arbitrary user
* `MsgUpdateAdmin` - set a new admin for a contract
* `MsgClearAdmin` - clear admin for a contract to prevent further migrations
* `MsgPinCodes` - pin the given code ids in cache. This trades memory for reduced startup time and lowers gas cost
* `MsgUnpinCodes` - unpin the given code ids from the cache. This frees up memory and returns to standard speed and gas cost
* `MsgUpdateInstantiateConfig` - update instantiate permissions to a list of given code ids.
* `MsgStoreAndInstantiateContract` - upload and instantiate a wasm contract.
* `MsgRemoveCodeUploadParamsAddresses` - remove addresses from code upload params.
* `MsgAddCodeUploadParamsAddresses` - add addresses to code upload params.
* `MsgStoreAndMigrateContract` - upload and migrate a wasm contract.

## Wasmd Authorization Settings

Settings via sdk `params` module: 
- `code_upload_access` - who can upload a wasm binary: `Nobody`, `Everybody`, `AnyOfAddresses`
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

The values can be updated via gov proposal `MsgUpdateParams`.

### Update Params Via [MsgUpdateParams](https://github.com/CosmWasm/wasmd/blob/v0.41.0/proto/cosmwasm/wasm/v1/tx.proto#L263)
Example to submit a parameter change gov proposal:

- First create a draft proposal using the interactive CLI
```sh
wasmd tx gov draft-proposal
```

- Submit the proposal
```sh
wasmd tx gov submit-proposal <proposal-json-file> --from validator --chain-id=testing -b block
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
        "permission": "AnyOfAddresses",
        "addresses": ["cosmos1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq0fr2sh"]
      }
    }
  ],
  "deposit": ""
}
```

* Restrict code uploads to two addresses
```json
{
  "title": "Foo",
  "description": "Bar",
  "changes": [
    {
      "subspace": "wasm",
      "key": "uploadAccess",
      "value": {
        "permission": "AnyOfAddresses",
        "addresses": ["cosmos1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq0fr2sh", "cosmos1bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb0fr2sh"]
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

### Tests
* [params validation unit tests](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/types/params_test.go)
* [genesis validation tests](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/types/genesis_test.go)
* [policy integration tests](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/keeper/keeper_test.go)

## CLI

```shell script
  wasmd tx wasm submit-proposal [command]

Available Commands:
  add-code-upload-params-addresses    Submit an add code upload params addresses proposal to add addresses to code upload config params
  clear-contract-admin                Submit a clear admin for a contract to prevent further migrations proposal
  execute-contract                    Submit a execute wasm contract proposal (run by any address)
  instantiate-contract                Submit an instantiate wasm contract proposal
  instantiate-contract-2              Submit an instantiate wasm contract proposal with predictable address
  migrate-contract                    Submit a migrate wasm contract to a new code version proposal
  pin-codes                           Submit a pin code proposal for pinning a code to cache
  remove-code-upload-params-addresses Submit a remove code upload params addresses proposal to remove addresses from code upload config params
  set-contract-admin                  Submit a new admin for a contract proposal
  store-instantiate                   Submit and instantiate a wasm contract proposal
  sudo-contract                       Submit a sudo wasm contract proposal (to call privileged commands)
  unpin-codes                         Submit a unpin code proposal for unpinning a code to cache
  update-instantiate-config           Submit an update instantiate config proposal.
  wasm-store                          Submit a wasm binary proposal
...
```



