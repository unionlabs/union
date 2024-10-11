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

* [#18737](https://github.com/cosmos/cosmos-sdk/pull/18737) Added a limit of 200 grants pruned per `BeginBlock` and the `PruneExpiredGrants` message that prunes 75 expired grants on every run.
* [#20161](https://github.com/cosmos/cosmos-sdk/pull/20161) Added `RevokeAll` method to revoke all grants at once.
* [#20687](https://github.com/cosmos/cosmos-sdk/pull/20687) Prevent user to grant authz MsgGrant to other accounts. Preventing user from accidentally authorizing their entire account to a different account.

### Improvements 

* [#18070](https://github.com/cosmos/cosmos-sdk/pull/18070) Use clientCtx adress codecs in cli.

### API Breaking Changes

* [#21044](https://github.com/cosmos/cosmos-sdk/pull/21044) `k.DispatchActions` returns a slice of byte slices of proto marshaled anys instead of a slice of byte slices of `sdk.Result.Data`.
* [#20502](https://github.com/cosmos/cosmos-sdk/pull/20502) `Accept` on the `Authorization` interface now expects the authz environment in the `context.Context`. This is already done when `Accept` is called by `k.DispatchActions`, but should be done manually if `Accept` is called directly.
* [#19783](https://github.com/cosmos/cosmos-sdk/pull/19783) Removes the use of Accounts String() method
    * `NewMsgExec`, `NewMsgGrant` and `NewMsgRevoke` now takes strings as arguments instead of `sdk.AccAddress`.
    * `ExportGenesis` also returns an error.
    * `IterateGrants` returns an error, its handler function also returns an error.
* [#19637](https://github.com/cosmos/cosmos-sdk/pull/19637) `NewKeeper` doesn't take a message router anymore. Set the message router in the `appmodule.Environment` instead.
    * `Exec` no longer emits duplicate events containing a "authz_msg_index" attribute.
* [#19490](https://github.com/cosmos/cosmos-sdk/pull/19490) `appmodule.Environment` is received on the Keeper to get access to different application services.
* [#18737](https://github.com/cosmos/cosmos-sdk/pull/18737) Update the keeper method `DequeueAndDeleteExpiredGrants` to take a limit argument for the number of grants to prune.
* [#16509](https://github.com/cosmos/cosmos-sdk/pull/16509) `AcceptResponse` has been moved to sdk/types/authz and the `Updated` field is now of the type `sdk.Msg` instead of `authz.Authorization`.
* [#19740](https://github.com/cosmos/cosmos-sdk/pull/19740) `InitGenesis` and `ExportGenesis` module code and keeper code do not panic but return errors.

### Consensus Breaking Changes

* [#19188](https://github.com/cosmos/cosmos-sdk/pull/19188) Remove creation of `BaseAccount` when sending a message to an account that does not exist.

### Bug Fixes

* [#19874](https://github.com/cosmos/cosmos-sdk/pull/19923) Now when querying transaction events (cosmos.tx.v1beta1.Service/GetTxsEvent) the response will contain only UTF-8 characters
