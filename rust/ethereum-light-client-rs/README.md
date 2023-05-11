# ethereum-light-client-rs

[![test](https://github.com/datachainlab/ethereum-light-client-rs/actions/workflows/test.yml/badge.svg)](https://github.com/datachainlab/ethereum-light-client-rs/actions/workflows/test.yml)

A rust implementation of the ethereum light client that supports `no_std`.

It currently supports the verification of [Sync Protocol](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md), also called Altair Light Client.

## Key Features

- Sync Protocol verification
- Support the detection of Sync committee's misbehaviour
- `no_std` support: easy to integrate into any environment(e.g. wasm, sgx enclave)

## Crates

- [light-client-verifier](./crates/light-client-verifier): provides a Sync Protocol and Execution layer verifiers
- [consensus](./crates/consensus): provides the implementation of [the consensus specs](https://github.com/ethereum/consensus-specs) for beacon chain and sync protocol
- [light-client-cli](./crates/light-client-cli): A toy CLI for Light Client
- [lodestar-rpc](./crates/lodestar-rpc): A RPC client for [lodestar](https://github.com/chainSafe/lodestar)
