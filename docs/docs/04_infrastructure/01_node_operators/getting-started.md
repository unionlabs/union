---
title: "Getting Started"
sidebar_position: 0
---

This guide is intended for validators running on bare-metal servers and explains how Union releases work. Check out the [NixOS](./nixos) and the [Kubernetes](./kubernetes) guide for more production-ready deployments.

Validators are the backbone of the network. Becoming one requires significant token bonding and delegations, and is not intended for non-power users.

## Obtaining uniond

:::note
Currently, directly downloading the `uniond` binary requires access to our private GitHub repository.

If you don't have access to our private GitHub repository, you can still run our node using the public Docker image.
:::

You can obtain `uniond` from a recent [release](https://github.com/unionlabs/union/releases/latest).

:::caution
Double-check the version and architecture in the link before downloading.
:::

```sh
curl -L -o uniond https://github.com/unionlabs/union/releases/download/v0.14.0/uniond-x86_64-linux
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
docker pull ghcr.io/unionlabs/uniond:v0.14.0
```

When running the container, make sure to map a volume to the path passed in `--home` options to ensure data persistence. From here on the guide assumes the usage of a regular binary. The [docker-compose](./docker-compose) section is more suited for docker users.

:::caution
`uniond` is a stateful application and interacts with the file system. Make sure to use [volumes](https://docs.docker.com/storage/volumes/).
:::

## Initialization

We'll need to set up a few configuration files and obtain the [`genesis.json`](https://rpc.cryptware.io/genesis) before we can run the node.

First, set some environment variables, which are used throughout initialization.

```sh
export CHAIN_ID=union-testnet-4 # or union-1
export MONIKER="Unionized Goblin"
export KEY_NAME=alice
export GENESIS_URL="https://rpc.cryptware.io/genesis"
```

Then we'll have `uniond` initialize our data and configuration directories. By default, `/User/{USER}/.uniond` is used.

```sh
uniond init $MONIKER "bn254" --chain-id $CHAIN_ID
```

The `[key_type]` is `"bn254"`, which most validators haven't encountered before when running Tendermint nodes. That's because it's part of [CometBLS](02_architecture/cometbls.md).

### Seeds

Next, edit `~/.union/config/config.toml`. We'll set the seeds to ensure your node can connect to the peer-to-peer network.

For `union-testnet-4` replace `seeds = ""` with:

```toml
seeds = "a069a341154484298156a56ace42b6e6a71e7b9d@blazingbit.io:27656,8a07752a234bb16471dbb577180de7805ba6b5d9@union.testnet.4.seed.poisonphang.com:26656"
```

### Genesis Configuration

Download the `genesis.json` and copy it to your `uniond` home directory.

```sh
curl $GENESIS_URL > ~/.union/config/genesis.json
```

### Registration

To join as a validator, you need to submit a registration transaction. You can do this from the command line on your validator node.

First, add a wallet that holds Union tokens.

```sh
uniond keys add $KEY_NAME --recover
```

:::caution
For production usage, we recommend not storing the wallet on a server.
:::

To submit the registration transaction and become a validator, you must submit a `create-validator` transaction:

```sh
uniond tx staking create-validator \
  --amount 1000000muno \
  --pubkey $(uniond tendermint show-validator) \
  --moniker $MONIKER \
  --chain-id $CHAIN_ID \
  --from $KEY_NAME \
  --commission-max-change-rate "0.1" \
  --commission-max-rate "0.20" \
  --commission-rate "0.1" \
  --min-self-delegation "1"
```

:::note
If your own node isn't set up to accept RPC request, you can send them to another node such as `https://rpc.0xc0dejug.uno:443` via the `--node` option.
:::

## Systemd Service

We recommend running `uniond` as a systemd service. Create a file in `/etc/systemd/system` called `uniond.service`. Make sure to replace $USER with your username.

```systemd
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

```sh
sudo journalctl -f -u uniond
```

It's then recommended to back up these files from `~/.union/config` in a secure location:

- `priv_validator_key.json`
- `node_key.json`
