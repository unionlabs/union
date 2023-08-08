---
title: "Getting Started"
---

This guide is intended for validators running on bare-metal servers and explains how Union releases work. Check out the [NixOS](./nixos) and the [kubernetes](./kubernetes) guide for more production-ready deployments.

Validators are the backbone of the network. Becoming one requires significant token bonding and delegations, and is not intended for non-power users.

## Obtaining uniond

You can obtain `uniond` from a recent [release](https://github.com/unionlabs/union/releases/latest).

```sh
curl -L -o uniond https://github.com/unionlabs/union/releases/download/v0.5.0-rc1/uniond-x86_64-linux
```

Verify that the binary works on your server by running:

```sh
./uniond --help
```

For convenience, we can add the binary to the `PATH`, to make it callable from anywhere.

```sh
mv ./uniond /usr/bin/
```

### Using Docker

We also provide containers in our [package registry](https://github.com/unionlabs/union/pkgs/container/uniond).

```sh
docker pull ghcr.io/unionlabs/uniond:v0.5.0-rc1
```

When running the container, make sure to map a volume to the path passed in `--home` options to ensure data persistence. From here on the guide assumes the usage of a regular binary. The [docker-compose](./docker-compose) section is more suited for docker users.

:::caution
`uniond` is a stateful application and interacts with the file system. Make sure to use [volumes](https://docs.docker.com/storage/volumes/).
:::

## Initialization

We'll need to set up a few configuration files and obtain the [genesis.json](https://raw.githubusercontent.com/unionlabs/genesis/main/union-testnet-1/genesis.json) before we can run the node.

First, set some environment variables, which are used throughout initialization.

```sh
export CHAIN_ID=union-testnet-1 # or union-1
export MONIKER="Unionized Goblin"
export KEY_NAME=alice
export GENESIS_URL="https://raw.githubusercontent.com/unionlabs/genesis/main/union-testnet-1/genesis.json"
```

Then we'll have `uniond` initialize our data and configuration directories. By default `/User/{USER}/.uniond` is used.

```
uniond init $MONIKER "bn254" --chain-id $CHAIN_ID
```

The `[key_type]` is `"bn254"`, which most validators haven't encountered before when running Tendermint nodes. That's because it's part of [CometBLS](02_architecture/cometbls.md).

### Seeds

Next, edit `~/.union/config/config.toml`. We'll set the seeds to ensure your node can connect to the peer-to-peer network.

For `union-testnet-1` replace `seeds = ""` with:

```toml
seeds = "c649931f0ef98bc3e086bbfbcf3b04896a9ec7de@uniontestnet.poisonphang.com:26656"
```

### Genesis Configuration

Download the [genesis.json](https://raw.githubusercontent.com/unionlabs/genesis/main/union-testnet-1/genesis.json) and copy it to your `uniond` home directory.

```
curl $GENESIS_URL > ~/.union/config/genesis.json
```

### Registration

To join as a validator, you need to submit a registration transaction. You can do this from the command line on your validator node.

First, add a wallet that holds Union tokens.

```
uniond keys add $KEY_NAME --recover
```

:::caution
For production usage, we recommend not storing the wallet on a server.
:::

To submit the registration transaction and become a validator, you must submit a `create-validator` transaction:

```sh
uniond tx staking create-validator \
  --amount $STAKE \
  --pubkey=$(uniond tendermint show-validator) \
  --moniker $MONIKER \
  --chain-id $CHAIN_ID \
  --from $KEY_NAME
```

## Systemd Service

We recommend running `uniond` as a systemd service. Create a file in `/etc/systemd/system` called `uniond.service`. Make sure to replace $USER with your username.

```
[Unit]
Description=uniond
[Service]
Type=simple
Restart=always
RestartSec=1
User=$USER
ExecStart=/usr/bin/uniond start

[Install]
WantedBy=multi-user.target
```

You should be able to view the node logs by executing

```
sudo journalctl -f -u uniond
```

It's then recommended to backup these files from `~/.union/config` in a secure location:

- `priv_validator_key.json`
- `node_key.json`
