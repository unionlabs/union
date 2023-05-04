---
title: 'Creating the Testnet Genesis'
---

<!--
Spelling ignore list
cspell:ignore nunion
cspell:ignore wupuxf
cspell:ignore thrynh
 -->

# Creating the Testnet Genesis

This document details the steps necessary to create a genesis file, specifically for the testnet.

There are two main sets of instructions, those for the host node that will compile the final genesis file and those for the other genesis nodes that will be part of the genesis validator set.

# Key Generation

Both the host node and genesis validators should generate an app key pair.

## App Key

To add your app key, you can use the `uniond keys` interface to either create a new key pair or import an existing key pair.

*Note: The variable ` $KEY_NAME` is to be replaced by whatever name you desire to locally refer to your app key by.*

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

# Genesis Configuration Options

Before anything else, an initial genesis file must be created with some basic configurations.

To create the initial genesis file at `~/.uniond/config/genesis.json`, both the host node and genesis validators should run:

```sh
uniond init $MONIKER bn254
```

This creates a new genesis file with the current time as the `genesis_time`.

At this point, you can move your consensus key from earlier to `~/.uniond/config/priv_validator_key.json`.

## Host Node

Now, configuration of the genesis file can begin.

### Tokens

Before configuring other options of the genesis file, it's important to define the genesis tokens that will be present in our testnet.

For example, we could define `nunion` (nano Union) as having one billion indivisible units. We then replace instances of `stake` with `nunion` in `~/.uniond/config/genesis.json`

### Genesis Accounts

Genesis accounts and their balances will need to be added to the `genesis.json`. This can be done manually, or with the `uniond add-genesis-account` command.

Each genesis validator will need to be added in this step. This means you should know the address and balance of each validator before this point. By default, a minimum stake of `10000000` is required for validators.

For example, to add Alice with `128000000000000` `nunion` you would run:

```sh
uniond add-genesis-account union1wupuxf8sqz3h24kzd90thrynh9w0ry7y9eu3jc 128000000000000nunion
```

### Other Options

The `genesis.json` file has many options for configuration. A good summary of what's available can be found in Cosmos Hub's [Genesis File](https://hub.cosmos.network/main/resources/genesis.html) documentation.

Most notably, the sections:

* [Mint](https://hub.cosmos.network/main/resources/genesis.html#mint)
* [Staking](https://hub.cosmos.network/main/resources/genesis.html#staking)
* [Slashing](https://hub.cosmos.network/main/resources/genesis.html#slashing)

### Provide Genesis Validators Proto-Config

At this point, the host node should supply the current `genesis.json` to the genesis validators so that they can create their genesis transactions.

## Genesis Validators

Before the host node is able to produce the final genesis config, each validator needs to produce a genesis transaction and supply it to the host node.

### Generating a Genesis Transaction

Before continuing, replace `~/.uniond/config/genesis.json` with the proto-config supplied by the host node. Also ensure you have replaced `~/.uniond/config/priv_validator_key.json` with the `bn254` key pair you generated earlier.

To generate the genesis transaction, we will use `uniond gentx`.

The following values should be defined as follows:

* `KEY_NAME` Name of the app key you created/restored earlier.

* `STAKE` The stake you will put down to be a validator (`10000000 <= STAKE < genesis_balance`).

* `PUBKEY` the value returned from `uniond tendermint show-validator`

* `MONIKER` Your moniker ID.

```sh
uniond gentx $KEY_NAME $STAKE "bn254" --chain-id union-testnet-1 --pubkey $PUBKEY --moniker $MONIKER
```

This should create a `.json` file under `~/.uniond/config/gentx/`. This `.json` file should be provided to the host node.

# Finalizing the Genesis Configuration

The final step to produce a genesis file is to aggregate the genesis transactions created by the genesis validators.

## Host Node

The host node will be responsible for aggregating the transactions and dispersing the final genesis file.

### Transaction Aggregation

Once the host node has all the genesis transaction files, they need to be moved to `~/.uniond/config/gentx`.

Once the transaction files are all together, you can then run:

```sh
uniond collect-gentxs
```

This completes the creation of the genesis configuration.

### Dispersing the Genesis Configuration

Now the time has come to ensure each of the genesis validators have the final genesis file.

We have a repository for this at [unionfi/genesis](https://github.com/unionfi/genesis).

Here we should create a folder for the testnet and copy our genesis file into it.

```sh
mkdir union-testnet
cp ~/.uniond/config/genesis.json union-testnet/
```

After you have contributed the genesis file, genesis validators will be able to obtain the final genesis config.


## Genesis Validators

The last step for genesis validators is to obtain the genesis configuration and overwrite their current one with it.

### Overwriting the Genesis Configuration

To do this, run:

```sh
curl https://raw.githubusercontent.com/unionfi/genesis/main/union-testnet/genesis.json > ~/.union/config/genesis.json
```

This concludes the instructions for creating and sharing the genesis file.
