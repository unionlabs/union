# light-client-cli (ethlc)

This crate provides a toy cli application for light client.

## Build

```bash
$ cargo build --bin ethlc
```

## Getting Started

The following instruction is an example that uses mainnet:

### Initialize Light Client store

```shell
$ ethlc --network=mainnet --beacon_endpoint=https://lodestar-mainnet.chainsafe.io init
```

### Update Light Client store (one-shot)

```shell
$ ethlc --network=mainnet --beacon_endpoint=https://lodestar-mainnet.chainsafe.io update
```

### Update Light Client store (forever until killing the process)

```shell
$ ethlc --network=mainnet --beacon_endpoint=https://lodestar-mainnet.chainsafe.io update --target=infinity
```
