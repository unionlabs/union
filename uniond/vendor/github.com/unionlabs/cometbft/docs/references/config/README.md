---
order: 1
parent:
  title: Configuration Manual
  description: A comprehensive reference manual for configuring CometBFT
  order: false
---
# CometBFT Configuration Manual

## Overview
The CometBFT configuration has three distinct parts:
1. The network parameters in [genesis.json](genesis.json.md).
2. The nodeID in [node_key.json](node_key.json.md).
3. The configuration of the node and its services in [config.toml](config.toml.md).

Validator nodes also require a private/public key-pair to sign consensus messages.

If a Hardware Security Module (HSM) is not available, CometBFT stores an unencrypted key-pair on the file system in the
[priv_validator_key.json](priv_validator_key.json.md) file and the state of the last block signed in
[priv_validator_state.json](priv_validator_state.json.md).

## The HOME folder
The CometBFT HOME folder contains all configuration (in the `$HOME/config` folder) for CometBFT as well as all the databases (in the `$HOME/data` folder)
used during execution.

Path to the folder is defined by these steps:
1. The home folder for CometBFT is read from the `CMTHOME` environment variable.
2. If the variable is undefined, it is assumed the default `$HOME/.cometbft`.
3. The environment variable is overridden by the `--home` command-line parameter.

By default, all configuration files are stored under the `$CMTHOME/config` directory.
These can be overridden individually for each file in the `config.toml` file, for example to
override the `genesis_file` location change it [here](config.toml.md#genesis_file).

By default, all databases are stored under the `$CMTHOME/data` directory.
This can be overridden at the [`db_dir`](config.toml.md#db_dir) parameter in the [`config.toml`](config.toml.md) file.
