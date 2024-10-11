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
* (<tag>) [#<issue-number>] Changelog message.
Types of changes (Stanzas):
"Features" for new features.
"Improvements" for changes in existing functionality.
"Deprecated" for soon-to-be removed features.
"Bug Fixes" for any bug fixes.
"API Breaking" for breaking exported APIs used by developers building on SDK.
Ref: https://keepachangelog.com/en/1.0.0/
-->

# Changelog

## [Unreleased]

### Bug Fixes

* (store) [#20425](https://github.com/cosmos/cosmos-sdk/pull/20425) Fix nil pointer panic when query historical state where a new store don't exist.
* (store) [#20644](https://github.com/cosmos/cosmos-sdk/pull/20644) Avoid nil error on not exhausted payload stream.

## v1.1.0 (March 20, 2024)

### Improvements

* [#19770](https://github.com/cosmos/cosmos-sdk/pull/19770) Upgrade IAVL to IAVL v1.1.1.

## v1.0.2 (January 10, 2024)

### Bug Fixes

* [#18897](https://github.com/cosmos/cosmos-sdk/pull/18897) Replace panic in pruning to avoid consensus halting. 

## v1.0.1 (November 28, 2023)

### Bug Fixes

* [#18563](https://github.com/cosmos/cosmos-sdk/pull/18563) `LastCommitID().Hash` will always return `sha256([]byte{})` if the store is empty.

## v1.0.0 (October 31, 2023)

### Features

* [#17294](https://github.com/cosmos/cosmos-sdk/pull/17294) Add snapshot manager Close method.
* [#15568](https://github.com/cosmos/cosmos-sdk/pull/15568) Migrate the `iavl` to the new key format.
    * Remove `DeleteVersion`, `DeleteVersions`, `LazyLoadVersionForOverwriting` from `iavl` tree API.
    * Add `DeleteVersionsTo` and `SaveChangeSet`, since it will keep versions sequentially like `fromVersion` to `toVersion`.
    * Refactor the pruning manager to use `DeleteVersionsTo`.
* [#15712](https://github.com/cosmos/cosmos-sdk/pull/15712) Add `WorkingHash` function to the store interface  to get the current app hash before commit.
* [#14645](https://github.com/cosmos/cosmos-sdk/pull/14645) Add limit to the length of key and value.
* [#15683](https://github.com/cosmos/cosmos-sdk/pull/15683) `rootmulti.Store.CacheMultiStoreWithVersion` now can handle loading archival states that don't persist any of the module stores the current state has.
* [#16060](https://github.com/cosmos/cosmos-sdk/pull/16060) Support saving restoring snapshot locally.
* [#14746](https://github.com/cosmos/cosmos-sdk/pull/14746) The `store` module is extracted to have a separate go.mod file which allows it be a standalone module.
* [#14410](https://github.com/cosmos/cosmos-sdk/pull/14410) `rootmulti.Store.loadVersion` has validation to check if all the module stores' height is correct, it will error if any module store has incorrect height.

### Improvements

* [#17158](https://github.com/cosmos/cosmos-sdk/pull/17158) Start the goroutine after need to create a snapshot.

### API Breaking Changes

* [#16321](https://github.com/cosmos/cosmos-sdk/pull/16321) QueryInterface defines its own request and response types instead of relying on comet/abci & returns an error
