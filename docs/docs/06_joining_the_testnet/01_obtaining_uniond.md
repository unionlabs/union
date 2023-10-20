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
