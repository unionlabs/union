# Wasm Zone

[![CircleCI](https://circleci.com/gh/CosmWasm/wasmd/tree/main.svg?style=shield)](https://circleci.com/gh/CosmWasm/wasmd/tree/main)
[![codecov](https://codecov.io/gh/cosmwasm/wasmd/branch/main/graph/badge.svg)](https://codecov.io/gh/cosmwasm/wasmd)
[![Go Report Card](https://goreportcard.com/badge/github.com/CosmWasm/wasmd)](https://goreportcard.com/report/github.com/CosmWasm/wasmd)
[![license](https://img.shields.io/github/license/CosmWasm/wasmd.svg)](https://github.com/CosmWasm/wasmd/blob/main/LICENSE)
[![LoC](https://tokei.rs/b1/github/CosmWasm/wasmd)](https://github.com/CosmWasm/wasmd)

<!-- [![GolangCI](https://golangci.com/badges/github.com/CosmWasm/wasmd.svg)](https://golangci.com/r/github.com/CosmWasm/wasmd) -->

This repository hosts `Wasmd`, the first implementation of a cosmos zone with wasm smart contracts enabled.

This code was forked from the `cosmos/gaia` repository as a basis and then we added `x/wasm` and cleaned up
many gaia-specific files. However, the `wasmd` binary should function just like `gaiad` except for the
addition of the `x/wasm` module.

**Note**: Requires [Go 1.21+](https://golang.org/dl/)

For critical security issues & disclosure, see [SECURITY.md](SECURITY.md).

## Compatibility

### For contract developers

Since CosmWasm 1.0 the contract-host interface has not changed in a breaking way.
Also CosmWasm 2.0 contracts remain compatible at the Wasm interface level.

To extend the feature set over time, contracts can specify required [capabilities](https://github.com/CosmWasm/cosmwasm/blob/main/docs/CAPABILITIES.md) through cargo features in cosmwasm-std.
The following table shows which of the [latest capabilities](https://github.com/CosmWasm/cosmwasm/blob/main/docs/CAPABILITIES-BUILT-IN.md) are supported by certain wasmd versions.

| capability   | >= 0.51 | >= 0.42 | >= 0.41 | >= 0.31 | >= 0.29 | 0.28 |
| ------------ | ------- | ------- | ------- | ------- | ------- | ---- |
| iterator     | x       | x       | x       | x       | x       | x    |
| stargate     | x       | x       | x       | x       | x       | x    |
| staking      | x       | x       | x       | x       | x       | x    |
| cosmwasm_1_1 | x       | x       | x       | x       | x       |      |
| cosmwasm_1_2 | x       | x       | x       | x       |         |      |
| cosmwasm_1_3 | x       | x       | x       |         |         |      |
| cosmwasm_1_4 | x       | x       |         |         |         |      |
| cosmwasm_2_0 | x       |         |         |         |         |      |

### For node developers

The [wasmvm](https://github.com/CosmWasm/wasmvm) dependency works in most aspects like any other Go dependency. When embedding wasmd as a module into your chain, wasmvm becomes a transitive (or "indirect") dependency of the final binary project. You can specify which wasmvm version you want in your node by adding it explicitly to go.mod or using a [`replace` directive](https://go.dev/ref/mod#go-mod-file-replace).

Please note that all minor version bumps of wasmvm are expected to be consensus breaking.
For patch releases this should not be the case but there are many exceptions and corner cases.

The following table shows

- **Specified wasmvm version:** the wasmvm dependency that wasmd specifies in its own go.mod
- **Compatible wasmvm version:** the versions you can use by setting it in your project's go.mod

| wasmd  | compatible | specified                                                         |
| ------ | ---------- | ----------------------------------------------------------------- |
| 0.51.0 | 2.0.x      | [2.0.0](https://github.com/CosmWasm/wasmd/blob/v0.51.0/go.mod#L6) |
| 0.50.0 | 1.5.x      | [1.5.0](https://github.com/CosmWasm/wasmd/blob/v0.50.0/go.mod#L6) |
| 0.45.0 | 1.5.x      | [1.5.0](https://github.com/CosmWasm/wasmd/blob/v0.45.0/go.mod#L6) |
| 0.44.0 | 1.5.x      | [1.5.0](https://github.com/CosmWasm/wasmd/blob/v0.44.0/go.mod#L6) |
| 0.43.0 | 1.4.x      | [1.4.1](https://github.com/CosmWasm/wasmd/blob/v0.43.0/go.mod#L6) |
| 0.42.0 | 1.4.x      | [1.4.1](https://github.com/CosmWasm/wasmd/blob/v0.42.0/go.mod#L6) |
| 0.41.0 | 1.3.x      | [1.3.0](https://github.com/CosmWasm/wasmd/blob/v0.41.0/go.mod#L6) |

Dependency resolution in Go is not obvious. In case of doubt, please use
`go list -m github.com/CosmWasm/wasmvm` to get the dynamically calculated version of the wasmvm dependency. Also check

```sh
# Replace <node> with you binary name
<node> query wasm libwasmvm-version
```

for getting the libwasmvm version loaded at runtime.

## Supported Systems

The supported systems are limited by the dlls created in [`wasmvm`](https://github.com/CosmWasm/wasmvm). In particular, **we only support MacOS and Linux**.
However, **M1 macs are not fully supported.** (Experimental support was merged with wasmd 0.24)
For linux, the default is to build for glibc, and we cross-compile with CentOS 7 to provide
backwards compatibility for `glibc 2.12+`. This includes all known supported distributions
using glibc (CentOS 7 uses 2.12, obsolete Debian Jessy uses 2.19).

As of `0.9.0` we support `muslc` Linux systems, in particular **Alpine linux**,
which is popular in docker distributions. Note that we do **not** store the
static `muslc` build in the repo, so you must compile this yourself, and pass `-tags muslc`.
Please look at the [`Dockerfile`](./Dockerfile) for an example of how we build a static Go
binary for `muslc`. (Or just use this Dockerfile for your production setup).

## Stability

**This is beta software** It is run in some production systems, but we cannot yet provide a stability guarantee
and have not yet gone through and audit of this codebase. Note that the
[CosmWasm smart contract framework](https://github.com/CosmWasm/cosmwasm) used by `wasmd` is in a 1.0 release candidate
as of March 2022, with stability guarantee and addressing audit results.

As of `wasmd` 0.22, we will work to provide upgrade paths _for this module_ for projects running a non-forked
version on their live networks. If there are Cosmos SDK upgrades, you will have to run their migration code
for their modules. If we change the internal storage of `x/wasm` we will provide a function to migrate state that
can be called by an `x/upgrade` handler.

The APIs are pretty stable, but we cannot guarantee their stability until we reach v1.0.
However, we will provide a way for you to hard-fork your way to v1.0.

Thank you to all projects who have run this code in your mainnets and testnets and
given feedback to improve stability.

## Encoding

The used cosmos-sdk version is in transition migrating from amino encoding to protobuf for state. So are we now.

We use standard cosmos-sdk encoding (amino) for all sdk Messages. However, the message body sent to all contracts,
as well as the internal state is encoded using JSON. Cosmwasm allows arbitrary bytes with the contract itself
responsible for decoding. For better UX, we often use `json.RawMessage` to contain these bytes, which enforces that it is
valid json, but also give a much more readable interface. If you want to use another encoding in the contracts, that is
a relatively minor change to wasmd but would currently require a fork. Please open an issue if this is important for
your use case.

## Quick Start

```
make install
make test
```

if you are using a linux without X or headless linux, look at [this article](https://ahelpme.com/linux/dbusexception-could-not-get-owner-of-name-org-freedesktop-secrets-no-such-name) or [#31](https://github.com/CosmWasm/wasmd/issues/31#issuecomment-577058321).

## Protobuf

The protobuf files for this project are published automatically to the [buf repository](https://buf.build/) to make integration easier:

| wasmd version | buf tag                                                                                                                                     |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| 0.31.x        | [e0e5a6fa433449e695af692478c86fb5](https://buf.build/cosmwasm/wasmd/tree/e0e5a6fa433449e695af692478c86fb5:cosmwasm/wasm/v1)                 |
| 0.30.x        | [6508ee062011440c907de6f5c40398ea](https://buf.build/cosmwasm/wasmd/tree/6508ee062011440c907de6f5c40398ea:cosmwasm/wasm/v1)                 |
| 0.29.x        | [51931206dbe09529c1819a8a2863d291035a2549](https://buf.build/cosmwasm/wasmd/tree/51931206dbe09529c1819a8a2863d291035a2549:cosmwasm/wasm/v1) |

Generate protobuf

```shell script
make proto-gen
```

The generators are executed within a Docker [container](./scripts/contrib/prototools-docker), now.

## Dockerized

We provide a docker image to help with test setups. There are two modes to use it

Build: `docker build -t cosmwasm/wasmd:latest .` or pull from dockerhub

### Dev server

Bring up a local node with a test account containing tokens

This is just designed for local testing/CI - do not use these scripts in production.
Very likely you will assign tokens to accounts whose mnemonics are public on github.

```sh
docker volume rm -f wasmd_data

# pass password (one time) as env variable for setup, so we don't need to keep typing it
# add some addresses that you have private keys for (locally) to give them genesis funds
docker run --rm -it \
    -e PASSWORD=xxxxxxxxx \
    --mount type=volume,source=wasmd_data,target=/root \
    cosmwasm/wasmd:latest /opt/setup_wasmd.sh cosmos1pkptre7fdkl6gfrzlesjjvhxhlc3r4gmmk8rs6

# This will start both wasmd and rest-server, both are logged
docker run --rm -it -p 26657:26657 -p 26656:26656 -p 1317:1317 \
    --mount type=volume,source=wasmd_data,target=/root \
    cosmwasm/wasmd:latest /opt/run_wasmd.sh
```

### CI

For CI, we want to generate a template one time and save to disk/repo. Then we can start a chain copying the initial state, but not modifying it. This lets us get the same, fresh start every time.

```sh
# Init chain and pass addresses so they are non-empty accounts
rm -rf ./template && mkdir ./template
docker run --rm -it \
    -e PASSWORD=xxxxxxxxx \
    --mount type=bind,source=$(pwd)/template,target=/root \
    cosmwasm/wasmd:latest /opt/setup_wasmd.sh cosmos1pkptre7fdkl6gfrzlesjjvhxhlc3r4gmmk8rs6

sudo chown -R $(id -u):$(id -g) ./template

# FIRST TIME
# bind to non-/root and pass an argument to run.sh to copy the template into /root
# we need wasmd_data volume mount not just for restart, but also to view logs
docker volume rm -f wasmd_data
docker run --rm -it -p 26657:26657 -p 26656:26656 -p 9090:9090 \
    --mount type=bind,source=$(pwd)/template,target=/template \
    --mount type=volume,source=wasmd_data,target=/root \
    cosmwasm/wasmd:latest /opt/run_wasmd.sh /template

# RESTART CHAIN with existing state
docker run --rm -it -p 26657:26657 -p 26656:26656 -p 1317:1317 \
    --mount type=volume,source=wasmd_data,target=/root \
    cosmwasm/wasmd:latest /opt/run_wasmd.sh
```

## Runtime flags

We provide a number of variables in `app/app.go` that are intended to be set via `-ldflags -X ...`
compile-time flags. This enables us to avoid copying a new binary directory over for each small change
to the configuration.

Available flags:

- `-X github.com/CosmWasm/wasmd/app.NodeDir=.corald` - set the config/data directory for the node (default `~/.wasmd`)
- `-X github.com/CosmWasm/wasmd/app.Bech32Prefix=coral` - set the bech32 prefix for all accounts (default `wasm`)

Examples:

- [`wasmd`](./Makefile#L50-L55) is a generic, permissionless version using the `cosmos` bech32 prefix

## Compile Time Parameters

Besides those above variables (meant for custom wasmd compilation), there are a few more variables which
we allow blockchains to customize, but at compile time. If you build your own chain and import `x/wasm`,
you can adjust a few items via module parameters, but a few others did not fit in that, as they need to be
used by stateless `ValidateBasic()`. Thus, we made them public `var` and these can be overridden in the `app.go`
file of your custom chain.

- `wasmtypes.MaxLabelSize = 64` to set the maximum label size on instantiation (default 128)
- `wasmtypes.MaxWasmSize=777000` to set the max size of compiled wasm to be accepted (default 819200)
- `wasmtypes.MaxProposalWasmSize=888000` to set the max size of gov proposal compiled wasm to be accepted (default 3145728)

## Genesis Configuration

We strongly suggest **to limit the max block gas in the genesis** and not use the default value (`-1` for infinite).

```json
  "consensus_params": {
    "block": {
      "max_gas": "SET_YOUR_MAX_VALUE",
```

Tip: if you want to lock this down to a permisisoned network, the following script can edit the genesis file
to only allow permissioned use of code upload or instantiating:

`sed -i 's/permission": "Everybody"/permission": "Nobody"/'  .../config/genesis.json`

## Contributors

Much thanks to all who have contributed to this project, from this app, to the `cosmwasm` framework, to example contracts and documentation.
Or even testing the app and bringing up critical issues. The following have helped bring this project to life:

- Ethan Frey [ethanfrey](https://github.com/ethanfrey)
- Simon Warta [webmaster128](https://github.com/webmaster128)
- Alex Peters [alpe](https://github.com/alpe)
- Aaron Craelius [aaronc](https://github.com/aaronc)
- Sunny Aggarwal [sunnya97](https://github.com/sunnya97)
- Cory Levinson [clevinson](https://github.com/clevinson)
- Sahith Narahari [sahith-narahari](https://github.com/sahith-narahari)
- Jehan Tremback [jtremback](https://github.com/jtremback)
- Shane Vitarana [shanev](https://github.com/shanev)
- Billy Rennekamp [okwme](https://github.com/okwme)
- Westaking [westaking](https://github.com/westaking)
- Marko [marbar3778](https://github.com/marbar3778)
- JayB [kogisin](https://github.com/kogisin)
- Rick Dudley [AFDudley](https://github.com/AFDudley)
- KamiD [KamiD](https://github.com/KamiD)
- Valery Litvin [litvintech](https://github.com/litvintech)
- Leonardo Bragagnolo [bragaz](https://github.com/bragaz)

Sorry if I forgot you from this list, just contact me or add yourself in a PR :)
