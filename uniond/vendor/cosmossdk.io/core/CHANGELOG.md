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
"Client Breaking" for breaking Protobuf, gRPC and REST routes used by end-users.
"CLI Breaking" for breaking CLI commands.
"API Breaking" for breaking exported APIs used by developers building on SDK.
Ref: https://keepachangelog.com/en/1.0.0/
-->

# Changelog

## [Unreleased]

<!-- ## [v1.0.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv1.0.0) -->
## [v1.0.0-alpha.1](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv1.0.0-alpha.1)

### Features

* [#21531](https://github.com/cosmos/cosmos-sdk/pull/21531) Add `registry.AminoRegistrar` to register types on the amino codec from modules.
* [#21222](https://github.com/cosmos/cosmos-sdk/pull/21222) Make `Iterator` a type alias so that `KVStore` is structurally typed.
* [#21166](https://github.com/cosmos/cosmos-sdk/pull/21166) Comment out `appmodule.HasServices` to simplify dependencies. This interface is however still supported.
* [#19953](https://github.com/cosmos/cosmos-sdk/pull/19953) Add transaction service.
* [#18379](https://github.com/cosmos/cosmos-sdk/pull/18379) Add branch service.
* [#18457](https://github.com/cosmos/cosmos-sdk/pull/18457) Add branch.ExecuteWithGasLimit.
* [#19041](https://github.com/cosmos/cosmos-sdk/pull/19041) Add `appmodule.Environment` interface to fetch different services
* [#19370](https://github.com/cosmos/cosmos-sdk/pull/19370) Add `appmodule.Migrations` interface to handle migrations
* [#19571](https://github.com/cosmos/cosmos-sdk/pull/19571) Add `router.Service` and add it in `appmodule.Environment`
* [#19617](https://github.com/cosmos/cosmos-sdk/pull/19617) Server/v2 compatible interface:
    * Add DataBaseService to store non-consensus data in a database
    * Create V2 appmodule with v2 api for runtime/v2
    * Introduce `Transaction.Tx` for use in runtime/v2
    * Introduce `HasUpdateValidators` interface and `ValidatorUpdate` struct for validator updates 
    * Introduce `HasTxValidation` interface for modules to register tx validation handlers
    * `HasGenesis` interface for modules to register import, export, validation and default genesis handlers. The new api works with `proto.Message`
    * Add `PreMsghandler`and `PostMsgHandler` for pre and post message hooks
    * Add `MsgHandler` as an alternative to grpc handlers
    * Provide separate `MigrationRegistrar` instead of grouping with `RegisterServices`
* [#19758](https://github.com/cosmos/cosmos-sdk/pull/19758) Add `registry.InterfaceRegistrar` to interact with the interface registry in modules.

### API Breaking Changes

* [#19672](https://github.com/cosmos/cosmos-sdk/pull/19672) `PreBlock` now returns only an error for consistency with server/v2. The SDK has upgraded x/upgrade accordingly.
* [#18857](https://github.com/cosmos/cosmos-sdk/pull/18857) Moved `FormatCoins` to `x/tx`.
* [#18861](https://github.com/cosmos/cosmos-sdk/pull/18861) Moved `coin.ParseCoin` to `client/v2/internal`.
* [#18866](https://github.com/cosmos/cosmos-sdk/pull/18866) All items related to depinject have been moved to `cosmossdk.io/depinject` (`Provide`, `Invoke`, `Register`)
* [#19041](https://github.com/cosmos/cosmos-sdk/pull/19041) `HasEventListeners` was removed from appmodule due to the fact that it was not used anywhere in the SDK nor implemented
* [#17689](https://github.com/cosmos/cosmos-sdk/pull/17689) Move Comet service to return structs instead of interfaces. 
    * `BlockInfo` was renamed to `Info` and `BlockInfoService` was renamed to `CometInfoService`
* [#17693](https://github.com/cosmos/cosmos-sdk/pull/17693) Remove `appmodule.UpgradeModule` interface in favor of preblock

## [v0.11.1](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.11.1)

* [#21022](https://github.com/cosmos/cosmos-sdk/pull/21022) Upgrade depinject to v1.0.0.

## [v0.11.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.11.0)

* [#17468](https://github.com/cosmos/cosmos-sdk/pull/17468) Add `appmodule.HasPreBlocker` interface.

## [v0.10.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.10.0)

* [#17383](https://github.com/cosmos/cosmos-sdk/pull/17383) Add `appmodule.UpgradeModule` interface.

## [v0.9.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.9.0)

* [#16739](https://github.com/cosmos/cosmos-sdk/pull/16739) Add `AppHash` to header.Info.

## [v0.8.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.8.0)

* [#15519](https://github.com/cosmos/cosmos-sdk/pull/15519) Update `comet.VoteInfo` for CometBFT v0.38.
* [#16310](https://github.com/cosmos/cosmos-sdk/pull/16310) Add `gas.Service` and `gas.GasMeter` interfaces.

## [v0.7.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.7.0)

* [#15850](https://github.com/cosmos/cosmos-sdk/pull/15850) Add `comet` and `header` packages.
* [#15923](https://github.com/cosmos/cosmos-sdk/pull/15923) Add `appmodule.HasPrepareCheckState` `appmodule.HasPrecommit` extension interfaces.
* [#15434](https://github.com/cosmos/cosmos-sdk/pull/15434) Add `coin.ParseCoin` for parsing a coin from a string.
* [#15999](https://github.com/cosmos/cosmos-sdk/pull/15999) Add `genesis.GenesisTxHandler` interface.

## [v0.6.1](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.6.1)

* [#15364](https://github.com/cosmos/cosmos-sdk/pull/15364) Add address codec to core.

## [v0.6.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.6.0)

* [#15045](https://github.com/cosmos/cosmos-sdk/pull/15045) Add error return parameter to `RegisterServices` method from `appmodule.HasServices` interface.
* [#14859](https://github.com/cosmos/cosmos-sdk/pull/14859) Simplify event service interface.

## [v0.5.1](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.5.1)

* [#14686](https://github.com/cosmos/cosmos-sdk/pull/14686) Add event service.
* [#14735](https://github.com/cosmos/cosmos-sdk/pull/14735) Specify event listener API.

## [v0.5.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.5.0)

* [#14604](https://github.com/cosmos/cosmos-sdk/pull/14604) Add begin/end block extension interfaces.
* [#14605](https://github.com/cosmos/cosmos-sdk/pull/14605) Add register services extension interface.

## [v0.4.1](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.4.1)

* [#14329](https://github.com/cosmos/cosmos-sdk/pull/14329) Implement basic core API genesis source and target.

## [v0.4.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.4.0)

* [#14416](https://github.com/cosmos/cosmos-sdk/pull/14416) Update core to use cosmos-db instead of tm-db.
* [#14326](https://github.com/cosmos/cosmos-sdk/pull/14326) Remove `appmodule.Service` from core.

## [v0.3.4](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.3.4)

* [#14223](https://github.com/cosmos/cosmos-sdk/pull/14223) Add genesis API.

## [v0.3.3](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.3.3)

* [#14227](https://github.com/cosmos/cosmos-sdk/pull/14227) Add store API.
* [#13696](https://github.com/cosmos/cosmos-sdk/pull/13696) Update `FormatCoins` where empty coins are rendered as "zero".

## [v0.3.2](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.3.2)

* [#13631](https://github.com/cosmos/cosmos-sdk/pull/13631) Add ADR 033 (inter-module communication) Client interface.

## [v0.3.1](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.3.1)

* [#13306](https://github.com/cosmos/cosmos-sdk/pull/13306) Move `FormatCoins` to core.
* [#13607](https://github.com/cosmos/cosmos-sdk/pull/13115) Add `AppModule` tag interface.

## [v0.3.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.3.0)

* [#13115](https://github.com/cosmos/cosmos-sdk/pull/13115) Update core module to depinject changes.

## [v0.2.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.2.0)

* [#12020](https://github.com/cosmos/cosmos-sdk/pull/12020) Use cosmossdk.io/depinject instead of container.
* [#12367](https://github.com/cosmos/cosmos-sdk/pull/12367) Add support for golang_bindings in app.yaml.

## [v0.1.0](https://github.com/cosmos/cosmos-sdk/releases/tag/core%2Fv0.1.0)

* [#11914](https://github.com/cosmos/cosmos-sdk/pull/11914) Add core module with app config support.
