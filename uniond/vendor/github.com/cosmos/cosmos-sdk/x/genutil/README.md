# `x/genutil`

## Concepts

The `genutil` package contains a variety of genesis utility functionalities for usage within a blockchain application. Namely:

* Genesis transactions related (gentx)
* Commands for collection and creation of gentxs
* `InitChain` processing of gentxs
* Genesis file creation
* Genesis file validation
* Genesis file migration
* CometBFT related initialization
    * Translation of an app genesis to a CometBFT genesis

## Genesis

Genutil contains the data structure that defines an application genesis.
An application genesis consist of a consensus genesis (g.e. CometBFT genesis) and application related genesis data.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-rc.0/x/genutil/types/genesis.go#L24-L34
```

The application genesis can then be translated to the consensus engine to the right format:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-rc.0/x/genutil/types/genesis.go#L126-L136
```

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-rc.0/server/start.go#L397-L407
```

## Client

### CLI

The genutil commands are available under the `genesis` subcommand.

#### add-genesis-account

Add a genesis account to `genesis.json`. Learn more [here](https://docs.cosmos.network/main/run-node/run-node#adding-genesis-accounts).

#### collect-gentxs

Collect genesis txs and output a `genesis.json` file.

```shell
simd genesis collect-gentxs
```

This will create a new `genesis.json` file that includes data from all the validators (we sometimes call it the "super genesis file" to distinguish it from single-validator genesis files).

#### gentx

Generate a genesis tx carrying a self delegation.

```shell
simd genesis gentx [key_name] [amount] --chain-id [chain-id]
```

This will create the genesis transaction for your new chain. Here `amount` should be at least `1000000000stake`.
If you provide too much or too little, you will encounter an error when starting a node.

#### migrate

Migrate genesis to a specified target (SDK) version.

```shell
simd genesis migrate [target-version]
```

:::tip
The `migrate` command is extensible and takes a `MigrationMap`. This map is a mapping of target versions to genesis migrations functions.
When not using the default `MigrationMap`, it is recommended to still call the default `MigrationMap` corresponding the SDK version of the chain and prepend/append your own genesis migrations.
:::

#### validate-genesis

Validates the genesis file at the default location or at the location passed as an argument.

```shell
simd genesis validate-genesis
```

:::warning
Validate genesis only validates if the genesis is valid at the **current application binary**. For validating a genesis from a previous version of the application, use the `migrate` command to migrate the genesis to the current version.
:::
