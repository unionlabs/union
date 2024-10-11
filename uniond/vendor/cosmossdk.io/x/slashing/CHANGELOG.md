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

### Features

### Improvements

* [#19458](https://github.com/cosmos/cosmos-sdk/pull/19458) Avoid writing SignInfo's for validators who did not miss a block. (Every BeginBlock)
* [#18959](https://github.com/cosmos/cosmos-sdk/pull/18959) Avoid deserialization of parameters with every validator lookup
* [#18636](https://github.com/cosmos/cosmos-sdk/pull/18636) `JailUntil` and `Tombstone` methods no longer panics if the signing info does not exist for the validator but instead returns error.

### API Breaking Changes

* [#20238](https://github.com/cosmos/cosmos-sdk/pull/20238) `NewAppModule` now takes in a `core/comet.Service` an argument.  `BeginBlocker` now takes in a `core/comet.Service`.
* [#20026](https://github.com/cosmos/cosmos-sdk/pull/20026) Removal of the Address.String() method and related changes:
    * `Migrate` now takes a `ValidatorAddressCodec` as argument.
    * `Migrator` has a new field of `ValidatorAddressCodec` type.
* [#18115](https://github.com/cosmos/cosmos-sdk/pull/18115) `NewValidatorSigningInfo` takes strings instead of `sdk.AccAddress`.
* [#16441](https://github.com/cosmos/cosmos-sdk/pull/16441) Params state is migrated to collections. `GetParams` has been removed.
* [#17023](https://github.com/cosmos/cosmos-sdk/pull/17023) Use collections for `ValidatorSigningInfo`:
    * remove `Keeper`: `SetValidatorSigningInfo`, `GetValidatorSigningInfo`, `IterateValidatorSigningInfos`
* [#17044](https://github.com/cosmos/cosmos-sdk/pull/17044) Use collections for `AddrPubkeyRelation`:
    * remove from `types`: `AddrPubkeyRelationKey`
    * remove from `Keeper`: `AddPubkey`
* [#19440](https://github.com/cosmos/cosmos-sdk/pull/19440) Slashing Module creation takes `appmodule.Environment` instead of individual services
* [#19458](https://github.com/cosmos/cosmos-sdk/pull/19458) ValidatorSigningInfo.IndexOffset is deprecated, and no longer used. The index is now derived using just the StartHeight.
* [#19740](https://github.com/cosmos/cosmos-sdk/pull/19740) `InitGenesis` and `ExportGenesis` module code and keeper code do not panic but return errors.

### Bug Fixes
