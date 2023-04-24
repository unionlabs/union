# Hosting a Node

This document will walk you through the process of self-hosting a union node.

# Nix Install

Run `nix build` in the root of our repository after cloning.

# Docker Install

*NOTE: Not yet available*

# Manual Install

This section details the manual process for self-hosting a node.

## Requirements

Before continuing with this process, ensure the following requirements are available on your system:

* make
* build-essential
* gcc
* git
* jq
* chrony
* go

After installing the requirements, you'll need to create some environment variables.

```sh
export GOPATH=$HOME/go
export PATH=$PATH:$HOME/go/bin
```

## Installing `uniond`

This section will walk you through installing the `uniond` binary that will be used for running the node.

To begin, clone the `UnionFi/union` repository and checkout the release branch you desire.

<!-- TODO: Replace `$RELEASE_BRANCH` with our current release branch or test-net release branch. -->

```sh
cd ~
git clone https://github.com/UnionFi/union
cd union
git fetch
git checkout $RELEASE_BRANCH
```

Now that you have the `UnionFi/union` repository, you can build the `uniond` binary.

```sh
cd ~/union/uniond
make install
```

To ensure the installation has succeeded, you can run:

```sh
uniond version
```

## Connect to the Public RPC

*NOTE: The public RPC is currently not available.*

Now to connect the `uniond` binary to the public RPC.

First, set the chain-id with:

<!-- TODO: Replace `$CHAIN_ID` with our the chain-id of our main-net or test-net. -->
```sh
uniond config chain-id $CHAIN_ID
```

Set the public RPC node:

<!-- TODO: Replace `$RPC_NODE_URL` with our RPC node URL. -->
```sh
uniond config node $RPC_NODE_URL
```

