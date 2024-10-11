---
order: 1
parent:
  title: genesis.json
  description: The network genesis file
  order: 1
---
# genesis.json
It is **crucial** that all nodes in a network must have _exactly_ the same contents in their `genesis.json` file.

On first start, the network parameters are read from the `genesis.json` file.
On subsequent starts (node recovery), the `genesis.json` file is ignored.

### Example
```json
{
  "genesis_time": "2024-03-01T20:22:57.532998Z",
  "chain_id": "test-chain-HfdKnD",
  "initial_height": "0",
  "consensus_params": {
    "block": {
      "max_bytes": "4194304",
      "max_gas": "10000000"
    },
    "evidence": {
      "max_age_num_blocks": "100000",
      "max_age_duration": "172800000000000",
      "max_bytes": "1048576"
    },
    "validator": {
      "pub_key_types": [
        "ed25519"
      ]
    },
    "version": {
      "app": "0"
    },
    "feature": {
      "vote_extensions_enable_height": "1"
      "pbts_enable_height": "1"
    }
  },
  "validators": [
    {
      "address": "E74FBE24164CFC4F88E311C3AC92E63D0DC310D8",
      "pub_key": {
        "type": "tendermint/PubKeyEd25519",
        "value": "UjxDQgVTlHJOZ7axpMl/iczMIJXiQpFxCFjwKGvzYqE="
      },
      "power": "10",
      "name": ""
    }
  ],
  "app_hash": ""
}
```

For a production example, you can see [here](https://github.com/cosmos/mainnet/tree/master/genesis)
the history of genesis files for the Cosmos Hub network.

## genesis_time
Timestamp of the genesis file creation.

| Value type          | string                     |
|:--------------------|:---------------------------|
| **Possible values** | RFC3339-formatted datetime |

RFC3339 has multiple representation. The one we use here has 6 digits for sub-second representation.

## chain_id
The chain ID of the blockchain network.

| Value type          | string                            |
|:--------------------|:----------------------------------|
| **Possible values** | usually in `"name-number"` format |

Cannot be empty.

Can be maximum 50 UTF-8-encoded character.

The `number` part is typically a revision number of the blockchain, starting at `1` and incrementing each time the network
undergoes a hard fork.

## initial_height
Initial height at genesis.

| Value type          | string      |
|:--------------------|:------------|
| **Possible values** | &gt;= `"0"` |

When a hard fork happens, a new chain can start from a higher initial height by setting this parameter.

> Notes:

>> If a Height `"0"` is specified in `initial_heigth`, then CometBFT during the genesis file validation, will change the
> initial height parameter to `"1"`.

>> Note: A height in CometBFT is an `int64` integer therefore its maximum value is `9223372036854775807`

## consensus_params

The initial values for the consensus parameters.
Consensus Parameters are global parameters that apply to all nodes in the network.

Please refer to the
[specification](https://docs.cometbft.com/v1.0/spec/abci/abci++_app_requirements#consensus-parameters)
for details on the existing consensus parameters, their default and valid values.


## validators
List of initial validators for consensus.

| Value type                              | array of objects |                                                                                         |
|:----------------------------------------|:-----------------|-----------------------------------------------------------------------------------------|
| **Mandatory keys of each array object** | address          | See [address](priv_validator_key.json.md#address) in priv_validator_key.json            |
|                                         | pub_key          | See [pub_key.type](priv_validator_key.json.md#pub_keytype)                              |
|                                         |                  | and [pub_key.value](priv_validator_key.json.md#pub_keyvalue) in priv_validator_key.json |
|                                         | power            | &gt;  `"0"`                                                                             |
|                                         | name             | string or `""`                                                                          |

## app_hash
The initial AppHash, represented by the state embedded in the genesis file.

| Value type          | string             |
|:--------------------|:-------------------|
| **Possible values** | hex-encoded number |
|                     | ""                 |

## app_state
A raw encoded JSON value that has the application state encoded in it.

| Value type          | string                 |
|:--------------------|:-----------------------|
| **Possible values** | raw bytes JSON-encoded |
|                     | ""                     |
