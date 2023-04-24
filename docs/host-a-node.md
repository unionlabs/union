# Hosting a Node

This document will walk you through the process of self-hosting a union node.

# Building The `uniond` Binary

This section will walk you through building the node binary. These instructions are different for Docker and Nix.

*NOTE: It's expected that all validators/nodes use Nix or Docker to set up their node. Imperative installations are not officially supported.*

## Docker

<!-- TODO: Add docker instructions following PR -->

## Nix

Run `nix build` in the root of our repository after cloning:

```sh
git clone https://github.com/UnionFi/union.git
cd union
nix build
```

# Connect to the Public RPC

*NOTE: The public RPC is currently not available.*

Now to connect the `uniond` binary to the public RPC.

First, set the chain-id with:

<!-- TODO: Replace `$CHAIN_ID` with our the chain-id of our main-net or test-net. -->
```sh
uniond config chain-id $CHAIN_ID
```

Set the public RPC node:

<!-- TODO: Replace `$RPC_NODE_URL` with our RPC node URL. -->
```sh
uniond config node $RPC_NODE_URL
```

# Setting up the Node

## Initialize the chain

```sh
uniond init <$YOUR_MONIKER_NAME> --chain-id $CHAIN_ID
```

## Download the Genesis File

<!-- TODO: Create and upload genisis file for users to download -->
```sh
curl $GENISIS_URL > ~/.union/config/genesis.json
```

## Configure Persistent Peers

<!-- TODO: Create and upload presistent peers linst for users to download -->
<!-- TODO: Update instructions -->

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

<!-- TODO: Determine process for distributing tokens on testnet -->

# Setup cosmovisor