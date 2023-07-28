# Architecture

This document describes the high-level architecture of Union and associated packages. If you are contributing, auditing or just having a look at the tech, this is the place to start. If you aren't a developer or unfamiliar with Go and Rust, it's probably better to have a look at the official docs instead.

## Repository Structure

At the root of the repository, you'll find directories such as `uniond`, `unionvisor`, and `galoisd`, which are binaries necessary to run the network. To see how to build one of these binaries, check the `flake.nix` file. These binaries can be built by running

```nix
nix build .\#uniond # or unionvisor or galoisd
```

To see all packages/apps that we define, run `nix flake show`. To see how these are built, check out `flake.nix`. Here we import all [`flake-parts`](https://flake.parts), such as `uniond/uniond.nix`.

### Generated Code

Currently generated code, such as protobuf definitions and a vendor directory, are checked into the repo. This will be removed later once we add support for private repos and proto derivations.

### READMEs

We attempt to have a README.md for every significant component, describing what it is and how to work on it.

### Documentation

Source code is always the source of truth. The best location to get into the nitty-gritty details is doc comments. We currently do not publish these, but might in the future.

We refrain from separating docs and links from the actual code, as refactors can create dead links and stale docs. It's best to use text search to find where components are defined, as PRs are still adding significant components.

## High-level Overview

### Binaries

`uniond` is the network node, which is run by validators to produce blocks.
`unionvisor` is a supervisor of `uniond`, which makes deployments easier and more resilient. It is not required for node operations but is recommended.
`galoisd` is the ZK prover. Validators do not need to run it, but IBC relayers and MEV searchers will need to process transactions and capture value.

### Support

`tools` is used to bring in 3rd party tooling and development tools.
`networks` is used to define docker-compose configurations of Union and Ethereum networks for local testing.
