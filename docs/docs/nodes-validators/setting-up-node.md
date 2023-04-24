---
title: 'Setting up a node'
---

# Setting Up a Node

This document will walk you through the process of self-hosting a Union node.

# Building The `uniond` Binary

This section will walk you through building the node binary. These instructions are different for Docker and Nix.

*NOTE: It's expected that all validators/nodes use Nix or Docker to set up their node. Imperative installations are not officially supported.*

## Docker

<!-- TODO: Add docker instructions following PR -->

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

*NOTE: the `uniond` executable is stored as `./result/bin/uniond`*

# Environment Variables

This document will often refer to environment variables you likely don't have set. You can either manually replace them before running commands, or set them before starting with the rest of the tutorial.

Here's a list of the environment variables we'll use and hints for setting them:

* ` $CHAIN_ID` - either `union-1` for mainnet, or `union-test-1` for testnet.

  ```sh
  # Example $CHAIN_ID
  CHAIN_ID=union-test-1
  ```

* ` $MONIKER_NAME` - The name used for your validator node.

  ```sh
  # Example $MONIKER_NAME
  MONIKER_NAME="Unionized Goblin"
  ```

* ` $KEY_NAME` - The name you've assigned to the key pair you'll use for this tutorial.

  ```sh
  # Example $KEY_NAME
  KEY_NAME=some_key
  ```

# Connect to the Public RPC

*NOTE: The public RPC is currently not available.*

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
uniond init $MONIKER_NAME --chain-id $CHAIN_ID
```

## Download the Genesis File

<!-- TODO: Create and upload genesis file for users to download. https://github.com/UnionFi/union/issues/31 -->
```sh
curl $GENESIS_URL > ~/.union/config/genesis.json
```

## Configure Persistent Peers

<!-- TODO: Create and upload persistent peers list for users to download. https://github.com/UnionFi/union/issues/32 -->
<!-- TODO: Update instructions. https://github.com/UnionFi/union/issues/32 -->

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

You can query the status of the node with:

```sh
curl http://localhost:26657/status | jq .result.sync_info.catching_up
```

If this returns `true` the node is still syncing, once it returns `false` you can move to becoming a validator.

# Become a Validator

To become a validator, you must submit a `create-validator` transaction:

```sh
uniond tx staking create-validator \
  --amount 9000000uunionx \
  --commission-max-change-rate "0.1" \
  --commission-max-rate "0.20" \
  --commission-rate "0.1" \
  --min-self-delegation "1" \
  --details "" \
  --pubkey=$(uniond tendermint show-validator) \
  --moniker $MONIKER_NAME \
  --chain-id $CHAIN_ID \
  --gas-prices 0.025uunionx \
  --from $KEY_NAME
```

It's then recommended to backup these files from `~/.union/config`:

* `priv_validator_key.json`
* `node_key.json`
