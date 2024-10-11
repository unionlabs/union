---
order: 2
---

# Quick Start

## Overview

This is a quick start guide. If you have a vague idea about how CometBFT
works and want to get started right away, continue.

## Install

See the [install guide](install.md).

## Initialization

Running:

```sh
cometbft init
```

will create the required files for a single, local node.

These files are found in `$HOME/.cometbft`:

```sh
$ ls $HOME/.cometbft

config  data

$ ls $HOME/.cometbft/config/

config.toml  genesis.json  node_key.json  priv_validator.json
```

For a single, local node, no further configuration is required.
Configuring a cluster is covered further below.

## Local Node

Start CometBFT with a simple in-process application:

```sh
cometbft node --proxy_app=kvstore
```

> Note: `kvstore` is a non persistent app, if you would like to run an application with persistence run `--proxy_app=persistent_kvstore`

and blocks will start to stream in:

```sh
I[01-06|01:45:15.592] Executed block                               module=state height=1 validTxs=0 invalidTxs=0
I[01-06|01:45:15.624] Committed state                              module=state height=1 txs=0 appHash=
```

Check the status with:

```sh
curl -s localhost:26657/status
```

### Sending Transactions

With the KVstore app running, we can send transactions:

```sh
curl -s 'localhost:26657/broadcast_tx_commit?tx="abcd"'
```

and check that it worked with:

```sh
curl -s 'localhost:26657/abci_query?data="abcd"'
```

We can send transactions with a key and value too:

```sh
curl -s 'localhost:26657/broadcast_tx_commit?tx="name=satoshi"'
```

and query the key:

```sh
curl -s 'localhost:26657/abci_query?data="name"'
```

where the value is returned in hex.

## Cluster of Nodes

First create four Ubuntu cloud machines. The following was tested on Digital
Ocean Ubuntu 16.04 x64 (3GB/1CPU, 20GB SSD). We'll refer to their respective IP
addresses below as IP1, IP2, IP3, IP4.

Then, `ssh` into each machine and install CometBFT following the [instructions](install.md).

Next, use the `cometbft testnet` command to create four directories of config files (found in `./mytestnet`) and copy each directory to the relevant machine in the cloud, so that each machine has `$HOME/mytestnet/node[0-3]` directory.

Before you can start the network, you'll need peers identifiers (IPs are not enough and can change). We'll refer to them as ID1, ID2, ID3, ID4.

```sh
cometbft show_node_id --home ./mytestnet/node0
cometbft show_node_id --home ./mytestnet/node1
cometbft show_node_id --home ./mytestnet/node2
cometbft show_node_id --home ./mytestnet/node3
```

Here's a handy Bash script to compile the persistent peers string, which will
be needed for our next step:

```bash
#!/bin/bash

# Check if the required argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <ip1> <ip2> <ip3> ..."
    exit 1
fi

# Command to run on each IP
BASE_COMMAND="cometbft show_node_id --home ./mytestnet/node"

# Initialize an array to store results
PERSISTENT_PEERS=""

# Iterate through provided IPs
for i in "${!@}"; do
    IP="${!i}"
    NODE_IDX=$((i - 1))  # Adjust for zero-based indexing

    echo "Getting ID of $IP (node $NODE_IDX)..."

    # Run the command on the current IP and capture the result
    ID=$($BASE_COMMAND$NODE_IDX)

    # Store the result in the array
    PERSISTENT_PEERS+="$ID@$IP:26656"

    # Add a comma if not the last IP
    if [ $i -lt $# ]; then
        PERSISTENT_PEERS+=","
    fi
done

echo "$PERSISTENT_PEERS"
```

Finally, from each machine, run:

```sh
cometbft node --home ./mytestnet/node0 --proxy_app=kvstore --p2p.persistent_peers="ID1@IP1:26656,ID2@IP2:26656,ID3@IP3:26656,ID4@IP4:26656"
cometbft node --home ./mytestnet/node1 --proxy_app=kvstore --p2p.persistent_peers="ID1@IP1:26656,ID2@IP2:26656,ID3@IP3:26656,ID4@IP4:26656"
cometbft node --home ./mytestnet/node2 --proxy_app=kvstore --p2p.persistent_peers="ID1@IP1:26656,ID2@IP2:26656,ID3@IP3:26656,ID4@IP4:26656"
cometbft node --home ./mytestnet/node3 --proxy_app=kvstore --p2p.persistent_peers="ID1@IP1:26656,ID2@IP2:26656,ID3@IP3:26656,ID4@IP4:26656"
```

Note that after the third node is started, blocks will start to stream in
because >2/3 of validators (defined in the `genesis.json`) have come online.
Persistent peers can also be specified in the `config.toml`. See [here](../explanation/core/configuration.md) for more information about configuration options.

Transactions can then be sent as covered in the single, local node example above.
