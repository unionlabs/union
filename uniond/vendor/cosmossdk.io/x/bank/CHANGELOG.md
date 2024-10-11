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

* [#17569](https://github.com/cosmos/cosmos-sdk/pull/17569) Introduce a new message type, `MsgBurn`, to burn coins.
* [#20014](https://github.com/cosmos/cosmos-sdk/pull/20014) Support app wiring for `SendRestrictionFn`.

### Improvements

* [#18636](https://github.com/cosmos/cosmos-sdk/pull/18636) `SendCoinsFromModuleToAccount`, `SendCoinsFromModuleToModule`, `SendCoinsFromAccountToModule`, `DelegateCoinsFromAccountToModule`, `UndelegateCoinsFromModuleToAccount`, `MintCoins` and `BurnCoins` methods now returns an error instead of panicking if any module accounts does not exist or unauthorized.
* [#20517](https://github.com/cosmos/cosmos-sdk/pull/20517) `SendCoins` now checks for `SendRestrictions` before instead of after deducting coins using `subUnlockedCoins`.
* [#20354](https://github.com/cosmos/cosmos-sdk/pull/20354) Reduce the number of `ValidateDenom` calls in `bank.SendCoins`.
* [#21976](https://github.com/cosmos/cosmos-sdk/pull/21976) Resolve a footgun by swapping send restrictions check in `InputOutputCoins` before coin deduction.

### Bug Fixes

* [#21407](https://github.com/cosmos/cosmos-sdk/pull/21407) Fix handling of negative spendable balances.
    * The `SpendableBalances` query now correctly reports spendable balances when one or more denoms are negative (used to report all zeros). Also, this query now looks up only the balances for the requested page.
    * The `SpendableCoins` keeper method now returns the positive spendable balances even when one or more denoms have more locked than available (used to return an empty `Coins`).
    * The `SpendableCoin` keeper method now returns a zero coin if there's more locked than available (used to return a negative coin).

### API Breaking Changes

* [#19954](https://github.com/cosmos/cosmos-sdk/pull/19954) Removal of the Address.String() method and related changes:
    * Changed `NewInput`, `NewOutput`, `NewQueryBalanceRequest`, `NewQueryAllBalancesRequest`, `NewQuerySpendableBalancesRequest` to accept a string instead of an `AccAddress`.
    * Added an address codec as an argument to `NewSendAuthorization`.
    * Added an address codec as an argument to `SanitizeGenesisBalances` which also returns an error.
    * (simulation) `RandomGenesisBalances` also returns an error.
* [#17569](https://github.com/cosmos/cosmos-sdk/pull/17569) `BurnCoins` takes an address instead of a module name
* [#19477](https://github.com/cosmos/cosmos-sdk/pull/19477) `appmodule.Environment` is passed to bank `NewKeeper`
* [#19627](https://github.com/cosmos/cosmos-sdk/pull/19627) The genesis api has been updated to match `appmodule.HasGenesis`.
* [#19740](https://github.com/cosmos/cosmos-sdk/pull/19740) `InitGenesis` and `ExportGenesis` module code and keeper code do not panic but return errors.

### Consensus Breaking Changes

* [#19188](https://github.com/cosmos/cosmos-sdk/pull/19188) Remove creation of `BaseAccount` when sending a message to an account that does not exist
