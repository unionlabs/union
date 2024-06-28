

# Sentinel

Sentinel is a blockchain interaction service designed to automate the periodic transfer of native and wrapped tokens between different blockchain networks. It also traces Inter-Blockchain Communication (IBC) events, handles token distributions, and ensures successful transaction completion. For monitoring and error tracking, Sentinel integrates with Datadog and BetterStack to provide alerts and notifications.

## Use Cases

1. **Send Native and Wrapped Tokens:** Sentinel periodically sends tokens between different chains, managing both native and wrapped tokens.
2. **Trace IBC Events:** Sentinel monitors IBC events to ensure transactions are completed successfully and handles any issues that arise.

### Config file

1. Chain's contain `chain_config` for `chain_utils::<chain>::Config`.

```json
{
  "chain_configs": {
    "ethereum": {
      "ethereum": {
        "enabled": true,
        "ibc_handler_address": "0xa390514f803a3b318b93bf6cd4beeb9f8299a0eb",
        "signers": [
          { "raw": "0x1419fbc200ea996170f89684d523637cdbf50db4e5285fde8da6d34558fb354a" }
        ],        
        "eth_rpc_api": "wss://eth-sepolia.g.alchemy.com/v2/Xn_VBUDyUtXUYb9O6b5ZmuBNDaSlH-BB",
        "transfer_module": {
          "type": "contract",
          "address": "0xd0081080ae8493cf7340458eaf4412030df5feeb"
        },
        "master_account": {"raw": "0x09368c5c0c4d6427bea98bc2ef0ee4a25442fb798633d2d91437a9dcc64fc5b9"}
      }
    },
    "bera": {
      "ethereum": {
        "enabled": false,
        "ibc_handler_address": "0x4e86d3eb0f4d8ddccec2b8fa5ccfc8170e8ac3dc",
        "signers": [
          {
            "raw": "0x09368c5c0c4d6427bea98bc2ef0ee4a25442fb798633d2d91437a9dcc64fc5b9"
          }
        ],
        "eth_rpc_api": "wss://fabled-serene-mountain.bera-bartio.quiknode.pro/6ab3f499dcce3d52591ce97a5f07a13fae75deb1",
        "transfer_module": {
          "type": "contract",
          "address": "0x0e7aee8a4109b1c1916281d25f43b937f103a409"
        },
        "master_account": {"raw": "0xc56de6bf91c78afb4757055a080e6c3a53f83e588750e39196f4e65931bc86a2"}

      }
    },
    "osmosis": {
      "cosmos": {
        "enabled": false,
        "chain_config": {
          "keyring": {
            "name": "osmosis-testnet",
            "keys": [
              {
                "type": "raw",
                "name": "osmosis-testnet-key0",
                "key": "0x1503e463998e28b130a2d4876632c80462bbd5e0d9eb7ce6ed5f6210f02a2913"
              }
            ]
          },
          "gas_config": {
            "gas_price": "1.0",
            "gas_denom": "uosmo",
            "gas_multiplier": "1.1",
            "max_gas": 400000
          },
          "fee_denom": "uosmo",
          "ws_url": "wss://rpc.osmo.test.yieldpay.finance/websocket",
          "grpc_url": "https://grpc.osmo.test.yieldpay.finance:443"
        },
        "transfer_module": {
          "type": "native"
        }
      }
    },
    "union": {
      "cosmos": {
        "enabled": true,
        "chain_config": {
          "keyring": {
            "name": "union-testnet",
            "keys": [
              {
                "type": "raw",
                "name": "union-testnet-key0",
                "key": "0xe6b7f3906f38ea3547c91ed2f5eab850d27dd5672424fa4759e471be45598860"
              }
            ]
          },

          "gas_config": {
            "gas_price": "1.0",
            "gas_denom": "muno",
            "gas_multiplier": "1.1",
            "max_gas": 400000
          },
          "fee_denom": "muno",
          "ws_url": "wss://rpc.testnet.bonlulu.uno/websocket",
          "prover_endpoint": "https://galois.testnet-8.union.build:443",
          "grpc_url": "https://grpc.testnet.bonlulu.uno"
        },
        "transfer_module": {
          "type": "contract",
          "address": "union177jpkxrhvzca0dhr7p05ty595ucdgdl6k4wv67500jxcu6t5hppqemdy20"
        }
      }
    }
  },
  "interactions": [
    {
      "source": {
        "chain": "ethereum",
        "channel": "channel-81"
      },
      "destination": {
        "chain": "union",
        "channel": "channel-89"
      },
      "protocol": {
        "Ucs01": {
          "receivers": ["union1qgvmcfkpd66wat6shhfas0z8z9dzp683mcj9tq"],
          "contract": "union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h"
        }
      },
      "memo": "{\"forward\":{\"receiver\":\"614E946f6D769Ad2983E4d4B81DDeBBFA51B09b5\",\"port\":\"wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h\",\"channel\":\"channel-80\"}}",
      "sending_memo_probability": 0,
      "denoms": ["0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14"],
      "send_packet_interval": 1,
      "expect_full_cycle": 900,
      "amount_min": 5000000,
      "amount_max": 10000000,
      "max_retry": 3
    }
  ],
  "single_interaction": {
    "source": {
      "chain": "ethereum",
      "channel": "channel-81"
    },
    "destination": {
      "chain": "union",
      "channel": "channel-89"
    },
    "protocol": {
      "Ucs01": {
        "receivers": ["union1qgvmcfkpd66wat6shhfas0z8z9dzp683mcj9tq"],
        "contract": "union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h"
      }
    },
    "memo": "{\"forward\":{\"receiver\":\"614E946f6D769Ad2983E4d4B81DDeBBFA51B09b5\",\"port\":\"wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h\",\"channel\":\"channel-80\"}}",
    "sending_memo_probability": 0,
    "denoms": ["0x08210F9170F89Ab7658F0B5E3fF39b0E03C594D4"],
    "send_packet_interval": 1,
    "expect_full_cycle": 900,
    "amount_min": 10000000,
    "amount_max": 10000000,
    "max_retry": 3
  }
}

```


