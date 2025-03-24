



# Union U-Combinator Workshop 1 Cheatsheet

## Walkthrough

1. Start Union devnet.
```
nix run .#devnet-union
```

2. Start Ethereum devnet.
```
nix run .#devnet-eth
```

3. Deploy CosmWasm IBC stack.
```
 nix run .#cosmwasm-scripts.deploy-full-union-devnet
```

4. Start Galois
```
nix run .#galoisd-testnet-standalone -- serve localhost:9999
```

5. Start voyager queue (PSQL db)
(Note that if you don't want to run this, and want to point voyager to an already running PSQL, make sure to create a DB and point voyager to that URL)
```
nix run .#voyager-queue
```

6. Start voyager
```
cargo b "voyager*" && RUST_LOG=voyager=info ./target/debug/voyager -c voyager/config.jsonc start
```

7. Fetch blocks
From Union:
```
./target/debug/voyager init-fetch union-devnet-1 --config-file-path voyager/config.jsonc -e
```

From Eth:
```
./target/debug/voyager init-fetch 32382 --config-file-path voyager/config.jsonc -e
```

8. Create CometBLS light client on Ethereum:
```
./target/debug/voyager -c voyager/config.jsonc msg create-client --tracking union-devnet-1 --on 32382 --ibc-interface ibc-solidity --ibc-spec-id ibc-union --client-type cometbls -e
```
- You will see the client id in the voyager logs, similar to this:
```
2025-03-24T13:10:44.372981Z  INFO voyager_vm: received data outside of an aggregation data={"@type":"ibc_event","@value":{"chain_id":"union-devnet-1","client_info":{"client_type":"ethereum","ibc_interface":"ibc-cosmwasm"},"counterparty_chain_id":"32382","tx_hash":"0x7b0f81b9246bb2dca0a2171e8f608f885ae3f61a637027537646cfafbcea0978","provable_height":"1-284","ibc_spec_id":"ibc-union","event":{"@type":"create_client","@value":{"client_id":1,"client_type":"ethereum"}}}}
```

9. Create Ethereum light client on Union:
```
./target/debug/voyager -c voyager/config.jsonc msg create-client --on union-devnet-1 --tracking 32382 --ibc-interface ibc-cosmwasm --ibc-spec-id ibc-union --client-type ethereum -e
```
- You can watch the transactions in Eth explorer running [locally](http://localhost). All voyager transactions call `multicall` method, and you will see the events in `Logs` section in a transaction detail.

10. Connection open:

- Change the client id's if necessary. The handshake starts from the chain with `chain_id`.
```
./target/debug/voyager -c voyager/config.jsonc  q e '{"@type":"call","@value":{"@type":"submit_tx","@value":{"chain_id":"union-devnet-1","datagrams":[{"ibc_spec_id":"ibc-union","datagram":{"@type":"connection_open_init","@value":{"client_id":1,"counterparty_client_id":1}}}]}}}'
```

- You can follow the transactions on the Eth block explorer. You will again see the events in the `Logs` section. When you see `ConnectionOpenConfirm` event, it means the connection is open.

11. Channel open:

- To do this, first fetch the evm addresses:
```
nix run .#evm-contracts-addresses -- 0x86D9aC0Bab011917f57B9E9607833b4340F9D4F8 0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD http://localhost:8545
```

- The output will be the following:
```
== Logs ==
  Multicall: 0x9fd9d9528c8373d990a1380b9414bde179007a35
  IBCHandler: 0xed2af2ad7fe0d92011b26a2e5d1b4dc7d12a47c5
  CometblsClient: 0xc4f27a952faba4174ce0ee6d9d0c6f4c41524d49
  StateLensIcs23MptClient: 0xfc72a169dd9d9c6ac9e46d89b8a450af13cede7e
  StateLensIcs23Ics23Client: 0xd1557410850d155a065c34dea61d906ed0d2e25c
  StateLensIcs23SmtClient: 0x47ec6840eb2a3c425334e35469e82bc3e25fd558
  UCS00: 0x21bd17aec8ceb789d3145a606968dcc428c1e4f4
  UCS03: 0x05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5
```

- We will use `UCS03` address. This is the ZKGM contract deployed on EVM.

- For the `UCS03` address on Union, check out the chain id "union-devnet-1" in `deployments/deployments-testnet-9.json`. You will see `ucs03` address
under `app`. You can convert this address to hex easily by running the following command:
```
printf "union162nwv92cfwthfcnlqmh30gmnem2u3uv56ap9y7e0vuqtdnz2qjeq2ckrsf" | xxd -c 0 -ps
```

- Now we do channel handshake:
- Few notes:
  - Do not forget to change the connection id according to your setup. This connection id will be the connection id on `Union` since we are starting the handshake on `Union`.
  - `port_id` is the hexified address of the `ucs03 (zkgm)` contract on `Union`.
  - `counterparty_port_id` is the `ucs03 (zkgm)` contract on `EVM` as is. 
```
./target/debug/voyager -c ../voyager-config-testnet-10.jsonc q e '{"@type":"call","@value":{"@type":"submit_tx","@value":{"chain_id":"union-devnet-1","datagrams":[{"ibc_spec_id":"ibc-union","datagram":{"@type":"channel_open_init","@value":{"port_id":"0x756e696f6e3136326e77763932636677746866636e6c716d683330676d6e656d327533757635366170397937653076757174646e7a32716a657132636b727366","counterparty_port_id":"0x05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5","connection_id":1,"version":"ucs03-zkgm-0"}}}]}}}'
```

- Follow the transactions and logs again to see the `channel_id`.



