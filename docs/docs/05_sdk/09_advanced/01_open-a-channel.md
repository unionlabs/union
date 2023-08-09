---
title: "Opening a channel"
---

Creating IBC capable apps on different chains is not enough to make them talk to each other. You first need to create a channel
and optionally, a connection between them. It's optional because you can reuse a connection to open a channel if there is already
a client on both chains that are already connected to each other. But in this tutorial, we'll guide you through all the steps required
to make two modules be able to talk to each other through IBC from scratch.

# Creating clients

As per IBC, you need to have a [client](https://github.com/cosmos/ibc/blob/main/spec/core/ics-002-client-semantics/README.md) to verify and track the counterparty chain's state. This client has a key role on verifying whether the IBC module on the counterparty chain committed a package as is.

Run the following to create a client on `CHAIN-A`:

```bash
relayer client create CHAIN-A CLIENT-A --on CHAIN-A-ID --counterparty CHAIN-B-ID ADDITIONAL-ARGS..
```

After a successful execution, the relayer will print the ID of the client on console. Don't lose it, since we are going to use it in later steps.

Note that `--on` and `--counterparty` are the keys that will be used for reading the chain configuration. For example, if we have the following relayer configuration:

```json
{
	"chain": {
		"ethereum-devnet": { .. },
		"union-devnet": { .. }
	}
}
```

To create an Ethereum client on `union-devnet` that verifies `ethereum-devnet`, you would run:

```bash
relayer client create union ethereum-08-wasm --on union-devnet --counterparty ethereum-devnet --evm-preset minimal
```

Note that `--evm-preset` can differ based on which Ethereum chain you are verifying. Please refer to [the relayer configuration documentation](04_infrastructure/02_relayers/relayer.md) to learn more.

Please run `relayer client create --help` to see all options.

Now that we have a client on `CHAIN-A` that verifies `CHAIN-B`, we also need to have a client on `CHAIN-B` that verifies `CHAIN-A`:

```bash
relayer client create CHAIN-B CLIENT-B --on CHAIN-B-ID --counterparty CHAIN-B-ID ADDITIONAL-ARGS..
```

For example, to create a CometBLS client on `ethereum-devnet` that verifies `union-devnet`, you would run:

```bash
relayer client create evm cometbls --on ethereum-devnet --counterparty union-devnet
```

# Connection Handshake

Next step is to create an [IBC connection](https://github.com/cosmos/ibc/blob/main/spec/core/ics-003-connection-semantics/README.md) between our clients. Note that this connection is reusable, so you can create many channels on it for different modules.

To do a connection handshake, run:

```bash
relayer connection open --from-chain CHAIN-A-ID --from-client CLIENT-A-ID --to-chain CHAIN-B-ID --to-client CLIENT-B-ID
```

For example, to do a handshake between the client `08-wasm-1` on `union-devnet` and the client `cometbls-1` on `ethereum-devnet`, you would run:

```bash
relayer connection open --from-chain union-devnet --from-client 08-wasm-1 --to-chain ethereum-devnet --to-client cometbls-1
```

After a successful execution, the connection id on both chains will be printed. We'll use those id's when creating a channel.

# Channel Handshake

The final step for connecting two IBC modules is to create an [IBC channel](https://github.com/cosmos/ibc/blob/main/spec/core/ics-004-channel-and-packet-semantics/README.md). Note that channels are specific to a single module on each chain.

To do a channel handshake, run:

```bash
relayer channel open --from-chain CHAIN-A-ID --from-connection CONNECTION-A-ID --from-port PORT-A-ID --to-chain CHAIN-B-ID --to-connection CONNECTION-B-ID --to-port PORT-B-ID
```

`port` is the port that the IBC app on your chain is bound to. This has to be done prior to opening a channel. One notable thing here is if you are using a CosmWasm contract as an IBC app, you don't need to do this because it is being done automatically when you instantiate an IBC contract. Please refer to the [CosmWasm IBC documentation](https://github.com/CosmWasm/cosmwasm/blob/main/IBC.md) for more.

As an example to channel handshake, if we want to do a handshake between the port `wasm.CONTRACT_ADDRESS` by using the connection `connection-1` on `union-devnet` and the port `transfer` by using the connection `connection-2` on `ethereum-devnet`, you would run.

```bash
relayer channel open --from-chain union-devnet --from-connection connection-1 --from-port wasm.CONTRACT_ADDRESS --to-chain ethereum-devnet --to-connection connection-2 --to-port transfer
```

After a successful execution, channel id's on both ends will be printed to the console.
