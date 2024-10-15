---
sidebar_position: 1
---

# Confix

`Confix` is a configuration management tool that allows you to manage your configuration via CLI.

It is based on the [CometBFT RFC 019](https://github.com/cometbft/cometbft/blob/5013bc3f4a6d64dcc2bf02ccc002ebc9881c62e4/docs/rfc/rfc-019-config-version.md).

## Installation

### Add Config Command

To add the confix tool, it's required to add the `ConfigCommand` to your application's root command file (e.g. `<appd>/cmd/root.go`).

Import the `confixCmd` package:

```go
import "cosmossdk.io/tools/confix/cmd"
```

Find the following line:

```go
initRootCmd(rootCmd, moduleManager)
```

After that line, add the following:

```go
rootCmd.AddCommand(
    confixcmd.ConfigCommand(),
)
```

The `ConfixCommand` function builds the `config` root command and is defined in the `confixCmd` package (`cosmossdk.io/tools/confix/cmd`).
An implementation example can be found in `simapp`.

The command will be available as `simd config`.

:::tip
Using confix directly in the application can have less features than using it standalone.
This is because confix is versioned with the SDK, while `latest` is the standalone version.
:::

### Using Confix Standalone

To use Confix standalone, without having to add it in your application, install it with the following command:

```bash
go install cosmossdk.io/tools/confix/cmd/confix@latest
```

Alternatively, for building from source, simply run `make confix`. The binary will be located in `tools/confix`.

## Usage

Use standalone:

```shell
confix --help
```

Use in simd:

```shell
simd config fix --help
```

### Get

Get a configuration value, e.g.:

```shell
simd config get app pruning # gets the value pruning from app.toml
simd config get client chain-id # gets the value chain-id from client.toml
```

```shell
confix get ~/.simapp/config/app.toml pruning # gets the value pruning from app.toml
confix get ~/.simapp/config/client.toml chain-id # gets the value chain-id from client.toml
```

### Set

Set a configuration value, e.g.:

```shell
simd config set app pruning "enabled" # sets the value pruning from app.toml
simd config set client chain-id "foo-1" # sets the value chain-id from client.toml
```

```shell
confix set ~/.simapp/config/app.toml pruning "enabled" # sets the value pruning from app.toml
confix set ~/.simapp/config/client.toml chain-id "foo-1" # sets the value chain-id from client.toml
```

### Migrate

Migrate a configuration file to a new version, config type defaults to `app.toml`, if you want to change it to `client.toml`, please indicate it by adding the optional parameter, e.g.:

```shell
simd config migrate v0.50 # migrates defaultHome/config/app.toml to the latest v0.50 config
simd config migrate v0.50 --client # migrates defaultHome/config/client.toml to the latest v0.50 config
```

```shell
confix migrate v0.50 ~/.simapp/config/app.toml # migrate ~/.simapp/config/app.toml to the latest v0.50 config
confix migrate v0.50 ~/.simapp/config/client.toml --client # migrate ~/.simapp/config/client.toml to the latest v0.50 config
```

### Diff

Get the diff between a given configuration file and the default configuration file, e.g.:

```shell
simd config diff v0.47 # gets the diff between defaultHome/config/app.toml and the latest v0.47 config
simd config diff v0.47 --client # gets the diff between defaultHome/config/client.toml and the latest v0.47 config
```

```shell
confix diff v0.47 ~/.simapp/config/app.toml # gets the diff between ~/.simapp/config/app.toml and the latest v0.47 config
confix diff v0.47 ~/.simapp/config/client.toml --client # gets the diff between ~/.simapp/config/client.toml and the latest v0.47 config
```

### View

View a configuration file, e.g:

```shell
simd config view client # views the current app client config
```

```shell
confix view ~/.simapp/config/client.toml # views the current app client conf
```

### Maintainer

At each SDK modification of the default configuration, add the default SDK config under `data/vXX-app.toml`.
This allows users to use the tool standalone.

### Compatibility

The recommended standalone version is `latest`, which is using the latest development version of the Confix.

| SDK Version | Confix Version |
| ----------- | -------------- |
| v0.50       | v0.1.x         |
| v0.52       | v0.2.x         |
| v2          | v0.2.x         |

## Credits

This project is based on the [CometBFT RFC 019](https://github.com/cometbft/cometbft/blob/5013bc3f4a6d64dcc2bf02ccc002ebc9881c62e4/docs/rfc/rfc-019-config-version.md) and their never released own implementation of [confix](https://github.com/cometbft/cometbft/blob/v0.36.x/scripts/confix/confix.go).
