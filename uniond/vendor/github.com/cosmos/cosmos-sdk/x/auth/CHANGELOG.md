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

* [#18641](https://github.com/cosmos/cosmos-sdk/pull/18641) Support the ability to broadcast unordered transactions per ADR-070. See UPGRADING.md for more details on integration.
* [#18281](https://github.com/cosmos/cosmos-sdk/pull/18281) Support broadcasting multiple transactions.

### Improvements

* [#19967](https://github.com/cosmos/cosmos-sdk/pull/19967) Refactor ante handlers to use `transaction.Service` for getting exec mode.
* [#18780](https://github.com/cosmos/cosmos-sdk/pull/18780) Move sig verification out of the for loop, into the authenticate method.
* [#19188](https://github.com/cosmos/cosmos-sdk/pull/19188) Remove creation of `BaseAccount` when sending a message to an account that does not exist. 
    * When signing a transaction with an account that has not been created accountnumber 0 must be used
* [#19363](https://github.com/cosmos/cosmos-sdk/pull/19363), [#19370](https://github.com/cosmos/cosmos-sdk/pull/19370) RegisterMigrations, InitGenesis and ExportGenesis return error instead of panic.

### CLI Breaking Changes

* (vesting) [#19539](https://github.com/cosmos/cosmos-sdk/pull/19539) Remove vesting CLI.

### API Breaking Changes

* [#19447](https://github.com/cosmos/cosmos-sdk/pull/19447) Address and validator address codecs are now arguments of `NewTxConfig`. `NewDefaultSigningOptions` has been replaced with `NewSigningOptions` which takes address and validator address codecs as arguments.
* [#17985](https://github.com/cosmos/cosmos-sdk/pull/17985) Remove `StdTxConfig`
* [#19161](https://github.com/cosmos/cosmos-sdk/pull/19161) Remove `simulate` from `SetGasMeter`
* [#19363](https://github.com/cosmos/cosmos-sdk/pull/19363) Remove `IterateAccounts` and `GetAllAccounts` methods from the AccountKeeper interface and Keeper.
* [#19290](https://github.com/cosmos/cosmos-sdk/issues/19290) Pass `appmodule.Environment` to NewKeeper instead of passing individual services. 
* [#19535](https://github.com/cosmos/cosmos-sdk/pull/19535) Remove vesting account creation when the chain is running. The accounts module is required for creating [#vesting accounts](../accounts/defaults/lockup/README.md) on a running chain. 
* [#19600](https://github.com/cosmos/cosmos-sdk/pull/19600) add a consensus query method to the consensus module in order for modules to query consensus for the consensus params. 
* [#19600](https://github.com/cosmos/cosmos-sdk/pull/21403) NewAppModule now takes in `ante.ExtensionOptionChecker`, but it's only used in server/v2 chains, so it's safe to pass in nil for the rest of the users. 
* [#21480](https://github.com/cosmos/cosmos-sdk/pull/21480) ConsensusKeeper is a required keeper for ante handlers.


### Consensus Breaking Changes

* [#18817](https://github.com/cosmos/cosmos-sdk/pull/18817) SigVerification, GasConsumption, IncreaseSequence ante decorators have all been joined into one SigVerification decorator. Gas consumption during TX validation flow has reduced.
* [#19093](https://github.com/cosmos/cosmos-sdk/pull/19093) SetPubKeyDecorator was merged into SigVerification, gas consumption is almost halved for a simple tx.
* [#19535](https://github.com/cosmos/cosmos-sdk/pull/19535) Remove vesting account creation when the chain is running. The accounts module is required for creating [#vesting accounts](../accounts/defaults/lockup/README.md) on a running chain. 
* [#21688](https://github.com/cosmos/cosmos-sdk/pull/21688) Allow x/accounts to be queriable from the `AccountInfo` and `Account` gRPC endpoints
* [#21820](https://github.com/cosmos/cosmos-sdk/pull/21820) Allow x/auth `BaseAccount` to migrate to a `x/accounts` via the new `MsgMigrateAccount`. 
### Bug Fixes

* [#19148](https://github.com/cosmos/cosmos-sdk/pull/19148) Checks the consumed gas for verifying a multisig pubKey signature during simulation.
* [#19239](https://github.com/cosmos/cosmos-sdk/pull/19239) Sets from flag in multi-sign command to avoid no key name provided error.
* [#19099](https://github.com/cosmos/cosmos-sdk/pull/19099) `verifyIsOnCurve` now checks if we are simulating to avoid malformed public key error.
* [#20323](https://github.com/cosmos/cosmos-sdk/pull/20323) Ignore undecodable txs in GetBlocksWithTxs.
* [#20963](https://github.com/cosmos/cosmos-sdk/pull/20963) UseGrantedFees used to return error with raw addresses. Now it uses addresses in string format.

### Deprecated

* (x/auth) [#20531](https://github.com/cosmos/cosmos-sdk/pull/20531) Deprecate auth keeper `NextAccountNumber`, use `keeper.AccountsModKeeper.NextAccountNumber` instead.
