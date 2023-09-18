---
sidebar_position: 1
---

# Running a Node

:::note Synopsis
Now that the application is ready and the keyring populated, it's time to see how to run the blockchain node. In this section, the application we are running is called [`simapp`](https://github.com/cosmos/cosmos-sdk/tree/main/simapp), and its corresponding CLI binary `simd`.
:::

:::note Pre-requisite Readings

* [Anatomy of a Cosmos SDK Application](../basics/00-app-anatomy.md)
* [Setting up the keyring](./00-keyring.md)

:::

## Initialize the Chain

:::warning
Make sure you can build your own binary, and replace `simd` with the name of your binary in the snippets.
:::

Before actually running the node, we need to initialize the chain, and most importantly its genesis file. This is done with the `init` subcommand:

```bash
# The argument <moniker> is the custom username of your node, it should be human-readable.
simd init <moniker> --chain-id my-test-chain
```

The command above creates all the configuration files needed for your node to run, as well as a default genesis file, which defines the initial state of the network. All these configuration files are in `~/.simapp` by default, but you can overwrite the location of this folder by passing the `--home` flag.

The `~/.simapp` folder has the following structure:

```bash
.                                   # ~/.simapp
  |- data                           # Contains the databases used by the node.
  |- config/
      |- app.toml                   # Application-related configuration file.
      |- config.toml                # CometBFT-related configuration file.
      |- genesis.json               # The genesis file.
      |- node_key.json              # Private key to use for node authentication in the p2p protocol.
      |- priv_validator_key.json    # Private key to use as a validator in the consensus protocol.
```

## Updating Some Default Settings

