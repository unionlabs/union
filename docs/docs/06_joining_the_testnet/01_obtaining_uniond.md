---
title: "Obtaining the Union Testnet Binary"
---

Currently, we are only officially supporting running the Union Testnet binary (`uniond`) as a Docker container.

It is possible to run the `uniond` binary outside of containers, however, we aren't directly supplying bare-metal binaries at this time.

This guide assumes you have [Docker](https://www.docker.com/get-started/) correctly installed and configured on your system. We provide `uniond` images for Linux on both x86_64 (amd64) and aarch64 (arm64).

## Getting the Docker Image

To get the `uniond` image, you can visit [our container on the GitHub Container Registry](https://github.com/orgs/unionlabs/packages/container/package/uniond), or run the following command:

```sh
export UNIOND_VERSION='v0.14.0'
docker pull ghcr.io/unionlabs/uniond:$UNIOND_VERSION
```

## Running uniond

### Creating a Chain Config & State Folder

Before running this image, we need to create a folder to host the chain configuration and state.

You can create this wherever you would like, but we'll be doing so in our current user's home directory.

:::caution
It's important that you will be able to edit this contents of this folder.
:::

To create a directory for `uniond` in your user home directory, run:

```sh
mkdir ~/.union
```

### Initializing the Chain Config & State Folder

Now, using the `uniond` image and the folder we just created, we can initialize the contents of this folder.

To do this, we'll be using Docker volumes.

```sh
docker run -u $(id -u):$(id -g) -v ~/.union:/.union -it ghcr.io/unionlabs/uniond:$UNIOND_VERSION init $MONIKER bn254 --home /.union
```

_Where `MONIKER` is the preferred moniker you'd like to use on this node._

:::note

Note the usage of the flags and arguments we pass to `docker run` run here:

- `-u $(id -u):(id -g)` ensures that the container is being created and ran with the current user and their permissions
- `-v ~/.union:/.union` mounts the folder we created to the `/.union` folder of the container
- `-it` ensures we are running the container interactively

:::

After the `uniond init` command is done running, you should have a `.union` folder with the following contents:

```
.union
├── config
│   ├── app.toml
│   ├── client.toml
│   ├── config.toml
│   ├── genesis.json
│   ├── node_key.json
│   └── priv_validator_key.json
└── data
    └── priv_validator_state.json
```

### Issuing Sub-Commands to uniond

To run `uniond` sub-commands, it will be useful to alias the Docker command in your shell `.*rc` file.

For example, in `zsh`, you can add the following alias to your `.zshrc`:

```sh
export UNIOND_VERSION='v0.14.0'
alias uniond='docker run -v ~/.union:/.union --network host -it ghcr.io/unionlabs/uniond:$UNIOND_VERSION --home /.union'
```

This will enable you to issue `uniond` sub-commands with such as `uniond keys add` with ease.

### Starting the Node

To run a node using `uniond`, you'll also need to expose ports to the container. We'll use this as an opportunity to create a Docker Compose file four `uniond`.

A minimal Docker Compose file for `uniond` looks like this:

```yaml
services:
  node:
    image: ghcr.io/unionlabs/uniond:${UNIOND_VERSION}
    volumes:
      - ~/.union:/.union
      - /tmp:/tmp
    network_mode: "host"
    restart: unless-stopped
    command: start --home /.union
```

:::note

You only need to mount `/tmp` if you intend to use [State Sync](./state_sync) to join the network

:::

This will mount our chain configuration and settings folder while also exposing ports `26657`, `1317`, and `9093`.

After creating a `compose.yml` file with the contents above, you'll be able to start your Union node with `docker compose`.

:::warning
Before starting your Union node for the first time, you should configure your node correctly and obtain the genesis file.

For some configuration recommendations see our [Node Configuration](../04_infrastructure/01_node_operators/node_configuration.md) page.

You can obtain the testnet genesis from https://rpc.cryptware.io/genesis or by running this command:

```sh
curl https://rpc.cryptware.io/genesis | jq '.result.genesis' > ~/.union/config/genesis.json
```

:::

To run your node in detached mode, run:

```sh
docker compose up -f path/to/compose.yml -d
```
