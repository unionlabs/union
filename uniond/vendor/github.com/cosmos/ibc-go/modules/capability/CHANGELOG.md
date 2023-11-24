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
"API Breaking" for breaking exported APIs used by developers building with this module.
"State Machine Breaking" for any changes that result in a different AppState given the same genesisState and txList.
Ref: https://keepachangelog.com/en/1.0.0/
-->

# Changelog

## v1.0.0

### Dependencies

* [\#4068](https://github.com/cosmos/ibc-go/pull/4068) Upgrade capability module to cosmos-sdk v0.51.0

### API Breaking

### State Machine Breaking

### Improvements

* [\#4068](https://github.com/cosmos/ibc-go/pull/4068) Various improvements made to testing to reduce the dependency tree and use new cosmos-sdk test utils.
* [\#4770](https://github.com/cosmos/ibc-go/pull/4770) Save gas on `IsInitialized`, use `Has` in favour of `Get`.

### Features

### Bug Fixes

* [\#15030](https://github.com/cosmos/cosmos-sdk/pull/15030) `InitMemStore` now correctly uses a `NewInfiniteGasMeter` for both `GasMeter` **and** `BlockGasMeter`. This fixes an issue where the `gasMeter` was incremented non-deterministically across validators. See [\#15015](https://github.com/cosmos/cosmos-sdk/issues/15015) for more information.

## Capability in the Cosmos SDK Repository

The capability module was originally released in [v0.40.0](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.40.0) of the Cosmos SDK.
Please see the [Release Notes](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/RELEASE_NOTES.md).

The capability module has been removed from the Cosmos SDK from `v0.50.0` onwards and has been migrated to this repository. 
It will now be maintained as a standalone go module. 

Please refer to the Cosmos SDK repository for historical content.