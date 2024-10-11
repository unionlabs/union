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

*

### Improvements

* [#20790](https://github.com/cosmos/cosmos-sdk/pull/20790) `x/distribution` does not depend on `x/protocolpool` anymore, now `x/distribution` only does token transfers and `x/protocolpool` does the rest.

### API Breaking Changes

* [#20790](https://github.com/cosmos/cosmos-sdk/pull/20790) `x/distribution` does not depend on `x/protocolpool` anymore, meaning NewAppModule and NewKeeper do not take it as an argument.
* [#20588](https://github.com/cosmos/cosmos-sdk/pull/20588) `x/distribution` now takes cometService in order to get consensus related information.
* [#19868](https://github.com/cosmos/cosmos-sdk/pull/19868) Removes Accounts String method
    * `NewMsgSetWithdrawAddress` now takes strings as argument instead of `sdk.AccAddress`.
    * `NewGenesisState` now takes a string as argument instead of `sdk.ConsAddress`.
* [#19445](https://github.com/cosmos/cosmos-sdk/pull/19445) `appmodule.Environment` is received on the Keeper to get access to different application services
* [#17115](https://github.com/cosmos/cosmos-sdk/pull/17115) Use collections for `PreviousProposer` and `ValidatorSlashEvents`:
    * remove from `Keeper`: `GetPreviousProposerConsAddr`, `SetPreviousProposerConsAddr`, `GetValidatorHistoricalReferenceCount`, `GetValidatorSlashEvent`, `SetValidatorSlashEvent`.
* [#16483](https://github.com/cosmos/cosmos-sdk/pull/16483) use collections for `DelegatorStartingInfo` state management:
    * remove `Keeper`: `IterateDelegatorStartingInfo`, `GetDelegatorStartingInfo`, `SetDelegatorStartingInfo`, `DeleteDelegatorStartingInfo`, `HasDelegatorStartingInfo`
* [#16571](https://github.com/cosmos/cosmos-sdk/pull/16571) use collections for `ValidatorAccumulatedCommission` state management:
    * remove `Keeper`: `IterateValidatorAccumulatedCommission`, `GetValidatorAccumulatedCommission`, `SetValidatorAccumulatedCommission`, `DeleteValidatorAccumulatedCommission`
* [#16590](https://github.com/cosmos/cosmos-sdk/pull/16590) use collections for `ValidatorOutstandingRewards` state management:
    * remove `Keeper`: `IterateValidatorOutstandingRewards`, `GetValidatorOutstandingRewards`, `SetValidatorOutstandingRewards`, `DeleteValidatorOutstandingRewards`
* [#16607](https://github.com/cosmos/cosmos-sdk/pull/16607) use collections for `ValidatorHistoricalRewards` state management:
    * remove `Keeper`: `IterateValidatorHistoricalRewards`, `GetValidatorHistoricalRewards`, `SetValidatorHistoricalRewards`, `DeleteValidatorHistoricalRewards`, `DeleteValidatorHistoricalReward`, `DeleteAllValidatorHistoricalRewards`
* [#17657](https://github.com/cosmos/cosmos-sdk/pull/17657) The `FundCommunityPool` and `DistributeFromFeePool` keeper methods are now removed from x/distribution.
* [#16440](https://github.com/cosmos/cosmos-sdk/pull/16440) use collections for `DelegatorWithdrawAddresState`:
    * remove `Keeper`: `SetDelegatorWithdrawAddr`, `DeleteDelegatorWithdrawAddr`, `IterateDelegatorWithdrawAddrs`.
* [#16459](https://github.com/cosmos/cosmos-sdk/pull/16459) use collections for `ValidatorCurrentRewards` state management:
    * remove `Keeper`: `IterateValidatorCurrentRewards`, `GetValidatorCurrentRewards`, `SetValidatorCurrentRewards`, `DeleteValidatorCurrentRewards`
* [#17657](https://github.com/cosmos/cosmos-sdk/pull/17657) ~The distribution module keeper now takes a new argument `PoolKeeper` in addition.~ Reverted on #20790
* [#17670](https://github.com/cosmos/cosmos-sdk/pull/17670) `AllocateTokens` takes `comet.VoteInfos` instead of `[]abci.VoteInfo`
* [#19740](https://github.com/cosmos/cosmos-sdk/pull/19740) `InitGenesis` and `ExportGenesis` module code and keeper code do not panic but return errors.

### Improvements

* [#18636](https://github.com/cosmos/cosmos-sdk/pull/18636) `CalculateDelegationRewards` and `DelegationTotalRewards` methods no longer panics on any sanity checks and instead returns appropriate errors.

### CLI Breaking Changes

* [#17963](https://github.com/cosmos/cosmos-sdk/pull/17963) `appd tx distribution withdraw-rewards` now only withdraws rewards for the delegator's own delegations. For withdrawing validators commission, use `appd tx distribution withdraw-validator-commission`.

### State Machine Breaking

* [#20735](https://github.com/cosmos/cosmos-sdk/pull/20735) Remove PreviousProposer from the state machine.
* [#17657](https://github.com/cosmos/cosmos-sdk/pull/17657) Migrate community pool funds from `x/distribution` to `x/protocolpool`.
* [#17115](https://github.com/cosmos/cosmos-sdk/pull/17115) Migrate `PreviousProposer` to collections.
* [#18539](https://github.com/cosmos/cosmos-sdk/pull/18539) Introduce `FeePool.DecimalPool` to replace `FeePool.CommunityPool`, which temporarily holds fractional rewards until they are distributed to the community pool every 1000 blocks.

### Client Breaking Changes

* [#17657](https://github.com/cosmos/cosmos-sdk/pull/17657) Deprecate `CommunityPool` and `FundCommunityPool` rpc methods. Use `x/protocolpool` module's rpc methods instead.

### Bug Fixes

* [#19301](https://github.com/cosmos/cosmos-sdk/pull/19301) Fix vulnerability in `incrementReferenceCount` in distribution.
