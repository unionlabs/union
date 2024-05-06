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

## [Unreleased]

### Dependencies

* [#\5975](https://github.com/cosmos/ibc-go/pull/5975) Upgrade Cosmos SDK to v0.50.5.
* [#\5909](https://github.com/cosmos/ibc-go/pull/5909) Update wasmvm to v2.0.0 and cometBFT to v0.38.6.
* [#\6097](https://github.com/cosmos/ibc-go/pull/6097) Update wasmvm to v2.0.1.

### API Breaking

### State Machine Breaking

### Improvements

### Features

* [#\5821](https://github.com/cosmos/ibc-go/pull/5821) feat: add `VerifyMembershipProof` RPC query (querier approach for conditional clients).
* [#\6231](https://github.com/cosmos/ibc-go/pull/6231) feat: add CLI to broadcast transaction with `MsgMigrateContract`.

### Bug Fixes

<!-- markdown-link-check-disable-next-line -->
## [v0.1.0+ibc-go-v8.0-wasmvm-v1.5](https://github.com/cosmos/ibc-go/releases/tag/modules%2Flight-clients%2F08-wasm%2Fv0.1.0%2Bibc-go-v7.3-wasmvm-v1.5) - 2023-12-18

### Features

* [\#5079](https://github.com/cosmos/ibc-go/pull/5079) feat: 08-wasm light client proxy module for wasm clients.

<!-- markdown-link-check-disable-next-line -->
## [v0.1.0+ibc-go-v7.3-wasmvm-v1.5](https://github.com/cosmos/ibc-go/releases/tag/modules%2Flight-clients%2F08-wasm%2Fv0.1.0%2Bibc-go-v8.0-wasmvm-v1.5) - 2023-12-18

### Features

* [\#5079](https://github.com/cosmos/ibc-go/pull/5079) feat: 08-wasm light client proxy module for wasm clients.
