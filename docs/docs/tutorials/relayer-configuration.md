---
title: "Configuring the Relayer"
---

Our relayer uses a configuration file so you don't have to type the same parameters on each execution.
By default, the relayer uses `~/.config/relayer/config.json`. You can use `--config-file-path` to use a location of your choice.

# Creating the relayer configuration

You can download the example configuration file from [here](https://github.com/unionlabs/union/blob/main/relayer-config.json),
or use the following command to download it:

```bash
wget https://raw.githubusercontent.com/unionlabs/union/main/relayer-config.json
```

Here is an example configuration file:

```json
{
  "chain": {
    "ethereum-devnet": {
      "chain_type": "evm",
      "preset_base": "minimal",
      "cometbls_client_address": "0x774667629726ec1fabebcec0d9139bd1c8f72a23",
      "ibc_handler_address": "0xfc97a6197dc90bef6bbefd672742ed75e9768553",
      "ics20_transfer_bank_address": "0xf8f7758fbcefd546eaeff7de24aff666b6228e73",
      "ics20_bank_address": "0x83428c7db9815f482a39a1715684dcf755021997",
      "signer": {
        "raw": "0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
      },
      "eth_rpc_api": "ws://localhost:8546",
      "eth_beacon_rpc_api": "http://localhost:9596",
      "wasm_code_id": "0x44ab11c49e3aa40301679bff45818117158d6cc8c326a5bafa69089709c3d13e"
    },
    "union-devnet": {
      "chain_type": "union",
      "signer": {
        "raw": "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
      },
      "ws_url": "ws://127.0.0.1:26657/websocket",
      "wasm_code_id": "0x44ab11c49e3aa40301679bff45818117158d6cc8c326a5bafa69089709c3d13e",
      "prover_endpoint": "http://0.0.0.0:16657"
    }
  }
}
```

## Ethereum configurations

To see the correct values of the following parameters, check out [the IBC configuration parameters documentation](../configurations/ibc-parameters).
- `chain_type`: `evm`
- `preset_base`: `minimal` or `mainnet` based on the target Ethereum chain.
- `xxx_address`: Contract addresses that are belong to the IBC module on Ethereum.
- `signer.raw`: Private key of the signer of the transactions to Ethereum in hexadecimal format.
- `eth_rpc_api`: RPC endpoint of the Ethereum's execution layer.
- `eth_beacon_rpc_api`: RPC endpoint of the Ethereum's beacon api.
- `wasm_code_id`: Code ID of the Ethereum light client on Union.

## Union configurations

To see the correct values of the following parameters, check out [the IBC configuration parameters documentation](../configurations/ibc-parameters).
- `chain_type`: `union`
- `signer.raw`: Private key of the signer of the transactions to Union in hexadecimal format.
- `ws_url`: Websocket endpoint of Union.
- `wasm_code_id`: Code ID of the Ethereum light client on Union.
- `prover_endpoint`: The endpoint of `Galois`.