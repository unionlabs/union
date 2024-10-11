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

* [#18448](https://github.com/cosmos/cosmos-sdk/pull/18448) Extend group config
* [18286](https://github.com/cosmos/cosmos-sdk/pull/18286) Move prefix store creation down after error checks.

### API Breaking Changes

* [#20082](https://github.com/cosmos/cosmos-sdk/pull/20082) Removes the use of `MustAccAddressFromBech32`:
    * `PrimaryKeyFields` function from interface `PrimaryKeyed` now takes an address codec as argument.
    * `PrimaryKey`, `NewAutoUInt64Table` and `NewPrimaryKeyTable` now take an address codec as argument.
* [#19916](https://github.com/cosmos/cosmos-sdk/pull/19916) Removes the use of Address String methods:
    * `NewMsgCreateGroupPolicy` now takes a string as argument instead of an `AccAddress`.
    * `NewMsgUpdateGroupPolicyDecisionPolicy` now takes strings as argument instead of `AccAddress`.
    * `NewGroupPolicyInfo` address and admin arguments are now strings instead of `AccAddress`.
    * `MigrateGenState` now takes an address codec as argument.
* [#19638](https://github.com/cosmos/cosmos-sdk/pull/19638) Migrate module to use `appmodule.Environment` router service so no `baseapp.MessageRouter` is required is `NewKeeper` anymore.
* [#19489](https://github.com/cosmos/cosmos-sdk/pull/19489) `appmodule.Environment` is received on the Keeper to get access to different application services.
* [#19410](https://github.com/cosmos/cosmos-sdk/pull/19410) Migrate to Store Service.
* [#19740](https://github.com/cosmos/cosmos-sdk/pull/19740) `InitGenesis` and `ExportGenesis` module code and keeper code do not panic but return errors.
