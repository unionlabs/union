# Changes to the api

## [\#196](https://github.com/CosmWasm/wasmd/issues/196) - Move history of contract code migrations to their own prefix store

The `ContractDetails.initMsg` used in cosmJs was moved into a new entity `ContractCodeHistoryEntry`. They contain code updates to a contract.

### Route
This data is available via a new route `/wasm/contract/{contractAddr}/history`

### Response
A list of ContractCodeHistoryEntries with following fields:
* `operation` can be any of `"Init", "Migrate", "Genesis"`
* `code_id` uint64
* `msg` as raw json

### Errors
* 404 - for an unknown contract

### CLI
`wasmd query wasm contract-history [bech32_address] to print all the code changes.`
Example:
`wasmd query wasm contract-history cosmos18r5szma8hm93pvx6lwpjwyxruw27e0k5uw835c` 
```json
[
  {
    "operation": "Init",
    "code_id": 1,
    "msg": "\"init-msg\""
  },
  {
    "operation": "Migrate",
    "code_id": 2,
    "msg": "\"migrate-msg\""
  }
]
```
