# Unionvisor

Unionvisor is a utility for managing `uniond` deployments. It manages running, upgrading, and interacting with the node.

## Obtaining Unionvisor

We release container images of Unionvisor called bundles. Each bundle contains everything required for running Unionvisor and joining a particular network. The Unionvisor bundle for `union-testnet-4` is `bundle-testnet-4`. You can obtain the Unionvisor bundle for `union-testnet-4` from our [GitHub Container Registry](https://github.com/orgs/unionlabs/packages/container/package/bundle-testnet-4).

Alternatively, you can run the following command:

```sh
docker pull ghcr.io/unionlabs/bundle-testnet-4:$UNIOND_VERSION
```

## Running Unionvisor

Before running this image, we need to create a folder to host the chain configuration and state.

You can create this wherever you would like, but we'll be doing so in our current user's home directory.

:::caution
It's important that you will be able to edit this contents of this folder.
:::

To create a directory for `unionvisor` in your user home directory, run:

```sh
mkdir ~/.unionvisor
```

### Initializing the Chain Config & State Folder

Now, using the `unionvisor` image and the folder we just created, we can initialize the contents of this folder.

To do this, we'll be using Docker volumes.

```sh
docker run -v ~/.unionvisor:/.unionvisor -it 268347bed4b8 init --moniker $MONIKER --network union-testnet-4 --seeds "a069a341154484298156a56ace42b6e6a71e7b9d@blazingbit.io:27656,8a07752a234bb16471dbb577180de7805ba6b5d9@union.testnet.4.seed.poisonphang.com:26656"
```

_Where `MONIKER` is the preferred moniker you'd like to use on this node._

:::note

Note the usage of the flags and arguments we pass to `docker run` run here:

- `-v ~/.unionvisor:/.unionvisor` mounts the folder we created to the `/.unionvisor` folder of the container
- `-it` ensures we are running the container interactively

:::

After the `uniond init` command is done running, you should have a `.union` folder with the following contents:

```
~/.unionvisor
├── home
│   ├── config
│   │   ├── app.toml
│   │   ├── client.toml
│   │   ├── config.toml
│   │   ├── genesis.json
│   │   ├── node_key.json
│   │   └── priv_validator_key.json
│   └── data
│       └── priv_validator_state.json
└── uniond -> /versions/v0.14.0/uniond
```

### Issuing Sub-Commands to uniond via Unionvisor

To run `uniond` sub-commands, it will be useful to alias the Docker command in your shell `.*rc` file.

For example, in `zsh`, you can add the following alias to your `.zshrc`:

```sh
export UNIOND_VERSION='v0.14.0'
alias uniond='docker run -v ~/.unionvisor:/.unionvisor --network host -it ghcr.io/unionlabs/bundle-testnet-4:$UNIOND_VERSION call'
```

:::note

The `unionvisor call` sub-command passes commands and arguments to `uniond`.

:::

This will enable you to issue `uniond` sub-commands with such as `uniond keys add` with ease.

### Starting the Node

To run a node using Unionvisor, you'll also need to expose ports to the container. We'll use this as an opportunity to create a Docker Compose file four Unionvisor.

A minimal Docker Compose file for Unionvisor looks like this:

```yaml
services:
  node:
    image: ghcr.io/unionlabs/bundle-testnet-4:$UNIOND_VERSION
    volumes:
      - ~/.unionvisor:/.unionvisor
      - /tmp:/tmp
    network_mode: "host"
    restart: unless-stopped
    command: run --poll-interval 1000
```

:::note

You only need to mount `/tmp` if you intend to use [State Sync](./state_sync) to join the network

:::

This will mount our chain configuration and settings folder while also exposing ports `26657`, `1317`, and `9093`.

After creating a `compose.yml` file with the contents above, you'll be able to start your Union node with `docker compose`.

:::warning
Before starting your Union node for the first time, you should configure your node correctly.

For some configuration recommendations see our [Node Configuration](../04_infrastructure/01_node_operators/node_configuration.md) page.

:::

To run your node in detached mode, run:

```sh
docker compose -f path/to/compose.yaml up  -d
```
