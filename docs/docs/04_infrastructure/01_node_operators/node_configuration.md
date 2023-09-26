---
title: "Node Configuration"
---

After successfully running your node with `uniond`, you can refer to this guide to aid you in configuring your node.

This is not a complete guide to all node configuration options, this is intended to help ensure that your node is fully operational.

This guide will assume that you're starting at the root of your Union node configuration (located at `~/.union/` by default).

## Client Configuration

Located in `config/client.toml`, this file is host to client settings.

### The Network Chain ID

Update this value to ensure that your client is supplied with the correct chain ID.

For the Union Testnet, this value should be `"union-testnet-3"`.

```toml
# The network chain ID
chain-id = "union-testnet-3"
```

### Host/Port for the Tendermint RPC

This will determine which address your client will listen for Tendermint RPC request on.

This will default to `"tcp://localhost:26657"`, setting this to `"tcp://0.0.0.0:26657"` will ensure it's listening on every available network interface.

```toml
# <host>:<port> to Tendermint RPC interface for this chain
node = "tcp://0.0.0.0:26657"
```

:::caution

You should ensure this device is protected from DDoS attacks with a service such as Cloudflare's proxies.

:::

## App Configuration

Located in `config/app.toml`, this file is host to app settings.

### Minimum Gas Price

Located under the "Base Configuration" section of `config/app.toml`.

While optional, you may wish to ensure your node receives a minimum fee when processing transactions. Whatever you choose for this value, ensure it uses the correct denom. For the Union Testnet, the correct denom is `muno`.

```toml
# The minimum gas prices a validator is willing to accept for processing a
# transaction. A transaction's fees must meet the minimum of any denomination
# specified in this config (e.g. 0.25token1;0.0001token2).
minimum-gas-prices = "0muno"
```

### Pruning

Located under the "Base Configuration" section of `config/app.toml`.

Several options are available here, ensure you've selected the one that best fits your nodes storage capabilities.

```toml
# default: the last 362880 states are kept, pruning at 10 block intervals
# nothing: all historic states will be saved, nothing will be deleted (i.e. archiving node)
# everything: 2 latest states will be kept; pruning at 10 block intervals.
# custom: allow pruning options to be manually specified through 'pruning-keep-recent', and 'pruning-interval'
pruning = "default"
```

## Node Configuration

Located in `config/config.toml`, this file is host to many settings.

### P2P Listening Address

Located in the `p2p` TOML group under the "P2P Configuration Options" section.

You'll want to ensure your node is configured to accept p2p connections. To do so, set this value to the appropriate address.

For example, to listen on every available network interface - set this to `"tcp://0.0.0.0:26656"`.

```toml
# Address to listen for incoming connections
laddr = "tcp://0.0.0.0:26656"
```

### External Address

Located in the `p2p` TOML group under the "P2P Configuration Options" section.

If you've configured a domain name for your node, this is the place to inform your node of it.

```toml
# Address to advertise to peers for them to dial
# If empty, will use the same port as the laddr,
# and will introspect on the listener or use UPnP
# to figure out the address. ip and port are required
# example: 159.89.10.97:26656
external_address = "example.com:26656"
```

### Seed Mode

Located in the `p2p` TOML group under the "P2P Configuration Options" section.

If you'd like to be a seed node, be sure to set this to `true`.

```toml
# Seed mode, in which node constantly crawls the network and looks for
# peers. If another node asks it for addresses, it responds and disconnects.
#
# Does not work if the peer-exchange reactor is disabled.
seed_mode = false
```
