

# Sentinel

## Use cases

1. Send native tokens and wrapped tokens back periodically.

2. Trace the ibc events.

## Structure

### Config file

1. Chain's contain `chain_config` for `chain_utils::<chain>::Config`.

```json
{
  "ethereum": {
    "preset": "minimal",
    "chain_config": {
      "ibc_handler_address": "0xed2af2ad7fe0d92011b26a2e5d1b4dc7d12a47c5",
      "signers": [
        {
          "raw": "0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
        }
      ],
      "eth_rpc_api": "ws://localhost:8546",
      "eth_beacon_rpc_api": "http://localhost:9596"
    },
    "transfer_module": {
      "contract": "0xed2af2ad7fe0d92011b26a2e5d1b4dc7d12a47c5"
    }
  },
  "osmosis": {
    "chain_config": {
      "signers": [
        {
          "raw": "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
        }
      ],
      "fee_denom": "uosmo",
      "ws_url": "ws://localhost:26857/websocket",
      "grpc_url": "http://localhost:9290"
    },
    "transfer_module": "native"
  },
  "union": {
    "chain_config": {
      "signers": [
        {
          "raw": "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
        }
      ],
      "fee_denom": "muno",
      "ws_url": "ws://localhost:26657/websocket",
      "prover_endpoint": "https://galois.testnet-8.union.build:443",
      "grpc_url": "http://localhost:9090"
    },    
    "transfer_module": {
      "contract": "union177jpkxrhvzca0dhr7p05ty595ucdgdl6k4wv67500jxcu6t5hppqemdy20"
    }
  },
  "interactions": [
    {
      "source": {
        "chain": "union",
        "channel": "channel-0"
      },
      "target": {
        "chain": "osmosis",
        "channel": "channel-0"
      },
      "protocol": "ics20",
      "send_packet_interval": 40,
      "expect_full_circle": 60
    }
  ]
}
```

