---
order: 1
parent:
  title: RPC
  order: 6
---

# RPC spec

This file defines the JSON-RPC spec of CometBFT. This is meant to be implemented by all clients.

## Support

  |              | [CometBFT](https://github.com/cometbft/cometbft/) | [Tendermint-Rs](https://github.com/informalsystems/tendermint-rs) |
  |--------------|:----------------------------------------------------------:|:----------------------------------------------------------------:|
  | JSON-RPC 2.0 |                             ✅                              |                                ✅                                 |
  | HTTP         |                             ✅                              |                                ✅                                 |
  | HTTPS        |                             ✅                              |                                ❌                                 |
  | WS           |                             ✅                              |                                ✅                                 |
  
  | Routes                                  | [CometBFT](https://github.com/cometbft/cometbft/) | [Tendermint-Rs](https://github.com/informalsystems/tendermint-rs) |
  |-----------------------------------------|:----------------------------------------------------------:|:-----------------------------------------------------------------:|
  | [Health](#health)                       |                             ✅                              |                                 ✅                                 |
  | [Status](#status)                       |                             ✅                              |                                 ✅                                 |
  | [NetInfo](#netinfo)                     |                             ✅                              |                                 ✅                                 |
  | [Blockchain](#blockchain)               |                             ✅                              |                                 ✅                                 |
  | [Block](#block)                         |                             ✅                              |                                 ✅                                 |
  | [BlockByHash](#blockbyhash)             |                             ✅                              |                                 ❌                                 |
  | [BlockResults](#blockresults)           |                             ✅                              |                                 ✅                                 |
  | [Commit](#commit)                       |                             ✅                              |                                 ✅                                 |
  | [Validators](#validators)               |                             ✅                              |                                 ✅                                 |
  | [Genesis](#genesis)                     |                             ✅                              |                                 ✅                                 |
  | [GenesisChunked](#genesischunked)       |                             ✅                              |                                 ❌                                 |
  | [ConsensusParams](#consensusparams)     |                             ✅                              |                                 ❌                                 |
  | [UnconfirmedTxs](#unconfirmedtxs)       |                             ✅                              |                                 ❌                                 |
  | [NumUnconfirmedTxs](#numunconfirmedtxs) |                             ✅                              |                                 ❌                                 |
  | [Tx](#tx)                               |                             ✅                              |                                 ❌                                 |
  | [BroadCastTxSync](#broadcasttxsync)     |                             ✅                              |                                 ✅                                 |
  | [BroadCastTxAsync](#broadcasttxasync)   |                             ✅                              |                                 ✅                                 |
  | [ABCIInfo](#abciinfo)                   |                             ✅                              |                                 ✅                                 |
  | [ABCIQuery](#abciquery)                 |                             ✅                              |                                 ✅                                 |
  | [BroadcastTxAsync](#broadcasttxasync)   |                             ✅                              |                                 ✅                                 |
  | [BroadcastEvidence](#broadcastevidence) |                             ✅                              |                                 ✅                                 |

## Timestamps

Timestamps in the RPC layer of CometBFT follows RFC3339Nano.  The RFC3339Nano format removes trailing zeros from the seconds field.

This means if a block has a timestamp like: `1985-04-12T23:20:50.5200000Z`, the value returned in the RPC will be `1985-04-12T23:20:50.52Z`.



## Info Routes

### Health

Node heartbeat

#### Parameters

None

#### Request

##### HTTP

```sh
curl http://127.0.0.1:26657/v1/health
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"health\"}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": -1,
  "result": {}
}
```

### Status

Get CometBFT status including node info, pubkey, latest block hash, app hash, block height and time.

#### Parameters

None

#### Request

##### HTTP

```sh
curl http://127.0.0.1:26657/v1/status
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"status\"}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": -1,
  "result": {
    "node_info": {
      "protocol_version": {
        "p2p": "8",
        "block": "11",
        "app": "0"
      },
      "id": "b93270b358a72a2db30089f3856475bb1f918d6d",
      "listen_addr": "tcp://0.0.0.0:26656",
      "network": "cosmoshub-4",
      "version": "v0.34.8",
      "channels": "40202122233038606100",
      "moniker": "aib-hub-node",
      "other": {
        "tx_index": "on",
        "rpc_address": "tcp://0.0.0.0:26657"
      }
    },
    "sync_info": {
      "latest_block_hash": "50F03C0EAACA8BCA7F9C14189ACE9C05A9A1BBB5268DB63DC6A3C848D1ECFD27",
      "latest_app_hash": "2316CFF7644219F4F15BEE456435F280E2B38955EEA6D4617CCB6D7ABF781C22",
      "latest_block_height": "5622165",
      "latest_block_time": "2021-03-25T14:00:43.356134226Z",
      "earliest_block_hash": "1455A0C15AC49BB506992EC85A3CD4D32367E53A087689815E01A524231C3ADF",
      "earliest_app_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
      "earliest_block_height": "5200791",
      "earliest_block_time": "2019-12-11T16:11:34Z",
      "catching_up": false
    },
    "validator_info": {
      "address": "38FB765D0092470989360ECA1C89CD06C2C1583C",
      "pub_key": {
        "type": "tendermint/PubKeyEd25519",
        "value": "Z+8kntVegi1sQiWLYwFSVLNWqdAUGEy7lskL78gxLZI="
      },
      "voting_power": "0"
    }
  }
}
```

### NetInfo

Network information

#### Parameters

None

#### Request

##### HTTP

```sh
curl http://127.0.0.1:26657/v1/net_info
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"net_info\"}"
```

#### Response

```json
{
  "id": 0,
  "jsonrpc": "2.0",
  "result": {
    "listening": true,
    "listeners": [
      "Listener(@)"
    ],
    "n_peers": "1",
	"peers": [
	  {
		  "node_id": "5576458aef205977e18fd50b274e9b5d9014525a",
		  "url": "tcp://5576458aef205977e18fd50b274e9b5d9014525a@95.179.155.35:26656"
	  }
	]
  }
}
```

### Blockchain

Get block headers. Returned in descending order. May be limited in quantity.

#### Parameters

- `minHeight (integer)`: The lowest block to be returned in the response
- `maxHeight (integer)`: The highest block to be returned in the response

#### Request

##### HTTP

```sh
curl http://127.0.0.1:26657/v1/blockchain

curl http://127.0.0.1:26657/v1/blockchain?minHeight=1&maxHeight=2
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"blockchain\",\"params\":{\"minHeight\":\"1\", \"maxHeight\":\"2\"}}"
```

#### Response

```json
{
  "id": 0,
  "jsonrpc": "2.0",
  "result": {
    "last_height": "1276718",
    "block_metas": [
      {
        "block_id": {
          "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
          "parts": {
            "total": 1,
            "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
          }
        },
        "block_size": 1000000,
        "header": {
          "version": {
            "block": "10",
            "app": "0"
          },
          "chain_id": "cosmoshub-2",
          "height": "12",
          "time": "2019-04-22T17:01:51.701356223Z",
          "last_block_id": {
            "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
            "parts": {
              "total": 1,
              "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
            }
          },
          "last_commit_hash": "21B9BC845AD2CB2C4193CDD17BFC506F1EBE5A7402E84AD96E64171287A34812",
          "data_hash": "970886F99E77ED0D60DA8FCE0447C2676E59F2F77302B0C4AA10E1D02F18EF73",
          "validators_hash": "D658BFD100CA8025CFD3BECFE86194322731D387286FBD26E059115FD5F2BCA0",
          "next_validators_hash": "D658BFD100CA8025CFD3BECFE86194322731D387286FBD26E059115FD5F2BCA0",
          "consensus_hash": "0F2908883A105C793B74495EB7D6DF2EEA479ED7FC9349206A65CB0F9987A0B8",
          "app_hash": "223BF64D4A01074DC523A80E76B9BBC786C791FB0A1893AC5B14866356FCFD6C",
          "last_results_hash": "",
          "evidence_hash": "",
          "proposer_address": "D540AB022088612AC74B287D076DBFBC4A377A2E"
        },
        "num_txs": "54"
      }
    ]
  }
}
```

### Block

Get block at a specified height.

#### Parameters

- `height (integer)`: height of the requested block. If no height is specified the latest block will be used.

#### Request

##### HTTP

```sh
curl http://127.0.0.1:26657/v1/block

curl http://127.0.0.1:26657/v1/block?height=1
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"block\",\"params\":{\"height\":\"1\"}}"
```

#### Response

```json
{
  "id": 0,
  "jsonrpc": "2.0",
  "result": {
    "block_id": {
      "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
      "parts": {
        "total": 1,
        "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
      }
    },
    "block": {
      "header": {
        "version": {
          "block": "10",
          "app": "0"
        },
        "chain_id": "cosmoshub-2",
        "height": "12",
        "time": "2019-04-22T17:01:51.701356223Z",
        "last_block_id": {
          "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
          "parts": {
            "total": 1,
            "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
          }
        },
        "last_commit_hash": "21B9BC845AD2CB2C4193CDD17BFC506F1EBE5A7402E84AD96E64171287A34812",
        "data_hash": "970886F99E77ED0D60DA8FCE0447C2676E59F2F77302B0C4AA10E1D02F18EF73",
        "validators_hash": "D658BFD100CA8025CFD3BECFE86194322731D387286FBD26E059115FD5F2BCA0",
        "next_validators_hash": "D658BFD100CA8025CFD3BECFE86194322731D387286FBD26E059115FD5F2BCA0",
        "consensus_hash": "0F2908883A105C793B74495EB7D6DF2EEA479ED7FC9349206A65CB0F9987A0B8",
        "app_hash": "223BF64D4A01074DC523A80E76B9BBC786C791FB0A1893AC5B14866356FCFD6C",
        "last_results_hash": "",
        "evidence_hash": "",
        "proposer_address": "D540AB022088612AC74B287D076DBFBC4A377A2E"
      },
      "data": [
        "yQHwYl3uCkKoo2GaChRnd+THLQ2RM87nEZrE19910Z28ABIUWW/t8AtIMwcyU0sT32RcMDI9GF0aEAoFdWF0b20SBzEwMDAwMDASEwoNCgV1YXRvbRIEMzEwMRCd8gEaagom61rphyEDoJPxlcjRoNDtZ9xMdvs+lRzFaHe2dl2P5R2yVCWrsHISQKkqX5H1zXAIJuC57yw0Yb03Fwy75VRip0ZBtLiYsUqkOsPUoQZAhDNP+6LY+RUwz/nVzedkF0S29NZ32QXdGv0="
      ],
      "evidence": [
        {
          "type": "string",
          "height": 0,
          "time": 0,
          "total_voting_power": 0,
          "validator": {
            "pub_key": {
              "type": "tendermint/PubKeyEd25519",
              "value": "A6DoBUypNtUAyEHWtQ9bFjfNg8Bo9CrnkUGl6k6OHN4="
            },
            "voting_power": 0,
            "address": "string"
          }
        }
      ],
      "last_commit": {
        "height": 0,
        "round": 0,
        "block_id": {
          "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
          "parts": {
            "total": 1,
            "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
          }
        },
        "signatures": [
          {
            "type": 2,
            "height": "1262085",
            "round": 0,
            "block_id": {
              "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
              "parts": {
                "total": 1,
                "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
              }
            },
            "timestamp": "2019-08-01T11:39:38.867269833Z",
            "validator_address": "000001E443FD237E4B616E2FA69DF4EE3D49A94F",
            "validator_index": 0,
            "signature": "DBchvucTzAUEJnGYpNvMdqLhBAHG4Px8BsOBB3J3mAFCLGeuG7uJqy+nVngKzZdPhPi8RhmE/xcw/M9DOJjEDg=="
          }
        ]
      }
    }
  }
}
```

### BlockByHash

#### Parameters

- `hash (string)`: Hash of the block to query for.

#### Request

##### HTTP

```sh
curl http://127.0.0.1:26657/v1/block_by_hash?hash=0xD70952032620CC4E2737EB8AC379806359D8E0B17B0488F627997A0B043ABDED
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"block_by_hash\",\"params\":{\"hash\":\"0xD70952032620CC4E2737EB8AC379806359D8E0B17B0488F627997A0B043ABDED\"}}"
```

#### Response

```json
{
  "id": 0,
  "jsonrpc": "2.0",
  "result": {
    "block_id": {
      "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
      "parts": {
        "total": 1,
        "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
      }
    },
    "block": {
      "header": {
        "version": {
          "block": "10",
          "app": "0"
        },
        "chain_id": "cosmoshub-2",
        "height": "12",
        "time": "2019-04-22T17:01:51.701356223Z",
        "last_block_id": {
          "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
          "parts": {
            "total": 1,
            "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
          }
        },
        "last_commit_hash": "21B9BC845AD2CB2C4193CDD17BFC506F1EBE5A7402E84AD96E64171287A34812",
        "data_hash": "970886F99E77ED0D60DA8FCE0447C2676E59F2F77302B0C4AA10E1D02F18EF73",
        "validators_hash": "D658BFD100CA8025CFD3BECFE86194322731D387286FBD26E059115FD5F2BCA0",
        "next_validators_hash": "D658BFD100CA8025CFD3BECFE86194322731D387286FBD26E059115FD5F2BCA0",
        "consensus_hash": "0F2908883A105C793B74495EB7D6DF2EEA479ED7FC9349206A65CB0F9987A0B8",
        "app_hash": "223BF64D4A01074DC523A80E76B9BBC786C791FB0A1893AC5B14866356FCFD6C",
        "last_results_hash": "",
        "evidence_hash": "",
        "proposer_address": "D540AB022088612AC74B287D076DBFBC4A377A2E"
      },
      "data": [
        "yQHwYl3uCkKoo2GaChRnd+THLQ2RM87nEZrE19910Z28ABIUWW/t8AtIMwcyU0sT32RcMDI9GF0aEAoFdWF0b20SBzEwMDAwMDASEwoNCgV1YXRvbRIEMzEwMRCd8gEaagom61rphyEDoJPxlcjRoNDtZ9xMdvs+lRzFaHe2dl2P5R2yVCWrsHISQKkqX5H1zXAIJuC57yw0Yb03Fwy75VRip0ZBtLiYsUqkOsPUoQZAhDNP+6LY+RUwz/nVzedkF0S29NZ32QXdGv0="
      ],
      "evidence": [
        {
          "type": "string",
          "height": 0,
          "time": 0,
          "total_voting_power": 0,
          "validator": {
            "pub_key": {
              "type": "tendermint/PubKeyEd25519",
              "value": "A6DoBUypNtUAyEHWtQ9bFjfNg8Bo9CrnkUGl6k6OHN4="
            },
            "voting_power": 0,
            "address": "string"
          }
        }
      ],
      "last_commit": {
        "height": 0,
        "round": 0,
        "block_id": {
          "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
          "parts": {
            "total": 1,
            "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
          }
        },
        "signatures": [
          {
            "type": 2,
            "height": "1262085",
            "round": 0,
            "block_id": {
              "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
              "parts": {
                "total": 1,
                "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
              }
            },
            "timestamp": "2019-08-01T11:39:38.867269833Z",
            "validator_address": "000001E443FD237E4B616E2FA69DF4EE3D49A94F",
            "validator_index": 0,
            "signature": "DBchvucTzAUEJnGYpNvMdqLhBAHG4Px8BsOBB3J3mAFCLGeuG7uJqy+nVngKzZdPhPi8RhmE/xcw/M9DOJjEDg=="
          }
        ]
      }
    }
  }
}
```

### BlockResults

### Parameters

- `height (integer)`: Height of the block which contains the results. If no height is specified, the latest block height will be used

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/block_results


curl  http://127.0.0.1:26657/v1/block_results?height=1
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"block_results\",\"params\":{\"height\":\"1\"}}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "height": "12",
    "total_gas_used": "100",
    "txs_results": [
      {
        "code": "0",
        "data": "",
        "log": "not enough gas",
        "info": "",
        "gas_wanted": "100",
        "gas_used": "100",
        "events": [
          {
            "type": "app",
            "attributes": [
              {
                "key": "YWN0aW9u",
                "value": "c2VuZA==",
                "index": false
              }
            ]
          }
        ],
        "codespace": "ibc"
      }
    ],
    "begin_block_events": [
      {
        "type": "app",
        "attributes": [
          {
            "key": "YWN0aW9u",
            "value": "c2VuZA==",
            "index": false
          }
        ]
      }
    ],
    "end_block": [
      {
        "type": "app",
        "attributes": [
          {
            "key": "YWN0aW9u",
            "value": "c2VuZA==",
            "index": false
          }
        ]
      }
    ],
    "validator_updates": [
      {
        "pub_key": {
          "type": "tendermint/PubKeyEd25519",
          "value": "9tK9IT+FPdf2qm+5c2qaxi10sWP+3erWTKgftn2PaQM="
        },
        "power": "300"
      }
    ],
    "consensus_params_updates": {
      "block": {
        "max_bytes": "22020096",
        "max_gas": "1000",
        "time_iota_ms": "1000"
      },
      "evidence": {
        "max_age": "100000"
      },
      "validator": {
        "pub_key_types": [
          "ed25519"
        ]
      }
    }
  }
}
```

### Commit

#### Parameters

- `height (integer)`: Height of the block the requested commit pertains to. If no height is set the latest commit will be returned.

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/commit


curl  http://127.0.0.1:26657/v1/commit?height=1
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"commit\",\"params\":{\"height\":\"1\"}}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "signed_header": {
      "header": {
        "version": {
          "block": "10",
          "app": "0"
        },
        "chain_id": "cosmoshub-2",
        "height": "12",
        "time": "2019-04-22T17:01:51.701356223Z",
        "last_block_id": {
          "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
          "parts": {
            "total": 1,
            "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
          }
        },
        "last_commit_hash": "21B9BC845AD2CB2C4193CDD17BFC506F1EBE5A7402E84AD96E64171287A34812",
        "data_hash": "970886F99E77ED0D60DA8FCE0447C2676E59F2F77302B0C4AA10E1D02F18EF73",
        "validators_hash": "D658BFD100CA8025CFD3BECFE86194322731D387286FBD26E059115FD5F2BCA0",
        "next_validators_hash": "D658BFD100CA8025CFD3BECFE86194322731D387286FBD26E059115FD5F2BCA0",
        "consensus_hash": "0F2908883A105C793B74495EB7D6DF2EEA479ED7FC9349206A65CB0F9987A0B8",
        "app_hash": "223BF64D4A01074DC523A80E76B9BBC786C791FB0A1893AC5B14866356FCFD6C",
        "last_results_hash": "",
        "evidence_hash": "",
        "proposer_address": "D540AB022088612AC74B287D076DBFBC4A377A2E"
      },
      "commit": {
        "height": "1311801",
        "round": 0,
        "block_id": {
          "hash": "112BC173FD838FB68EB43476816CD7B4C6661B6884A9E357B417EE957E1CF8F7",
          "parts": {
            "total": 1,
            "hash": "38D4B26B5B725C4F13571EFE022C030390E4C33C8CF6F88EDD142EA769642DBD"
          }
        },
        "signatures": [
          {
            "block_id_flag": 2,
            "validator_address": "000001E443FD237E4B616E2FA69DF4EE3D49A94F",
            "timestamp": "2019-04-22T17:01:58.376629719Z",
            "signature": "14jaTQXYRt8kbLKEhdHq7AXycrFImiLuZx50uOjs2+Zv+2i7RTG/jnObD07Jo2ubZ8xd7bNBJMqkgtkd0oQHAw=="
          }
        ]
      }
    },
    "canonical": true
  }
}
```

### Validators

#### Parameters

- `height (integer)`: Block height at which the validators were present on. If no height is set the latest commit will be returned.
- `page (integer)`:
- `per_page (integer)`:

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/validators
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"validators\",\"params\":{\"height\":\"1\", \"page\":\"1\", \"per_page\":\"20\"}}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "block_height": "55",
    "validators": [
      {
        "address": "000001E443FD237E4B616E2FA69DF4EE3D49A94F",
        "pub_key": {
          "type": "tendermint/PubKeyEd25519",
          "value": "9tK9IT+FPdf2qm+5c2qaxi10sWP+3erWTKgftn2PaQM="
        },
        "voting_power": "239727",
        "proposer_priority": "-11896414"
      }
    ],
    "count": "1",
    "total": "25"
  }
}
```

### Genesis

Get Genesis of the chain. If the response is large, this operation
will return an error: use `genesis_chunked` instead.

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/genesis
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"genesis\"}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "genesis": {
      "genesis_time": "2019-04-22T17:00:00Z",
      "chain_id": "cosmoshub-2",
      "initial_height": "2",
      "consensus_params": {
        "block": {
          "max_bytes": "22020096",
          "max_gas": "1000",
          "time_iota_ms": "1000"
        },
        "evidence": {
          "max_age": "100000"
        },
        "validator": {
          "pub_key_types": [
            "ed25519"
          ]
        }
      },
      "validators": [
        {
          "address": "B00A6323737F321EB0B8D59C6FD497A14B60938A",
          "pub_key": {
            "type": "tendermint/PubKeyEd25519",
            "value": "cOQZvh/h9ZioSeUMZB/1Vy1Xo5x2sjrVjlE/qHnYifM="
          },
          "power": "9328525",
          "name": "Certus One"
        }
      ],
      "app_hash": "",
      "app_state": {}
    }
  }
}
```

### GenesisChunked

Get the genesis document in a chunks to support easily transferring larger documents.

#### Parameters

- `chunk` (integer): the index number of the chunk that you wish to
  fetch. These IDs are 0 indexed.

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/genesis_chunked?chunk=0
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"genesis_chunked\",\"params\":{\"chunk\":0}}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
     "chunk": 0,
     "total": 10,
     "data": "dGVuZGVybWludAo="
  }
}
```

### ConsensusParams

Get the consensus parameters.

#### Parameters

- `height (integer)`: Block height at which the consensus params would like to be fetched for.

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/consensus_params
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"consensus_params\"}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "block_height": "1",
    "consensus_params": {
      "block": {
        "max_bytes": "22020096",
        "max_gas": "1000",
        "time_iota_ms": "1000"
      },
      "evidence": {
        "max_age": "100000"
      },
      "validator": {
        "pub_key_types": [
          "ed25519"
        ]
      }
    }
  }
}
```

### UnconfirmedTxs

Get a list of unconfirmed transactions.

#### Parameters

- `limit (integer)` The amount of txs to respond with.

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/unconfirmed_txs
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"unconfirmed_txs\, \"params\":{\"limit\":\"20\"}}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "n_txs": "82",
    "total": "82",
    "total_bytes": "19974",
    "txs": [
      "gAPwYl3uCjCMTXENChSMnIkb5ZpYHBKIZqecFEV2tuZr7xIUA75/FmYq9WymsOBJ0XSJ8yV8zmQKMIxNcQ0KFIyciRvlmlgcEohmp5wURXa25mvvEhQbrvwbvlNiT+Yjr86G+YQNx7kRVgowjE1xDQoUjJyJG+WaWBwSiGannBRFdrbma+8SFK2m+1oxgILuQLO55n8mWfnbIzyPCjCMTXENChSMnIkb5ZpYHBKIZqecFEV2tuZr7xIUQNGfkmhTNMis4j+dyMDIWXdIPiYKMIxNcQ0KFIyciRvlmlgcEohmp5wURXa25mvvEhS8sL0D0wwgGCItQwVowak5YB38KRIUCg4KBXVhdG9tEgUxMDA1NBDoxRgaagom61rphyECn8x7emhhKdRCB2io7aS/6Cpuq5NbVqbODmqOT3jWw6kSQKUresk+d+Gw0BhjiggTsu8+1voW+VlDCQ1GRYnMaFOHXhyFv7BCLhFWxLxHSAYT8a5XqoMayosZf9mANKdXArA="
    ]
  }
}
```

### NumUnconfirmedTxs

Get data about unconfirmed transactions.

#### Parameters

None

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/num_unconfirmed_txs
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"num_unconfirmed_txs\"}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "n_txs": "31",
    "total": "82",
    "total_bytes": "19974"
  }
}
```

### Tx

#### Parameters

- `hash (string)`: The hash of the transaction
- `prove (bool)`: If the response should include proof the transaction was included in a block.

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/num_unconfirmed_txs
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"num_unconfirmed_txs\"}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "hash": "D70952032620CC4E2737EB8AC379806359D8E0B17B0488F627997A0B043ABDED",
    "height": "1000",
    "index": 0,
    "tx_result": {
      "log": "[{\"msg_index\":\"0\",\"success\":true,\"log\":\"\"}]",
      "gas_wanted": "200000",
      "gas_used": "28596",
      "tags": [
        {
          "key": "YWN0aW9u",
          "value": "c2VuZA==",
          "index": false
        }
      ]
    },
    "tx": "5wHwYl3uCkaoo2GaChQmSIu8hxpJxLcCuIi8fiHN4TMwrRIU/Af1cEG7Rcs/6LjTl7YjRSymJfYaFAoFdWF0b20SCzE0OTk5OTk1MDAwEhMKDQoFdWF0b20SBDUwMDAQwJoMGmoKJuta6YchAwswBShaB1wkZBctLIhYqBC3JrAI28XGzxP+rVEticGEEkAc+khTkKL9CDE47aDvjEHvUNt+izJfT4KVF2v2JkC+bmlH9K08q3PqHeMI9Z5up+XMusnTqlP985KF+SI5J3ZOIhhNYWRlIGJ5IENpcmNsZSB3aXRoIGxvdmU="
  }
}
```

## Transaction Routes

### BroadCastTxSync

Returns with the response from CheckTx. Does not wait for DeliverTx result.

#### Parameters

- `tx (string)`: The transaction encoded

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/broadcast_tx_sync?tx=encoded_tx
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"broadcast_tx_sync\",\"params\":{\"tx\":\"a/encoded_tx/c\"}}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "code": "0",
    "data": "",
    "log": "",
    "codespace": "ibc",
    "hash": "0D33F2F03A5234F38706E43004489E061AC40A2E"
  },
  "error": ""
}
```

### BroadCastTxAsync

Returns right away, with no response. Does not wait for CheckTx nor DeliverTx results.

#### Parameters

- `tx (string)`: The transaction encoded

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/broadcast_tx_async?tx=encoded_tx
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"broadcast_tx_async\",\"params\":{\"tx\":\"a/encoded_tx/c\"}}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "code": "0",
    "data": "",
    "log": "",
    "codespace": "ibc",
    "hash": "0D33F2F03A5234F38706E43004489E061AC40A2E"
  },
  "error": ""
}
```

### CheckTx

Checks the transaction without executing it.

#### Parameters

- `tx (string)`: String of the encoded transaction

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/check_tx?tx=encoded_tx
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"check_tx\",\"params\":{\"tx\":\"a/encoded_tx/c\"}}"
```

#### Response

```json
{
  "id": 0,
  "jsonrpc": "2.0",
  "error": "",
  "result": {
    "code": "0",
    "data": "",
    "log": "",
    "info": "",
    "gas_wanted": "1",
    "gas_used": "0",
    "events": [
      {
        "type": "app",
        "attributes": [
          {
            "key": "YWN0aW9u",
            "value": "c2VuZA==",
            "index": false
          }
        ]
      }
    ],
    "codespace": "bank"
  }
}
```

## ABCI Routes

### ABCIInfo

Get some info about the application.

#### Parameters

None

#### Request

##### HTTP

```sh
curl  http://127.0.0.1:26657/v1/abci_info
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"abci_info\"}"
```

#### Response

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "response": {
      "data": "{\"size\":0}",
      "version": "0.16.1",
      "app_version": "1314126"
    }
  }
}
```

### ABCIQuery

Query the application for some information.

#### Parameters

- `path (string)`: A request path for the application to interpret analogously to a [URI path component](https://www.rfc-editor.org/rfc/rfc3986#section-3.3) in e.g. routing.
- `data (string)`: Request parameters for the application to interpret analogously to a [URI query component](https://www.rfc-editor.org/rfc/rfc3986#section-3.4), expressed as hexadecimal-serialized bytes.
- `height (integer)`: The block height against which to query.
- `prove (bool)`: Return Merkle proof with response if possible.

#### Request

##### HTTP

```sh
curl 'http://127.0.0.1:26657/v1/abci_query?path="/store/foo/key"&data=0x636f6d6574626674&height=1&prove=true'
```

##### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"abci_query\",\"params\":{\"path\":\"/store/foo/key\", \"data\":\"636f6d6574626674\", \"height\":\"1\", \"prove\":\"true\"}}"
```

#### Response

```json
{
  "error": "",
  "result": {
    "response": {
      "log": "exists",
      "height": "0",
      "proof": "010114FED0DAD959F36091AD761C922ABA3CBF1D8349990101020103011406AA2262E2F448242DF2C2607C3CDC705313EE3B0001149D16177BC71E445476174622EA559715C293740C",
      "key": "Y29tZXRiZnQ=",
      "value": "cm9ja3M=",
      "index": "-1",
      "code": "0"
    }
  },
  "id": 0,
  "jsonrpc": "2.0"
}
```

## Evidence Routes

### BroadcastEvidence

Broadcast evidence of the misbehavior.

#### Parameters

- `evidence (string)`:

#### Request

##### HTTP

```sh
curl http://localhost:26657/v1/broadcast_evidence?evidence=JSON_EVIDENCE_encoded
```

#### JSONRPC

```sh
curl -X POST https://localhost:26657/v1 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"broadcast_evidence\",\"params\":{\"evidence\":\"JSON_EVIDENCE_encoded\"}}"
```

#### Response

```json
{
  "error": "",
  "result": "",
  "id": 0,
  "jsonrpc": "2.0"
}
```
