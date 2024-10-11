<!--
Guiding Principles:

Changelogs are for humans, not machines.
There should be an entry for every single version.
The same types of changes should be grouped.
Versions and sections should be linkable.
The latest version comes first.
The release date of each version is displayed.
Mention whether you follow Semantic Versioning.

Usage:

Change log entries are to be added to the Unreleased section under the
appropriate stanza (see below). Each entry should ideally include a tag and
the Github issue reference in the following format:

* (<tag>) \#<issue-number> message

The issue numbers will later be link-ified during the release process so you do
not have to worry about including a link manually, but you can if you wish.

Types of changes (Stanzas):

"Features" for new features.
"Improvements" for changes in existing functionality.
"Deprecated" for soon-to-be removed features.
"Bug Fixes" for any bug fixes.
"Client Breaking" for breaking CLI commands and REST routes used by end-users.
"API Breaking" for breaking exported APIs used by developers building on SDK.
"State Machine Breaking" for any changes that result in a different AppState given the same genesisState and txList.
Ref: https://keepachangelog.com/en/1.0.0/
-->

# Changelog

## [[Unreleased]]

### Dependencies

* [\#7114](https://github.com/cosmos/ibc-go/pull/7114) Bump WasmVM to v2.1.2.

### API Breaking

### State Machine Breaking

### Improvements

### Features

### Bug Fixes

<!-- markdown-link-check-disable-next-line -->
## [v0.4.1+ibc-go-v8.4-wasmvm-v2.0](https://github.com/cosmos/ibc-go/releases/tag/modules%2Flight-clients%2F08-wasm%2Fv0.4.1%2Bibc-go-v8.4-wasmvm-v2.0) - 2024-07-31

### API Breaking

* [\#6923](https://github.com/cosmos/ibc-go/pull/6923) The JSON message API for `VerifyMembershipMsg` and `VerifyNonMembershipMsg` payloads for client contract `SudoMsg` has been updated. The field `path` has been changed to `merkle_path`. This change requires updates to 08-wasm client contracts for integration.

<!-- markdown-link-check-disable-next-line -->
## [v0.3.0+ibc-go-v8.3-wasmvm-v2.0](https://github.com/cosmos/ibc-go/releases/tag/modules%2Flight-clients%2F08-wasm%2Fv0.3.0%2Bibc-go-v8.3-wasmvm-v2.0) - 2024-07-17

### Dependencies

* [\#6807](https://github.com/cosmos/ibc-go/pull/6807) Update wasmvm to v2.1.0.

### API Breaking

* [\#6644](https://github.com/cosmos/ibc-go/pull/6644) Add `v2.MerklePath` for contract api `VerifyMembershipMsg` and `VerifyNonMembershipMsg` structs. Note, this requires a migration for existing client contracts to correctly handle deserialization of `MerklePath.KeyPath` which has changed from `[]string` to `[][]bytes`. In JSON message structures this change is reflected as the `KeyPath` being a marshalled as a list of base64 encoded byte strings. This change supports proving values stored under keys which contain non-utf8 encoded symbols. See migration docs for more details.

### Improvements

* [\#5923](https://github.com/cosmos/ibc-go/pull/5923) imp: add 08-wasm build opts for libwasmvm linking disabled 

### Features

* [\#6055](https://github.com/cosmos/ibc-go/pull/6055) feat: add 08-wasm `ConsensusHost` implementation for custom self client/consensus state validation in 03-connection handshake.

### Bug Fixes

* [\#6815](https://github.com/cosmos/ibc-go/pull/6815) Decode to bytes the hex-encoded checksum argument of the `migrate-contract` CLI. 

<!-- markdown-link-check-disable-next-line -->
## [v0.2.0+ibc-go-v8.3-wasmvm-v2.0](https://github.com/cosmos/ibc-go/releases/tag/modules%2Flight-clients%2F08-wasm%2Fv0.2.0%2Bibc-go-v8.3-wasmvm-v2.0) - 2024-05-23

### Dependencies

* [\#5909](https://github.com/cosmos/ibc-go/pull/5909) Update wasmvm to v2.0.0 and cometBFT to v0.38.6.
* [\#6097](https://github.com/cosmos/ibc-go/pull/6097) Update wasmvm to v2.0.1.
* [\#6350](https://github.com/cosmos/ibc-go/pull/6350) Upgrade Cosmos SDK to v0.50.6.

### Features

* [\#5821](https://github.com/cosmos/ibc-go/pull/5821) feat: add `VerifyMembershipProof` RPC query (querier approach for conditional clients).
* [\#6231](https://github.com/cosmos/ibc-go/pull/6231) feat: add CLI to broadcast transaction with `MsgMigrateContract`.

<!-- markdown-link-check-disable-next-line -->
## [v0.1.0+ibc-go-v8.0-wasmvm-v1.5](https://github.com/cosmos/ibc-go/releases/tag/modules%2Flight-clients%2F08-wasm%2Fv0.1.0%2Bibc-go-v7.3-wasmvm-v1.5) - 2023-12-18

### Features

* [\#5079](https://github.com/cosmos/ibc-go/pull/5079) feat: 08-wasm light client proxy module for wasm clients.

<!-- markdown-link-check-disable-next-line -->
## [v0.1.0+ibc-go-v7.3-wasmvm-v1.5](https://github.com/cosmos/ibc-go/releases/tag/modules%2Flight-clients%2F08-wasm%2Fv0.1.0%2Bibc-go-v8.0-wasmvm-v1.5) - 2023-12-18

### Features

* [\#5079](https://github.com/cosmos/ibc-go/pull/5079) feat: 08-wasm light client proxy module for wasm clients.
