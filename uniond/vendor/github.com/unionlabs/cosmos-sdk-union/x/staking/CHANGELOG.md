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

* [#20688](https://github.com/cosmos/cosmos-sdk/pull/20688) Avoid overslashing unbonding delegations after a redelegation.
* [#19226](https://github.com/cosmos/cosmos-sdk/pull/19226) Ensure `GetLastValidators` in `x/staking` does not return an error when `MaxValidators` exceeds total number of bonded validators.

### Features

* [#19537](https://github.com/cosmos/cosmos-sdk/pull/19537) Changing `MinCommissionRate` in `MsgUpdateParams` now updates the minimum commission rate for all validators.
* [#20434](https://github.com/cosmos/cosmos-sdk/pull/20434) Add consensus address to validator query response

### Improvements

* [#19779](https://github.com/cosmos/cosmos-sdk/pull/19779) Allows for setting `unbonding_time` to zero.

* [#19277](https://github.com/cosmos/cosmos-sdk/pull/19277) Hooks calls on `SetUnbondingDelegationEntry`, `SetRedelegationEntry`, `Slash` and `RemoveValidator` returns errors instead of logging just like other hooks calls.
* [#18636](https://github.com/cosmos/cosmos-sdk/pull/18636) `IterateBondedValidatorsByPower`, `GetDelegatorBonded`, `Delegate`, `Unbond`, `Slash`, `Jail`, `SlashRedelegation`, `ApplyAndReturnValidatorSetUpdates` methods no longer panics on any kind of errors but instead returns appropriate errors.
* [#18506](https://github.com/cosmos/cosmos-sdk/pull/18506) Detect the length of the ed25519 pubkey in CreateValidator to prevent panic.


### API Breaking Changes

* [#20238](https://github.com/cosmos/cosmos-sdk/pull/20238) `NewKeeper` now accepts a `core/comet.Service` as its last argument. 
* [#19788](https://github.com/cosmos/cosmos-sdk/pull/19788) Remove `ABCIValidatorUpdate` and `ABCIValidatorUpdateZero`, use `ModuleValidatorUpdate` and `ModuleValidatorUpdateIsZero` instead.
* [#19754](https://github.com/cosmos/cosmos-sdk/pull/19754) Update to use `[]appmodule.ValidatorUpdate` as return for `ApplyAndReturnValidatorSetUpdates`.
* [#19414](https://github.com/cosmos/cosmos-sdk/pull/19414) `NewStakingKeeper` takes an environment variable instead of individual services.
* [#19742](https://github.com/cosmos/cosmos-sdk/pull/19742) `NewStakeAuthorization` now takes `address.Codec` as argument.
* [#19735](https://github.com/cosmos/cosmos-sdk/pull/19735) Update genesis api to match new `appmodule.HasGenesis` interface.
* [#18198](https://github.com/cosmos/cosmos-sdk/pull/18198) `Validator` and `Delegator` interfaces were moved to `github.com/cosmos/cosmos-sdk/types` to avoid interface dependency on staking in other modules.
* [#17778](https://github.com/cosmos/cosmos-sdk/pull/17778) Use collections for `Params`
    * remove from `Keeper`: `GetParams`, `SetParams`
* [#17486](https://github.com/cosmos/cosmos-sdk/pull/17486) Use collections for `RedelegationQueueKey`:
    * remove from `types`: `GetRedelegationTimeKey`
    * remove from `Keeper`: `RedelegationQueueIterator`
* [#17562](https://github.com/cosmos/cosmos-sdk/pull/17562) Use collections for `ValidatorQueue`
    * remove from `types`: `GetValidatorQueueKey`, `ParseValidatorQueueKey`
    * remove from `Keeper`: `ValidatorQueueIterator`
* [#17498](https://github.com/cosmos/cosmos-sdk/pull/17498) Use collections for `LastValidatorPower`:
    * remove from `types`: `GetLastValidatorPowerKey`
    * remove from `Keeper`: `LastValidatorsIterator`, `IterateLastValidators`
* [#17291](https://github.com/cosmos/cosmos-sdk/pull/17291) Use collections for `UnbondingDelegationByValIndex`:
    * remove from `types`: `GetUBDKeyFromValIndexKey`, `GetUBDsByValIndexKey`, `GetUBDByValIndexKey`
* (x/slashing) [#17568](https://github.com/cosmos/cosmos-sdk/pull/17568) Use collections for `ValidatorMissedBlockBitmap`:
    * remove from `types`: `ValidatorMissedBlockBitmapPrefixKey`, `ValidatorMissedBlockBitmapKey`
* [#17481](https://github.com/cosmos/cosmos-sdk/pull/17481) Use collections for `UnbondingQueue`:
    * remove from `Keeper`: `UBDQueueIterator`
    * remove from `types`: `GetUnbondingDelegationTimeKey`
* [#17123](https://github.com/cosmos/cosmos-sdk/pull/17123) Use collections for `Validators`
* [#17270](https://github.com/cosmos/cosmos-sdk/pull/17270) Use collections for `UnbondingDelegation`:
    * remove from `types`: `GetUBDsKey`
    * remove from `Keeper`: `IterateUnbondingDelegations`, `IterateDelegatorUnbondingDelegations`
* [#17336](https://github.com/cosmos/cosmos-sdk/pull/17336) Use collections for `RedelegationByValDstIndexKey`:
    * remove from `types`: `GetREDByValDstIndexKey`, `GetREDsToValDstIndexKey`
* [#17332](https://github.com/cosmos/cosmos-sdk/pull/17332) Use collections for `RedelegationByValSrcIndexKey`:
    * remove from `types`: `GetREDKeyFromValSrcIndexKey`, `GetREDsFromValSrcIndexKey`
* [#17315](https://github.com/cosmos/cosmos-sdk/pull/17315) Use collections for `RedelegationKey`:
    * remove from `keeper`: `GetRedelegation`
* [#17260](https://github.com/cosmos/cosmos-sdk/pull/17260) Use collections for `DelegationKey`:
    * remove from `types`: `GetDelegationKey`, `GetDelegationsKey`
* [#17288](https://github.com/cosmos/cosmos-sdk/pull/17288) Use collections for `UnbondingIndex`:
    * remove from `types`: `GetUnbondingIndexKey`.
* [#17256](https://github.com/cosmos/cosmos-sdk/pull/17256) Use collections for `UnbondingID`.
* [#17260](https://github.com/cosmos/cosmos-sdk/pull/17260) Use collections for `ValidatorByConsAddr`:
    * remove from `types`: `GetValidatorByConsAddrKey`
* [#17248](https://github.com/cosmos/cosmos-sdk/pull/17248) Use collections for `UnbondingType`.
    * remove from `types`: `GetUnbondingTypeKey`.
* [#17062](https://github.com/cosmos/cosmos-sdk/pull/17062), [#19788](https://github.com/cosmos/cosmos-sdk/pull/19788) Remove `GetValidatorUpdates` and `ValidatorUpdates` storage.
* [#17026](https://github.com/cosmos/cosmos-sdk/pull/17026) Use collections for `LastTotalPower`:
    * remove `Keeper`: `SetLastTotalPower`, `GetLastTotalPower`
* [#17335](https://github.com/cosmos/cosmos-sdk/pull/17335) Remove usage of `"cosmossdk.io/x/staking/types".Infraction_*` in favour of `"cosmossdk.io/api/cosmos/staking/v1beta1".Infraction_` in order to remove dependency between modules on staking
* [#20295](https://github.com/cosmos/cosmos-sdk/pull/20295) `GetValidatorByConsAddr` now returns the Cosmos SDK `cryptotypes.Pubkey` instead of `cometcrypto.Publickey`. The caller is responsible to translate the returned value to the expected type. 
    * Remove `CmtConsPublicKey()` and `TmConsPublicKey()` from `Validator` interface and as methods on the `Validator` struct.
* [#21480](https://github.com/cosmos/cosmos-sdk/pull/21480) ConsensusKeeper is required to be passed to the keeper. 

### State Breaking changes

* [#18841](https://github.com/cosmos/cosmos-sdk/pull/18841) In a undelegation or redelegation if the shares being left delegated correspond to less than 1 token (in base denom) the entire delegation gets removed.
* [#18142](https://github.com/cosmos/cosmos-sdk/pull/18142) Introduce `key_rotation_fee` param to calculate fees while rotating the keys
* [#19740](https://github.com/cosmos/cosmos-sdk/pull/19740) `InitGenesis` and `ExportGenesis` module code and keeper code do not panic but return errors.
* [#20845](https://github.com/cosmoc/cosmos-sdk/pull/20845) Remove HistoricalInfo from the staking modules storage
