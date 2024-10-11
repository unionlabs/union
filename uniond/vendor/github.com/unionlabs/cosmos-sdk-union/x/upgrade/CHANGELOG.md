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

### Improvements

* [#19672](https://github.com/cosmos/cosmos-sdk/pull/19672) Follow latest `cosmossdk.io/core` `PreBlock` simplification.

### API Breaking Changes

* [#19443](https://github.com/cosmos/cosmos-sdk/pull/19443) `NewKeeper` takes an `appmodule.Environment` instead of individual services.
* [#21259](https://github.com/cosmos/cosmos-sdk/pull/21259) Upgrade has been migrated to cosrestore.StoreUpgrades. Renaming keys support has been removed from the upgrade module.
* [#21480](https://github.com/cosmos/cosmos-sdk/pull/21480) ConsensusKeeper is required to be passed to the keeper. 

### State Machine Breaking

* (x/upgrade) [#16244](https://github.com/cosmos/cosmos-sdk/pull/16244) Upgrade module no longer stores the app version but gets and sets the app version stored in the `ParamStore` of baseapp.

## [v0.1.4](https://github.com/cosmos/cosmos-sdk/releases/tag/x/upgrade/v0.1.4) - 2024-07-15

### Improvements

* [#20771](https://github.com/cosmos/cosmos-sdk/pull/20771) Create upgrade directory only when necessary (upgrade flow and not init flow).

## [v0.1.3](https://github.com/cosmos/cosmos-sdk/releases/tag/x/upgrade/v0.1.3) - 2024-06-04

* (deps) [#20530](https://github.com/cosmos/cosmos-sdk/pull/20530) Bump vulnerable `github.com/hashicorp/go-getter` to v1.7.4.
    * In theory, we do not need a new release for this, but we are doing it as `x/upgrade` is used in Cosmovisor.

## [v0.1.2](https://github.com/cosmos/cosmos-sdk/releases/tag/x/upgrade/v0.1.2) - 2024-04-22

### Improvements

* (deps) [#19810](https://github.com/cosmos/cosmos-sdk/pull/19810) Upgrade SDK version due to Prometheus breaking change.
* (deps) [#19810](https://github.com/cosmos/cosmos-sdk/pull/19810) Bump `cosmossdk.io/store` to v1.1.0.

### Bug Fixes

* [#19706](https://github.com/cosmos/cosmos-sdk/pull/19706) Stop treating inline JSON as a URL.

## [v0.1.1](https://github.com/cosmos/cosmos-sdk/releases/tag/x/upgrade/v0.1.1) - 2023-12-11

### Improvements

* [#18470](https://github.com/cosmos/cosmos-sdk/pull/18470) Improve go-getter settings.

## [v0.1.0](https://github.com/cosmos/cosmos-sdk/releases/tag/x/upgrade/v0.1.0) - 2023-11-07

### Features

* [#14880](https://github.com/cosmos/cosmos-sdk/pull/14880) Switch from using gov v1beta1 to gov v1 in upgrade CLIs.
* [#14764](https://github.com/cosmos/cosmos-sdk/pull/14764) The `x/upgrade` module is extracted to have a separate go.mod file which allows it be a standalone module.

### Improvements

* [#16903](https://github.com/cosmos/cosmos-sdk/pull/16903) Use AutoCLI for upgrade querying commands.

### API Breaking Changes

* [#16845](https://github.com/cosmos/cosmos-sdk/pull/16845) Remove gov v1beta1 handler. Use gov v1 proposals directly, or replicate the handler in your app.
* [#16511](https://github.com/cosmos/cosmos-sdk/pull/16511) `BinaryDownloadURLMap.ValidateBasic()` and `BinaryDownloadURLMap.CheckURLs` now both take a checksum parameter when willing to ensure a checksum is provided for each URL.
* [#16511](https://github.com/cosmos/cosmos-sdk/pull/16511) `plan.DownloadURLWithChecksum` has been renamed to `plan.DownloadURL` and does not validate the URL anymore. Call `plan.ValidateURL` before calling `plan.DownloadURL` to validate the URL.
* [#16511](https://github.com/cosmos/cosmos-sdk/pull/16511) `plan.DownloadUpgrade` does not validate URL anymore. Call `plan.ValidateURL` before calling `plan.DownloadUpgrade` to validate the URL.
* [#16227](https://github.com/cosmos/cosmos-sdk/issues/16227) `NewKeeper` now takes a `KVStoreService` instead of a `StoreKey`, methods in the `Keeper` now take a `context.Context` instead of a `sdk.Context` and return an `error`. `UpgradeHandler` now receives a `context.Context`. `GetUpgradedClient`, `GetUpgradedConsensusState`, `GetUpgradePlan` now return a specific error for "not found".


### Bug Fixes

* [#17421](https://github.com/cosmos/cosmos-sdk/pull/17421) Replace `BeginBlock` by `PreBlock`. Read [ADR-68](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-068-preblock.md) for more information.