If you want to change any field values in configuration files (for ex: genesis.json) you can use `jq` ([installation](https://stedolan.github.io/jq/download/) & [docs](https://stedolan.github.io/jq/manual/#Assignment)) & `sed` commands to do that. Few examples are listed here.

```bash
# to change the chain-id
jq '.chain_id = "testing"' genesis.json > temp.json && mv temp.json genesis.json

# to enable the api server
sed -i '/\[api\]/,+3 s/enable = false/enable = true/' app.toml

# to change the voting_period
jq '.app_state.gov.voting_params.voting_period = "600s"' genesis.json > temp.json && mv temp.json genesis.json

# to change the inflation
jq '.app_state.mint.minter.inflation = "0.300000000000000000"' genesis.json > temp.json && mv temp.json genesis.json
```

### Client Interaction

When instantiating a node, GRPC and REST are defaulted to localhost to avoid unknown exposure of your node to the public. It is recommended to not expose these endpoints without a proxy that can handle load balancing or authentication is setup between your node and the public. 

:::tip
A commonly used tool for this is [nginx](https://nginx.org).
:::


## Adding Genesis Accounts

Before starting the chain, you need to populate the state with at least one account. To do so, first [create a new account in the keyring](./00-keyring.md#adding-keys-to-the-keyring) named `my_validator` under the `test` keyring backend (feel free to choose another name and another backend).

Now that you have created a local account, go ahead and grant it some `stake` tokens in your chain's genesis file. Doing so will also make sure your chain is aware of this account's existence:

```bash
simd genesis add-genesis-account $MY_VALIDATOR_ADDRESS 100000000000stake
```

Recall that `$MY_VALIDATOR_ADDRESS` is a variable that holds the address of the `my_validator` key in the [keyring](./00-keyring.md#adding-keys-to-the-keyring). Also note that the tokens in the Cosmos SDK have the `{amount}{denom}` format: `amount` is is a 18-digit-precision decimal number, and `denom` is the unique token identifier with its denomination key (e.g. `atom` or `uatom`). Here, we are granting `stake` tokens, as `stake` is the token identifier used for staking in [`simapp`](https://github.com/cosmos/cosmos-sdk/tree/main/simapp). For your own chain with its own staking denom, that token identifier should be used instead.

Now that your account has some tokens, you need to add a validator to your chain. Validators are special full-nodes that participate in the consensus process (implemented in the [underlying consensus engine](../intro/02-sdk-app-architecture.md#cometbft)) in order to add new blocks to the chain. Any account can declare its intention to become a validator operator, but only those with sufficient delegation get to enter the active set (for example, only the top 125 validator candidates with the most delegation get to be validators in the Cosmos Hub). For this guide, you will add your local node (created via the `init` command above) as a validator of your chain. Validators can be declared before a chain is first started via a special transaction included in the genesis file called a `gentx`:

```bash
# Create a gentx.
simd genesis gentx my_validator 100000000stake --chain-id my-test-chain --keyring-backend test

# Add the gentx to the genesis file.
simd genesis collect-gentxs
```

A `gentx` does three things:

1. Registers the `validator` account you created as a validator operator account (i.e. the account that controls the validator).
2. Self-delegates the provided `amount` of staking tokens.
3. Link the operator account with a CometBFT node pubkey that will be used for signing blocks. If no `--pubkey` flag is provided, it defaults to the local node pubkey created via the `simd init` command above.

For more information on `gentx`, use the following command:

```bash
simd genesis gentx --help
```

## Configuring the Node Using `app.toml` and `config.toml`

The Cosmos SDK automatically generates two configuration files inside `~/.simapp/config`:

* `config.toml`: used to configure the CometBFT, learn more on [CometBFT's documentation](https://docs.cometbft.com/v0.37/core/configuration),
* `app.toml`: generated by the Cosmos SDK, and used to configure your app, such as state pruning strategies, telemetry, gRPC and REST servers configuration, state sync...

Both files are heavily commented, please refer to them directly to tweak your node.

One example config to tweak is the `minimum-gas-prices` field inside `app.toml`, which defines the minimum gas prices the validator node is willing to accept for processing a transaction. Depending on the chain, it might be an empty string or not. If it's empty, make sure to edit the field with some value, for example `10token`, or else the node will halt on startup. For the purpose of this tutorial, let's set the minimum gas price to 0:

```toml
 # The minimum gas prices a validator is willing to accept for processing a
 # transaction. A transaction's fees must meet the minimum of any denomination
 # specified in this config (e.g. 0.25token1;0.0001token2).
 minimum-gas-prices = "0stake"
```

## Run a Localnet

Now that everything is set up, you can finally start your node:

```bash
simd start
```

You should see blocks come in.

The previous command allow you to run a single node. This is enough for the next section on interacting with this node, but you may wish to run multiple nodes at the same time, and see how consensus happens between them.

The naive way would be to run the same commands again in separate terminal windows. This is possible, however in the Cosmos SDK, we leverage the power of [Docker Compose](https://docs.docker.com/compose/) to run a localnet. If you need inspiration on how to set up your own localnet with Docker Compose, you can have a look at the Cosmos SDK's [`docker-compose.yml`](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/docker-compose.yml).

## Logging

Logging provides a way to see what is going on with a node. By default the info level is set. This is a global level and all info logs will be outputted to the terminal. If you would like to filter specific logs to the terminal instead of all, then setting `module:log_level` is how this can work. 

Example: 

In config.toml:

```toml
log_level: "state:info,p2p:info,consensus:info,x/staking:info,x/ibc:info,*error"
```

## State Sync

State sync is the act in which a node syncs the latest or close to the latest state of a blockchain. This is useful for users who don't want to sync all the blocks in history. Read more in [CometBFT documentation](https://docs.cometbft.com/v0.37/core/state-sync).

State sync works thanks to snapshots. Read how the SDK handles snapshots [here](https://github.com/cosmos/cosmos-sdk/blob/825245d/store/snapshots/README.md).

### Local State Sync

Local state sync work similar to normal state sync except that it works off a local snapshot of state instead of one provided via the p2p network. The steps to start local state sync are similar to normal state sync with a few different designs. 

1. As mentioned in https://docs.cometbft.com/v0.37/core/state-sync, one must set a height and hash in the config.toml along with a few rpc servers (the afromentioned link has instructions on how to do this). 
2. Run `<appd snapshot restore <height> <format>` to restore a local snapshot (note: first load it from a file with the *load* command). 
3. Bootsrapping Comet state in order to start the node after the snapshot has been ingested. This can be done with the bootstrap command `<app> comet bootstrap-state`

### Snapshots Commands

The Cosmos SDK provides commands for managing snapshots.
These commands can be added in an app with the following snippet in `cmd/<app>/root.go`:

```go
import (
  "github.com/cosmos/cosmos-sdk/client/snapshot"
)

func initRootCmd(/* ... */) {
  // ...
  rootCmd.AddCommand(
    snapshot.Cmd(appCreator),
  )
}
```

Then following commands are available at `<appd> snapshots [command]`:

* **list**: list local snapshots
* **load**: Load a snapshot archive file into snapshot store
* **restore**: Restore app state from local snapshot
* **export**:  Export app state to snapshot store
* **dump**: Dump the snapshot as portable archive format
* **delete**: Delete a local snapshot
