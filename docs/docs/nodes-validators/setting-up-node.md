---
title: "Setting up a node"
---

# Setting Up a Node

This document will walk you through the process of self-hosting a Union node.

# Requirements

To be able to send and receive messages in the network, you'll need to conduct TCP port forwarding for ports `26656` & `26657`.

## System Specs

<!-- TODO(unionfi/union#120): document system requirements for validators -->

# Building The `uniond` Binary

This section will walk you through building the node binary. These instructions are different for Docker and Nix.

_NOTE: It's expected that all validators/nodes use Nix or Docker to set up their node. Imperative installations are not officially supported._

## Docker

To run `uniond` with Docker, you can follow the [Docker instructions for running `uniond`](./running-uniond.md#Docker)

## Nix

Run `nix build` in the root of our repository after cloning:

<!-- TODO: Once repo is public, update to:
```sh
nix build "github:UnionFi/union"
```
-->

```sh
nix build "git+ssh://git@github.com/UnionFi/union"
```

_NOTE: the `uniond` executable is stored as `./result/bin/uniond`_

# Environment Variables

This document will often refer to environment variables you likely don't have set. You can either manually replace them before running commands, or set them before starting with the rest of the tutorial.

Here's a list of the environment variables we'll use and hints for setting them:

- ` $CHAIN_ID` - either `union-1` for mainnet, or `union-testnet-1` for testnet.

  ```sh
  # Example $CHAIN_ID
  CHAIN_ID=union-testnet-1
  ```

- ` $MONIKER` - The name used for your validator node.

  ```sh
  # Example $MONIKER
  MONIKER="Unionized Goblin"
  ```

- ` $KEY_NAME` - The name you've assigned to the key pair you'll use for this tutorial.

  ```sh
  # Example $KEY_NAME
  KEY_NAME=alice
  ```

# Connect to the Public RPC

_NOTE: The public RPC is currently not available._

Now to connect the `uniond` binary to the public RPC.

First, set the chain-id with:

```sh
uniond config chain-id $CHAIN_ID
```

Set the public RPC node:

<!-- TODO: Replace `$RPC_NODE_URL` with our RPC node URL. https://github.com/UnionFi/union/issues/30 -->

```sh
uniond config node $RPC_NODE_URL
```

# Setting up the Node

## Initialize the chain

```sh
uniond init $MONIKER "bn254" --chain-id $CHAIN_ID
```

## Download the Genesis File

**`GENESIS_URL` options:**

- Union Testnet: `https://raw.githubusercontent.com/unionfi/genesis/main/union-testnet-1/genesis.json`

<!-- TODO: Create and upload genesis file for users to download. https://github.com/UnionFi/union/issues/31 -->

```sh
curl $GENESIS_URL > ~/.union/config/genesis.json
```

## Configure Seed nodes

To begin connecting to other nodes in the network, you will need to add the seeds nodes for the respective network to your configuration.

To do this, edit the configuration file at `~/.union/config/config.toml`

```toml
#######################################################
###           P2P Configuration Options             ###
#######################################################
[p2p]

# <snip>...

# Comma separated list of seed nodes to connect to
seeds = ""

# <snip>...
```

For the Union Testnet replace `seeds = ""` with:

```toml
# Comma separated list of seed nodes to connect to
seeds = "c649931f0ef98bc3e086bbfbcf3b04896a9ec7de@uniontestnet.poisonphang.com:26656"
```

## Create Local Key Pair

You'll need your own local key pair to participate on the chain. You can either create a new key pair, or import an existing one from your mnemonic seed.

### Create a new Key Pair

Create the new key pair, making sure to note the mnemonic seed and public key:

```sh
uniond keys add $KEY_NAME
```

Ensure the public key and other details match:

```sh
uniond keys show $KEY_NAME -a
```

### Import an Existing Key Pair

Restore existing Union wallet with mnemonic seed phrase:

```sh
uniond keys add $KEY_NAME --recover
```

Ensure the public key and other details match:

```sh
uniond keys show $KEY_NAME -a
```

## Acquire Tokens

<!-- TODO: Determine process for distributing tokens on testnet. https://github.com/UnionFi/union/issues/33 -->

# Start the node

```sh
uniond start
```

At this point, you may want to consider setting up a `systemd` service to run `uniond`.

Once the node is done catching up, you are ready to start the process of becoming a validator.

# Becoming a Validator

## Submitting a `create-validator` Transaction

To become a validator, you must submit a `create-validator` transaction:

```sh
uniond tx staking create-validator \
  --amount $STAKE \
  --pubkey=$(uniond tendermint show-validator) \
  --moniker $MONIKER \
  --chain-id $CHAIN_ID \
  --from $KEY_NAME
```

Where `STAKE` is the amount of stake you're putting down for your validator (i.e. `100000000uuno`).

It's then recommended to backup these files from `~/.union/config` in a secure location:

- `priv_validator_key.json`
- `node_key.json`