## Key Features

### Token Distribution

Sentinel supports token distribution to ensure even allocation among participants:

- **Native Token Distribution:** If the `native_token_distribution` flag is enabled, Sentinel collects native tokens from participants and redistributes them to ensure even balances.
- **ERC20 Token Distribution:** If the `token_distribution` flag is enabled, Sentinel manages ERC20 token distribution. It collects tokens from participants, consolidates them, and redistributes them to maintain healthy balances.

### Transaction Execution

Sentinel handles the execution of transactions between configured chains:

- **Scheduled Transactions:** Transactions are sent periodically as defined in the configuration.
- **Parallel Processing:** Transactions are executed in parallel, improving efficiency and ensuring timely transfers.
- **Configurable Interactions:** Interactions are defined in the configuration file, specifying source and destination chains, channels, protocols, denominations, and transaction parameters like amounts and retry limits.

### Event Listening

By default, Sentinel listens for IBC events on all enabled chains to monitor and verify transaction statuses:

- **Event Types:** Sentinel tracks various IBC event types such as SendPacket, RecvPacket, AcknowledgePacket, and WriteAcknowledgement.
- **Error Handling:** Sentinel logs detailed information about events and handles any errors that occur, providing insights for debugging and ensuring smooth operations.

### Configuration Flexibility

Sentinel's behavior can be customized through configuration settings:

- **Disabling Functionalities:** Listening and interaction functionalities can be disabled via command-line flags.
- **Single Transaction Mode:** Sentinel can perform a single transaction and then terminate, useful for testing and specific use cases.

### Monitoring and Alerts

Sentinel integrates with monitoring and alerting services to ensure reliability and quick issue resolution:

- **Datadog:** Tracks vital metrics and logs related to Sentinel’s operations, such as CPU usage, memory usage, network activity, and transaction statuses.
- **BetterStack Integration:** Provides alerts via email, Slack, and phone calls for critical errors or warnings, ensuring prompt issue resolution.

## Conclusion

Sentinel is a robust solution for automating blockchain interactions, ensuring secure and efficient token transfers between chains. With its comprehensive configuration options, real-time monitoring, and detailed error tracking, Sentinel provides a reliable and scalable service for managing blockchain transactions and IBC events.
