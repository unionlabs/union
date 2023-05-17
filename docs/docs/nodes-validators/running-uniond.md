---
title: "Running uniond"
---

# Running `uniond`

This document details the various ways of running `uniond`, the binary required to run a union node (devnet, testnet, or mainnet).

# Docker

To run `uniond` without manually installing `nix` or NixOS, you can use docker.

## Pre-requisites

- A working installation of docker

## Login

_Note: This section will not be required once the [union](https://github.com/unionfi/union) repository is public_

We use the GitHub Container Registry to host our docker images. Given the [union](https://github.com/unionfi/union) repository is under a private organization, you will first need to authenticate docker with ghcr.io before downloading and running the `uniond` docker image.

Before using `docker login` you will need to create a GitHub personal access token (classic). To do this, follow the steps from [Creating a personal access token (classic)](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token#creating-a-personal-access-token-classic). You must ensure the PAT has, at least, the `read:packages`.

After securely noting down your GitHub PAT, you can use it to authenticate `docker` with ghcr.io. To do this run:

```sh
docker login ghcr.io
```

Supply `docker` with your GitHub username. When asked for your password, instead supply your GitHub PAT that you just created and noted.

_Note: By default, `docker` will insecurely store your GitHub PAT. For alternatives, see: [docker login: Credentials Store](https://docs.docker.com/engine/reference/commandline/login/#credentials-store)_

You should now be able to download and run docker images from ghcr.io.

## Running `uniond` with Docker

You should now visit the [`uniond` Package Container](https://github.com/unionfi/union/pkgs/container/uniond) and find the version of `uniond` you are trying to run. With the version in mind, you can run:

```sh
docker run ghcr.io/unionfi/uniond:$UNIOND_VERSION
```

You should now be able to run this version of `uniond` locally.

### Tips

Running `uniond` with `docker run` has a few caveats. These tips will help ensure you're able to accomplish everything you need with `uniond` using `docker`.

- **Storing your `uniond` configuration:**

  When interacting with `uniond` it is helpful to have a persistent and accessible location to store your `uniond` configuration.

  This can be accomplished with the `--mount` flag for `docker run`. For example, to store your union configuration that `docker` will use in `$HOME/uniond-config` you should:

  ```sh
  cd ~
  # Ensure $HOME/uniond-config exists
  mkdir uniond-config
  # Then use the --mount flag from docker and --home flag from uniond to store/use the configuration stored in $HOME/uniond-config
  docker run --mount type=bind,source="$HOME/uniond-config",target=/uniond-config $DOCKER_FLAGS ghcr.io/unionfi/uniond:$UNIOND_VERSION $UNIOND_SUB_COMMAND --home "/uniond-config"
  ```

- **Publish your docker container ports:**

  When running a validator node, you will need to publish the necessary TCP ports to communicate with and receive request from other nodes.

  To do this, use the `-p` flag for `docker run`. For example, assuming you're using ports `26656` and `26657` you should:

  ```sh
  # Include both ports for traffic to flow through
  docker run -p 26656:26656 -p 26657:26657 $DOCKER_FLAGS ghcr.io/unionfi/uniond:$UNIOND_VERSION $UNIOND_SUB_COMMAND
  ```
