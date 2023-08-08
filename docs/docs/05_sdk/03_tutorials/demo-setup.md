---
title: "Devnet Demo"
---

# Union Devnet Demo

The demo setup is used to run every component there is locally to be able to experiment with IBC between Ethereum and Union. The nice thing about the setup is that, it requires you to run only two commands and it will handle the rest. This document will explain every option to do the setup for your needs.

## Quick start

Start with cloning the repository if you haven't already done it:

```bash
git clone git@github.com:unionlabs/union.git
```

Before running the setup command, we need to start both chains. The following command will start:

1. Union with 4 validators.
2. Ethereum with the [the minimal preset](https://github.com/ethereum/consensus-specs/blob/v1.3.0/configs/minimal.yaml).

```bash
nix run ".#devnet"
```

The above command will run both of the chains in a single session. So if you want to have separate logs for Ethereum and Union, you can run both of the devnets separately:

```bash
# For evm
nix run ".#devnet-evm"

# For union
nix run ".#devnet-cosmos"
```

Now that we are ready, we can run the setup:

```bash
nix run ".#setup-demo" -- --circuit-path ./
```

`--circuit-path` is passed for the prover to look for and download the files that it needs.

The setup might take some time based on your internet speed and computing power. But note that if you are going to run the prover locally, we recommend you to have at least 32 GB memory. If you don't have that much memory but are able to run the prover in a remote server or computer, we'll explain how to use a remote prover later in this document.

## What does this setup do exactly

There are plenty of things to make IBC work. To explain it briefly, the steps are:

1. If the prover is being run locally, it will first look for the files under `--circuit-path` and download or update them if necessary.
2. It checks whether both of the chains are alive.
3. It instantiates the following IBC apps to Union for you to try:
   1. `cw20-ics20`: For cross-chain token transfers.
   2. `ping-pong`: For starting endless ping-pong between Ethereum and Union.
4. It deploys the IBC protocol contracts as well as `ICS20Transfer` contract for cross-chain transfers to Ethereum.
5. It deploys the ping-pong contract on Ethereum.
6. It starts the local prover in the background if the `--circuit-path` is provided.
7. It sets up the initial channels for both `ICS20Transfer` and `PingPong` on Ethereum. Note that this is not being done in Union since this is handled at the genesis configuration.
8. It creates light clients on both chains.
9. It waits for the prover to be online, and then it starts the relayer to relay packets.

## CLI options to customize the execution

### Using an already running prover

This can be very handy because starting the prover locally takes time and memory. One possible use case here is deploying the prover on a remote server, on a computer within the same network or if you have enough memory you could also run the prover locally and run the demo over and over again. But note that deploying Ethereum contracts takes a lot of memory as well so 32 GB might not be enough to run both in parallel.

To run the prover:

```bash
nix run ".#galoisd-devnet" -- serve 0.0.0.0:16657
```

For using a prover where the TLS is not enabled:

```bash
nix run ".#setup-demo" -- --galois-url http://myserver.com:1111
```

To enable the TLS:

```bash
nix run ".#setup-demo" -- --galois-url http://myserver.com:1111 --galois-tls
```

Note that even if you want to run Galois locally but on a different endpoint, you can provide a local endpoint and `--circuit-path`. The `--circuit-path` parameter is the decision point on whether to run Galois locally or not.

### Using custom endpoints for the devnets

You might be running the devnet on a remote server, you can change the following options as you wish:

```text
--evm-beacon-rpc-url         Rpc endpoint for the evm beacon chain. (Default: http://localhost:9596)
--evm-ws-url                 Websocket endpoint for the evm execution chain (Default: ws://localhost:8546).
--union-rpc-url              Rpc endpoint for union (Default: http://localhost:26657).
--union-ws-url               Websocket endpoint for union (Default: ws://localhost:26657/websocket).
```

Run `--help` to see the description for yourself.

### Using a custom relayer configuration file

Our relayer uses a configuration file for the IBC contracts that are deployed, chain endpoints, etc. In a normal execution, a config file is created for you in a temporary location and this location is printed to the console after the setup is complete, so that you can use the relayer manually if you want.

To change this relayer configuration path, run:

```bash
nix run ".#setup-demo" -- --relayer-config-file /path/to/config.json
```

### Customize ping-pong setup

Our ping-pong contracts have the timeout argument which is the number of blocks before `ping` message times out. By default, this is `1000`. To change this, run:

```bash
nix run ".#setup-demo" -- --ping-pong-timeout 100
```
