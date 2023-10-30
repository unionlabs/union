---
title: "Obtaining the Union Testnet Binary"
---

Currently, we are only officially supporting running the Union Testnet binary (`uniond`) as a Docker container.

You are welcome to try other means of running the binary. If you succeed in running a node outside a Docker container, please let us know about it in our [Discord](https://discord.gg/union-build).

This guide assumes you have [Docker](https://www.docker.com/get-started/) correctly installed and configured for your system. We provide `uniond` images for Linux on both x86_64 (amd64) and aarch64 (arm64).

## Getting the Docker Image

To get the `uniond` Docker image, you can visit [our container on the GitHub Container Registry](https://github.com/unionlabs/union/pkgs/container/uniond), or run the following command:

```sh
docker pull ghcr.io/unionlabs/uniond:v0.13.0
```

## Running uniond

### Creating a Chain Config & State Folder

Before running this docker image, we'll want to create a folder to host the chain configuration and state.

You can create this wherever you would like, but we'll be doing so in our current user's home directory.


:::caution
It's important that you will be able to edit this contents of this folder.
:::

To create a directory for `uniond` in your user home directory, run:

```sh
mkdir ~/.union
```

### Initializing the Chain Config & State Folder

Now, using the `uniond` docker image and the folder we just created, we can initialize the contents of this folder.

To do this, we'll be using docker volumes.

```sh
docker run -u $(id -u):$(id -g) -v ~/.union:/.union -it ghcr.io/unionlabs/uniond:v0.13.0 init $MONIKER bn254 --home /.union
```
*Where `MONIKER` is the preferred moniker you'd like to use on this node.*

:::note
Note the usage of the flags and arguments we pass to `docker run` run here:

- `-u $(id -u):(id -g)` ensures that the docker container is being created and ran with the current user and their permissions
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

### Issuing Commands to uniond

### Starting the Node
