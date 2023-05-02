---
title: 'Running uniond'
---

# Running `uniond`

This document details the various ways of running `uniond`, the binary required to run a union node (devnet, testnet, or mainnet).

# Docker

To run `uniond` without manually installing `nix` or NixOS, you can use docker.

## Pre-requisites

* A working installation of docker

## Login

*Note: This section will not be required once the [union](https://github.com/unionfi/union) repository is public*

We use the GitHub Container Registry to host our docker images. Given the [union](https://github.com/unionfi/union) repository is under a private organization, you will first need to authenticate docker with ghcr.io before downloading and running the `uniond` docker image.

Before using `docker login` you will need to create a GitHub personal access token (classic). To do this, follow the steps from [Creating a personal access token (classic)](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token#creating-a-personal-access-token-classic). You must ensure the PAT has, at least, the `read:packages`.

After securely noting down your GitHub PAT, you can use it to authenticate `docker` with ghcr.io. To do this run:

```sh
docker login ghcr.io
```

Supply `docker` with your GitHub username. When asked for your password, instead supply your GitHub PAT that you just created and noted.

*Note: By default, `docker` will insecurely store your GitHub PAT. For alternatives, see: [docker login: Credentials Store](https://docs.docker.com/engine/reference/commandline/login/#credentials-store)*

You should now be able to download and run docker images from ghcr.io.

## Running `uniond` with Docker

You should now visit the [`uniond` Package Container](https://github.com/unionfi/union/pkgs/container/uniond) and find the version of `uniond` you are trying to run. With the version in mind, you can run:

```sh
docker run ghcr.io/unionfi/uniond:$VERSION
```

You should now be able to run this version of `uniond` locally.


