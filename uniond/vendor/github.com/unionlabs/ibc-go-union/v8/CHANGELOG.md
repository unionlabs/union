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
"Client Breaking" for breaking CLI commands and REST routes used by end-users.
"API Breaking" for breaking exported APIs used by developers building on SDK.
"State Machine Breaking" for any changes that result in a different AppState given the same genesisState and txList.
Ref: https://keepachangelog.com/en/1.0.0/
-->

# Changelog

## [v8.5.1](https://github.com/cosmos/ibc-go/releases/tag/v8.5.1) - 2024-09-13

### Bug Fixes

* (apps/27-interchain-accounts) [\#7277](https://github.com/cosmos/ibc-go/pull/7277) Use `GogoResolver` when populating module query safe allow list to avoid panics from unresolvable protobuf dependencies.

## [v8.5.0](https://github.com/cosmos/ibc-go/releases/tag/v8.5.0) - 2024-08-30

### Dependencies

* [\#6828](https://github.com/cosmos/ibc-go/pull/6828) Bump Cosmos SDK to v0.50.9.
* [\#7222](https://github.com/cosmos/ibc-go/pull/7221) Update ics23 to v0.11.0.

### State Machine Breaking

* (core/03-connection) [\#7129](https://github.com/cosmos/ibc-go/pull/7129) Remove verification of self client and consensus state from connection handshake. 

## [v8.4.0](https://github.com/cosmos/ibc-go/releases/tag/v8.4.0) - 2024-07-29

### Improvements

* (core/04-channel) [\#6871](https://github.com/cosmos/ibc-go/pull/6871) Add channel ordering to write acknowledgement event.

### Features

* (apps/transfer) [\#6877](https://github.com/cosmos/ibc-go/pull/6877) Added the possibility to transfer the entire user balance of a particular denomination by using [`UnboundedSpendLimit`](https://github.com/cosmos/ibc-go/blob/cdaaa94546af79760317f22bc93f270e3d754c84/modules/apps/transfer/types/coin.go#L60-L62) as the token amount.

### Bug Fixes

* (core/04-channel) [\#6935](https://github.com/cosmos/ibc-go/pull/6935) Check upgrade compatibility in `ChanUpgradeConfirm`.

## [v8.3.2](https://github.com/cosmos/ibc-go/releases/tag/v8.3.2) - 2024-06-20

### Dependencies

* [\#6614](https://github.com/cosmos/ibc-go/pull/6614) Bump Cosmos SDK to v0.50.7.

### Improvements

* (apps/27-interchain-accounts) [\#6436](https://github.com/cosmos/ibc-go/pull/6436) Refactor ICA host keeper instantiation method to avoid panic related to proto files.

## [v8.3.1](https://github.com/cosmos/ibc-go/releases/tag/v8.3.1) - 2024-05-22

### Improvements
* (core/ante) [\#6302](https://github.com/cosmos/ibc-go/pull/6302) Performance: Skip app callbacks during RecvPacket execution in checkTx within the redundant relay ante handler.
* (core/ante) [\#6280](https://github.com/cosmos/ibc-go/pull/6280) Performance: Skip redundant proof checking in RecvPacket execution in reCheckTx within the redundant relay ante handler.
* (core/ante) [\#6306](https://github.com/cosmos/ibc-go/pull/6306) Performance: Skip misbehaviour checks in UpdateClient flow and skip signature checks in reCheckTx mode.

## [v8.3.0](https://github.com/cosmos/ibc-go/releases/tag/v8.3.0) - 2024-05-16

### Dependencies

* [\#6300](https://github.com/cosmos/ibc-go/pull/6300) Bump Cosmos SDK to v0.50.6 and CometBFT to v0.38.7.

### State Machine Breaking

* (light-clients/07-tendermint) [\#6276](https://github.com/cosmos/ibc-go/pull/6276) Fix: No-op to avoid panicking on `UpdateState` for invalid misbehaviour submissions.

### Improvements

* (apps/27-interchain-accounts, apps/tranfer, apps/29-fee) [\#6253](https://github.com/cosmos/ibc-go/pull/6253) Allow channel handshake to succeed if fee middleware is wired up on one side, but not the other.
* (apps/27-interchain-accounts) [\#6251](https://github.com/cosmos/ibc-go/pull/6251) Use `UNORDERED` as the default ordering for new ICA channels.
* (apps/transfer) [\#6268](https://github.com/cosmos/ibc-go/pull/6268) Use memo strings instead of JSON keys in `AllowedPacketData` of transfer authorization.
* (core/ante) [\#6278](https://github.com/cosmos/ibc-go/pull/6278) Performance: Exclude pruning from tendermint client updates in ante handler executions.

### Features

* (core) [\#6055](https://github.com/cosmos/ibc-go/pull/6055) Introduce a new interface `ConsensusHost` used to validate an IBC `ClientState` and `ConsensusState` against the host chain's underlying consensus parameters.
* (core/02-client) [\#5821](https://github.com/cosmos/ibc-go/pull/5821) Add rpc `VerifyMembershipProof` (querier approach for conditional clients).
* (core/04-channel) [\#5788](https://github.com/cosmos/ibc-go/pull/5788) Add `NewErrorAcknowledgementWithCodespace` to allow codespaces in ack errors.
* (apps/27-interchain-accounts) [\#5785](https://github.com/cosmos/ibc-go/pull/5785) Introduce a new tx message that ICA host submodule can use to query the chain (only those marked with `module_query_safe`) and write the responses to the acknowledgement.

### Bug Fixes

* (apps/27-interchain-accounts) [\#6167](https://github.com/cosmos/ibc-go/pull/6167) Fixed an edge case bug where migrating params for a pre-existing ica module which implemented controller functionality only could panic when migrating params for newly added host, and align controller param migration with host.
* (app/29-fee) [\#6255](https://github.com/cosmos/ibc-go/pull/6255) Delete refunded fees from state if some fee(s) cannot be refunded on channel closure.

## [v8.2.0](https://github.com/cosmos/ibc-go/releases/tag/v8.2.0) - 2024-04-05

### Dependencies

* [\#5975](https://github.com/cosmos/ibc-go/pull/5975) Bump Cosmos SDK to v0.50.5.

### Improvements

* (proto) [\#5987](https://github.com/cosmos/ibc-go/pull/5987) Add wasm proto files.

## [v8.1.0](https://github.com/cosmos/ibc-go/releases/tag/v8.1.0) - 2024-01-31

### Dependencies

* [\#5663](https://github.com/cosmos/ibc-go/pull/5663) Bump Cosmos SDK to v0.50.3 and CometBFT to v0.38.2.

### State Machine Breaking

* (apps/27-interchain-accounts) [\#5442](https://github.com/cosmos/ibc-go/pull/5442) Increase the maximum allowed length for the memo field of `InterchainAccountPacketData`.

### Improvements

* (core/02-client) [\#5429](https://github.com/cosmos/ibc-go/pull/5429) Add wildcard `"*"` to allow all clients in `AllowedClients` param.
* (core) [\#5541](https://github.com/cosmos/ibc-go/pull/5541) Enable emission of events on erroneous IBC application callbacks by appending a prefix to all event type and attribute keys.

### Features

* (core/04-channel) [\#1613](https://github.com/cosmos/ibc-go/pull/1613) Channel upgradability.
* (apps/transfer) [\#5280](https://github.com/cosmos/ibc-go/pull/5280) Add list of allowed packet data keys to `Allocation` of `TransferAuthorization`.
* (apps/27-interchain-accounts) [\#5633](https://github.com/cosmos/ibc-go/pull/5633) Allow setting new and upgrading existing ICA (ordered) channels to use unordered ordering.

### Bug Fixes

* (apps/27-interchain-accounts) [\#5343](https://github.com/cosmos/ibc-go/pull/5343) Add check if controller is enabled in `sendTx` before sending packet to host.
* (apps/29-fee) [\#5441](https://github.com/cosmos/ibc-go/pull/5441) Allow setting the relayer address as payee.

## [v8.0.0](https://github.com/cosmos/ibc-go/releases/tag/v8.0.0) - 2023-11-10

### Dependencies

* [\#5038](https://github.com/cosmos/ibc-go/pull/5038) Bump SDK v0.50.1 and cometBFT v0.38.
* [\#4398](https://github.com/cosmos/ibc-go/pull/4398) Update all modules to go 1.21.

### API Breaking

* (core) [\#4703](https://github.com/cosmos/ibc-go/pull/4703) Make `PortKeeper` field of `IBCKeeper` a pointer.
* (core/23-commitment) [\#4459](https://github.com/cosmos/ibc-go/pull/4459) Remove `Pretty` and `String` custom implementations of `MerklePath`.
* [\#3205](https://github.com/cosmos/ibc-go/pull/3205) Make event emission functions unexported.
* (apps/27-interchain-accounts, apps/transfer) [\#3253](https://github.com/cosmos/ibc-go/pull/3253) Rename `IsBound` to `HasCapability`.
* (apps/27-interchain-accounts, apps/transfer) [\#3303](https://github.com/cosmos/ibc-go/pull/3303) Make `HasCapability` private.
* [\#3303](https://github.com/cosmos/ibc-go/pull/3856) Add missing/remove unnecessary gogoproto directive.
* (apps/27-interchain-accounts) [\#3967](https://github.com/cosmos/ibc-go/pull/3967) Add encoding type as argument to ICA encoding/decoding functions.
* (core) [\#3867](https://github.com/cosmos/ibc-go/pull/3867) Remove unnecessary event attribute from INIT handshake msgs.
* (core/04-channel) [\#3806](https://github.com/cosmos/ibc-go/pull/3806) Remove unused `EventTypeTimeoutPacketOnClose`.
* (testing) [\#4018](https://github.com/cosmos/ibc-go/pull/4018) Allow failure expectations when using `chain.SendMsgs`.

### State Machine Breaking

* (apps/transfer, apps/27-interchain-accounts, app/29-fee) [\#4992](https://github.com/cosmos/ibc-go/pull/4992) Set validation for length of string fields.

### Improvements

* [\#3304](https://github.com/cosmos/ibc-go/pull/3304) Remove unnecessary defer func statements.
* (apps/29-fee) [\#3054](https://github.com/cosmos/ibc-go/pull/3054) Add page result to ics29-fee queries.
* (apps/27-interchain-accounts, apps/transfer) [\#3077](https://github.com/cosmos/ibc-go/pull/3077) Add debug level logging for the error message which is discarded when generating a failed acknowledgement.
* (core/03-connection) [\#3244](https://github.com/cosmos/ibc-go/pull/3244) Cleanup 03-connection msg validate basic test.
* (core/02-client) [\#3514](https://github.com/cosmos/ibc-go/pull/3514) Add check for the client status in `CreateClient`.
* (apps/29-fee) [\#4100](https://github.com/cosmos/ibc-go/pull/4100) Adding `MetadataFromVersion` to `29-fee` package `types`.
* (apps/29-fee) [\#4290](https://github.com/cosmos/ibc-go/pull/4290) Use `types.MetadataFromVersion` helper function for callback handlers.
* (core/04-channel) [\#4155](https://github.com/cosmos/ibc-go/pull/4155) Adding `IsOpen` and `IsClosed` methods to `Channel` type.
* (core/03-connection) [\#4110](https://github.com/cosmos/ibc-go/pull/4110) Remove `Version` interface and casting functions from 03-connection.
* (core) [\#4835](https://github.com/cosmos/ibc-go/pull/4835) Use expected interface for legacy params subspace parameter of keeper constructor functions.

### Features

* (capability) [\#3097](https://github.com/cosmos/ibc-go/pull/3097) Migrate capability module from Cosmos SDK to ibc-go.
* (core/02-client) [\#3640](https://github.com/cosmos/ibc-go/pull/3640) Migrate client params to be self managed.
* (core/03-connection) [\#3650](https://github.com/cosmos/ibc-go/pull/3650) Migrate connection params to be self managed.
* (apps/transfer) [\#3553](https://github.com/cosmos/ibc-go/pull/3553) Migrate transfer parameters to be self managed (#3553)
* (apps/27-interchain-accounts) [\#3520](https://github.com/cosmos/ibc-go/pull/3590) Migrate ica/controller parameters to be self managed (#3590)
* (apps/27-interchain-accounts) [\#3520](https://github.com/cosmos/ibc-go/pull/3520) Migrate ica/host to params to be self managed.
* (apps/transfer) [\#3104](https://github.com/cosmos/ibc-go/pull/3104) Add metadata for IBC tokens.
* [\#4620](https://github.com/cosmos/ibc-go/pull/4620) Migrate to gov v1 via the additions of `MsgRecoverClient` and `MsgIBCSoftwareUpgrade`. The legacy proposal types `ClientUpdateProposal` and `UpgradeProposal` have been deprecated and will be removed in the next major release.

### Bug Fixes

* (apps/transfer) [\#4709](https://github.com/cosmos/ibc-go/pull/4709) Order query service RPCs to fix availability of denom traces endpoint when no args are provided.
* (core/04-channel) [\#3357](https://github.com/cosmos/ibc-go/pull/3357) Handle unordered channels in `NextSequenceReceive` query.
* (e2e) [\#3402](https://github.com/cosmos/ibc-go/pull/3402 Allow retries for messages signed by relayer.
* (core/04-channel) [\#3417](https://github.com/cosmos/ibc-go/pull/3417) Add missing query for next sequence send.
* (testing) [\#4630](https://github.com/cosmos/ibc-go/pull/4630) Update `testconfig` to use revision formatted chain IDs.
* (core/04-channel) [\#4706](https://github.com/cosmos/ibc-go/pull/4706) Retrieve correct next send sequence for packets in unordered channels.
* (core/02-client) [\#4746](https://github.com/cosmos/ibc-go/pull/4746) Register implementations against `govtypes.Content` interface.
* (apps/27-interchain-accounts) [\#4944](https://github.com/cosmos/ibc-go/pull/4944) Add missing proto interface registration.
* (core/02-client) [\#5020](https://github.com/cosmos/ibc-go/pull/5020) Fix expect pointer error when unmarshalling misbehaviour file.

### Documentation

* [\#3133](https://github.com/cosmos/ibc-go/pull/3133) Add linter for markdown documents.
* [\#4693](https://github.com/cosmos/ibc-go/pull/4693) Migrate docs to docusaurus.

### Testing

* [\#3138](https://github.com/cosmos/ibc-go/pull/3138) Use `testing.TB` instead of `testing.T` to support benchmarks and fuzz tests. 
* [\#3980](https://github.com/cosmos/ibc-go/pull/3980) Change `sdk.Events` usage to `[]abci.Event` in the testing package.
* [\#3986](https://github.com/cosmos/ibc-go/pull/3986) Add function `RelayPacketWithResults`.
* [\#4182](https://github.com/cosmos/ibc-go/pull/4182) Return current validator set when requesting current height in `GetValsAtHeight`.
* [\#4319](https://github.com/cosmos/ibc-go/pull/4319) Fix in `TimeoutPacket` function to use counterparty `portID`/`channelID` in `GetNextSequenceRecv` query.
* [\#4180](https://github.com/cosmos/ibc-go/pull/4180) Remove unused function `simapp.SetupWithGenesisAccounts`.

### Miscellaneous Tasks

* (apps/27-interchain-accounts) [\#4677](https://github.com/cosmos/ibc-go/pull/4677) Remove ica store key.
* [\#4724](https://github.com/cosmos/ibc-go/pull/4724) Add `HasValidateBasic` compiler assertions to messages.
* [\#4725](https://github.com/cosmos/ibc-go/pull/4725) Add fzf selection for config files.
* [\#4741](https://github.com/cosmos/ibc-go/pull/4741) Panic with error.
* [\#3186](https://github.com/cosmos/ibc-go/pull/3186) Migrate all SDK errors to the new errors go module.
* [\#3216](https://github.com/cosmos/ibc-go/pull/3216) Modify `simapp` to fulfill the SDK `runtime.AppI` interface.
* [\#3290](https://github.com/cosmos/ibc-go/pull/3290) Remove `gogoproto` yaml tags from proto files.
* [\#3439](https://github.com/cosmos/ibc-go/pull/3439) Use nil pointer pattern to check for interface compliance.
* [\#3433](https://github.com/cosmos/ibc-go/pull/3433) Add tests for `acknowledgement.Acknowledgement()`.
* (core, apps/29-fee) [\#3462](https://github.com/cosmos/ibc-go/pull/3462) Add missing `nil` check and corresponding tests for query handlers.
* (light-clients/07-tendermint, light-clients/06-solomachine) [\#3571](https://github.com/cosmos/ibc-go/pull/3571) Delete unused `GetProofSpecs` functions.
* (core) [\#3616](https://github.com/cosmos/ibc-go/pull/3616) Add debug log for redundant relay.
* (core) [\#3892](https://github.com/cosmos/ibc-go/pull/3892) Add deprecated option to `create_localhost` field.
* (core) [\#3893](https://github.com/cosmos/ibc-go/pull/3893) Add deprecated option to `MsgSubmitMisbehaviour`.
* (apps/transfer, apps/29-fee) [\#4570](https://github.com/cosmos/ibc-go/pull/4570) Remove `GetSignBytes` from 29-fee and transfer msgs.
* [\#3630](https://github.com/cosmos/ibc-go/pull/3630) Add annotation to Msg service.

## [v7.3.0](https://github.com/cosmos/ibc-go/releases/tag/v7.3.0) - 2023-08-31

### Dependencies

* [\#4122](https://github.com/cosmos/ibc-go/pull/4122) Update Cosmos SDK to v0.47.4.

### Improvements

* [\#4187](https://github.com/cosmos/ibc-go/pull/4187) Adds function `WithICS4Wrapper` to keepers to allow to set the middleware after the keeper's creation.
* (light-clients/06-solomachine) [\#4429](https://github.com/cosmos/ibc-go/pull/4429) Remove IBC key from path of bytes signed by solomachine and not escape the path.

### Features

* (apps/27-interchain-accounts) [\#3796](https://github.com/cosmos/ibc-go/pull/3796) Adds support for json tx encoding for interchain accounts.
* [\#4188](https://github.com/cosmos/ibc-go/pull/4188) Adds optional `PacketDataUnmarshaler` interface that allows a middleware to request the packet data to be unmarshaled by the base application.
* [\#4199](https://github.com/cosmos/ibc-go/pull/4199) Adds optional `PacketDataProvider` interface for retrieving custom packet data stored on behalf of another application.
* [\#4200](https://github.com/cosmos/ibc-go/pull/4200) Adds optional `PacketData` interface which application's packet data may implement.

### Bug Fixes

* (04-channel) [\#4476](https://github.com/cosmos/ibc-go/pull/4476) Use UTC time in log messages for packet timeout error.
* (testing) [\#4483](https://github.com/cosmos/ibc-go/pull/4483) Use the correct revision height when querying trusted validator set.

## [v7.2.1](https://github.com/cosmos/ibc-go/releases/tag/v7.2.1) - 2023-08-31

### Bug Fixes

* (04-channel) [\#4476](https://github.com/cosmos/ibc-go/pull/4476) Use UTC time in log messages for packet timeout error.
* (testing) [\#4483](https://github.com/cosmos/ibc-go/pull/4483) Use the correct revision height when querying trusted validator set.

## [v7.2.0](https://github.com/cosmos/ibc-go/releases/tag/v7.2.0) - 2023-06-22

### Dependencies

* [\#3810](https://github.com/cosmos/ibc-go/pull/3810) Update Cosmos SDK to v0.47.3.
* [\#3862](https://github.com/cosmos/ibc-go/pull/3862) Update CometBFT to v0.37.2.

### State Machine Breaking

* [\#3907](https://github.com/cosmos/ibc-go/pull/3907) Re-implemented missing functions of `LegacyMsg` interface to fix transaction signing with ledger.

## [v7.1.0](https://github.com/cosmos/ibc-go/releases/tag/v7.1.0) - 2023-06-09

### Dependencies

* [\#3542](https://github.com/cosmos/ibc-go/pull/3542) Update Cosmos SDK to v0.47.2 and CometBFT to v0.37.1.
* [\#3457](https://github.com/cosmos/ibc-go/pull/3457) Update to ics23 v0.10.0.

### Improvements

* (apps/transfer) [\#3454](https://github.com/cosmos/ibc-go/pull/3454) Support transfer authorization unlimited spending when the max `uint256` value is provided as limit.

### Features

* (light-clients/09-localhost) [\#3229](https://github.com/cosmos/ibc-go/pull/3229) Implementation of v2 of localhost loopback client.
* (apps/transfer) [\#3019](https://github.com/cosmos/ibc-go/pull/3019) Add state entry to keep track of total amount of tokens in escrow.

### Bug Fixes

* (core/04-channel) [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.
* (core/04-channel) [\#3593](https://github.com/cosmos/ibc-go/pull/3593) `SendPacket` now correctly returns `ErrClientNotFound` in favour of `ErrConsensusStateNotFound`.

## [v7.0.1](https://github.com/cosmos/ibc-go/releases/tag/v7.0.1) - 2023-05-25

### Bug Fixes

* [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.

## [v7.0.0](https://github.com/cosmos/ibc-go/releases/tag/v7.0.0) - 2023-03-17

### Dependencies

* [\#2672](https://github.com/cosmos/ibc-go/issues/2672) Update to cosmos-sdk v0.47.
* [\#3175](https://github.com/cosmos/ibc-go/issues/3175) Migrate to cometbft v0.37.

### API Breaking

* (core) [\#2897](https://github.com/cosmos/ibc-go/pull/2897) Remove legacy migrations required for upgrading from Stargate release line to ibc-go >= v1.x.x.
* (core/02-client) [\#2856](https://github.com/cosmos/ibc-go/pull/2856) Rename `IterateClients` to `IterateClientStates`. The function now takes a prefix argument which may be used for prefix iteration over the client store.
* (light-clients/tendermint)[\#1768](https://github.com/cosmos/ibc-go/pull/1768) Removed `AllowUpdateAfterExpiry`, `AllowUpdateAfterMisbehaviour` booleans as they are deprecated (see ADR026)
* (06-solomachine) [\#1679](https://github.com/cosmos/ibc-go/pull/1679) Remove `types` sub-package from `06-solomachine` lightclient directory.
* (07-tendermint) [\#1677](https://github.com/cosmos/ibc-go/pull/1677) Remove `types` sub-package from `07-tendermint` lightclient directory.
* (06-solomachine) [\#1687](https://github.com/cosmos/ibc-go/pull/1687) Bump `06-solomachine` protobuf version from `v2` to `v3`.
* (06-solomachine) [\#1687](https://github.com/cosmos/ibc-go/pull/1687) Removed `DataType` enum and associated message types from `06-solomachine`. `DataType` has been removed from `SignBytes` and `SignatureAndData` in favour of `path`.
* (02-client) [\#598](https://github.com/cosmos/ibc-go/pull/598) The client state and consensus state return value has been removed from `VerifyUpgradeAndUpdateState`. Light client implementations must update the client state and consensus state after verifying a valid client upgrade.
* (06-solomachine) [\#1100](https://github.com/cosmos/ibc-go/pull/1100) Remove `GetClientID` function from 06-solomachine `Misbehaviour` type.
* (06-solomachine) [\#1100](https://github.com/cosmos/ibc-go/pull/1100) Deprecate `ClientId` field in 06-solomachine `Misbehaviour` type.
* (07-tendermint) [\#1097](https://github.com/cosmos/ibc-go/pull/1097) Remove `GetClientID` function from 07-tendermint `Misbehaviour` type.
* (07-tendermint) [\#1097](https://github.com/cosmos/ibc-go/pull/1097) Deprecate `ClientId` field in 07-tendermint `Misbehaviour` type.
* (modules/core/exported) [\#1107](https://github.com/cosmos/ibc-go/pull/1107) Merging the `Header` and `Misbehaviour` interfaces into a single `ClientMessage` type.
* (06-solomachine)[\#1906](https://github.com/cosmos/ibc-go/pull/1906/files) Removed `AllowUpdateAfterProposal` boolean as it has been deprecated (see 01_concepts of the solo machine spec for more details).
* (07-tendermint) [\#1896](https://github.com/cosmos/ibc-go/pull/1896) Remove error return from `IterateConsensusStateAscending` in `07-tendermint`.
* (apps/27-interchain-accounts) [\#2638](https://github.com/cosmos/ibc-go/pull/2638) Interchain accounts host and controller Keepers now expects a keeper which fulfills the expected `exported.ScopedKeeper` interface for the capability keeper.
* (06-solomachine) [\#2761](https://github.com/cosmos/ibc-go/pull/2761) Removed deprecated `ClientId` field from `Misbehaviour` and `allow_update_after_proposal` field from `ClientState`.
* (apps) [\#3154](https://github.com/cosmos/ibc-go/pull/3154)  Remove unused `ProposalContents` function.
* (apps) [\#3149](https://github.com/cosmos/ibc-go/pull/3149) Remove legacy interface function `RandomizedParams`, which is no longer used.
* (light-clients/06-solomachine) [\#2941](https://github.com/cosmos/ibc-go/pull/2941) Remove solomachine header sequence.
* (core) [\#2982](https://github.com/cosmos/ibc-go/pull/2982) Moved the ibc module name into the exported package.

### State Machine Breaking

* (06-solomachine) [\#2744](https://github.com/cosmos/ibc-go/pull/2744) `Misbehaviour.ValidateBasic()` now only enforces that signature data does not match when the signature paths are different.
* (06-solomachine) [\#2748](https://github.com/cosmos/ibc-go/pull/2748) Adding sentinel value for header path in 06-solomachine.
* (apps/29-fee) [\#2942](https://github.com/cosmos/ibc-go/pull/2942) Check `x/bank` send enabled before escrowing fees.
* (core/04-channel) [\#3009](https://github.com/cosmos/ibc-go/pull/3009) Change check to disallow optimistic sends.

### Improvements

* (core) [\#3082](https://github.com/cosmos/ibc-go/pull/3082) Add `HasConnection` and `HasChannel` methods.
* (tests) [\#2926](https://github.com/cosmos/ibc-go/pull/2926) Lint tests
* (apps/transfer) [\#2643](https://github.com/cosmos/ibc-go/pull/2643) Add amount, denom, and memo to transfer event emission.
* (core) [\#2746](https://github.com/cosmos/ibc-go/pull/2746) Allow proof height to be zero for all core IBC `sdk.Msg` types that contain proofs.
* (light-clients/06-solomachine) [\#2746](https://github.com/cosmos/ibc-go/pull/2746) Discard proofHeight for solo machines and use the solo machine sequence instead.
* (modules/light-clients/07-tendermint) [\#1713](https://github.com/cosmos/ibc-go/pull/1713) Allow client upgrade proposals to update `TrustingPeriod`. See ADR-026 for context.
* (modules/core/02-client) [\#1188](https://github.com/cosmos/ibc-go/pull/1188/files) Routing `MsgSubmitMisbehaviour` to `UpdateClient` keeper function. Deprecating `SubmitMisbehaviour` endpoint.
* (modules/core/02-client) [\#1208](https://github.com/cosmos/ibc-go/pull/1208) Replace `CheckHeaderAndUpdateState` usage in 02-client with calls to `VerifyClientMessage`, `CheckForMisbehaviour`, `UpdateStateOnMisbehaviour` and `UpdateState`.
* (modules/light-clients/09-localhost) [\#1187](https://github.com/cosmos/ibc-go/pull/1187/) Removing localhost light client implementation as it is not functional. An upgrade handler is provided in `modules/migrations/v5` to prune `09-localhost` clients and consensus states from the store.
* (modules/core/02-client) [\#1186](https://github.com/cosmos/ibc-go/pull/1186) Removing `GetRoot` function from ConsensusState interface in `02-client`. `GetRoot` is unused by core IBC.
* (modules/core/02-client) [\#1196](https://github.com/cosmos/ibc-go/pull/1196) Adding VerifyClientMessage to ClientState interface.
* (modules/core/02-client) [\#1198](https://github.com/cosmos/ibc-go/pull/1198) Adding UpdateStateOnMisbehaviour to ClientState interface.
* (modules/core/02-client) [\#1170](https://github.com/cosmos/ibc-go/pull/1170) Updating `ClientUpdateProposal` to set client state in lightclient implementations `CheckSubstituteAndUpdateState` methods.
* (modules/core/02-client) [\#1197](https://github.com/cosmos/ibc-go/pull/1197) Adding `CheckForMisbehaviour` to `ClientState` interface.
* (modules/core/02-client) [\#1210](https://github.com/cosmos/ibc-go/pull/1210) Removing `CheckHeaderAndUpdateState` from `ClientState` interface & associated light client implementations.
* (modules/core/02-client) [\#1212](https://github.com/cosmos/ibc-go/pull/1212) Removing `CheckMisbehaviourAndUpdateState` from `ClientState` interface & associated light client implementations.
* (modules/core/exported) [\#1206](https://github.com/cosmos/ibc-go/pull/1206) Adding new method `UpdateState` to `ClientState` interface.
* (modules/core/02-client) [\#1741](https://github.com/cosmos/ibc-go/pull/1741) Emitting a new `upgrade_chain` event upon setting upgrade consensus state.
* (client) [\#724](https://github.com/cosmos/ibc-go/pull/724) `IsRevisionFormat` and `IsClientIDFormat` have been updated to disallow newlines before the dash used to separate the chainID and revision number, and the client type and client sequence.
* (02-client/cli) [\#897](https://github.com/cosmos/ibc-go/pull/897) Remove `GetClientID()` from `Misbehaviour` interface. Submit client misbehaviour cli command requires an explicit client id now.
* (06-solomachine) [\#1972](https://github.com/cosmos/ibc-go/pull/1972) Solo machine implementation of `ZeroCustomFields` fn now panics as the fn is only used for upgrades which solo machine does not support.
* (light-clients/06-solomachine) Moving `verifyMisbehaviour` function from update.go to misbehaviour_handle.go.
* [\#2434](https://github.com/cosmos/ibc-go/pull/2478) Removed all `TypeMsg` constants
* (modules/core/exported) [\#2539](https://github.com/cosmos/ibc-go/pull/2539) Removing `GetVersions` from `ConnectionI` interface.
* (core/02-connection) [\#2419](https://github.com/cosmos/ibc-go/pull/2419) Add optional proof data to proto definitions of `MsgConnectionOpenTry` and `MsgConnectionOpenAck` for host state machines that are unable to introspect their own consensus state.
* (light-clients/07-tendermint) [\#3046](https://github.com/cosmos/ibc-go/pull/3046) Moved non-verification misbehaviour checks to `CheckForMisbehaviour`.
* (apps/29-fee) [\#2975](https://github.com/cosmos/ibc-go/pull/2975) Adding distribute fee events to ics29.
* (light-clients/07-tendermint) [\#2965](https://github.com/cosmos/ibc-go/pull/2965) Prune expired `07-tendermint` consensus states on duplicate header updates.
* (light-clients) [\#2736](https://github.com/cosmos/ibc-go/pull/2736) Updating `VerifyMembership` and `VerifyNonMembership` methods to use `Path` interface.
* (light-clients) [\#3113](https://github.com/cosmos/ibc-go/pull/3113) Align light client module names.

### Features

* (apps/transfer) [\#3079](https://github.com/cosmos/ibc-go/pull/3079) Added authz support for ics20.
* (core/02-client) [\#2824](https://github.com/cosmos/ibc-go/pull/2824) Add genesis migrations for v6 to v7. The migration migrates the solo machine client state definition, removes all solo machine consensus states and removes the localhost client.
* (core/24-host) [\#2856](https://github.com/cosmos/ibc-go/pull/2856) Add `PrefixedClientStorePath` and `PrefixedClientStoreKey` functions to 24-host
* (core/02-client) [\#2819](https://github.com/cosmos/ibc-go/pull/2819) Add automatic in-place store migrations to remove the localhost client and migrate existing solo machine definitions.
* (light-clients/06-solomachine) [\#2826](https://github.com/cosmos/ibc-go/pull/2826) Add `AppModuleBasic` for the 06-solomachine client and remove solo machine type registration from core IBC. Chains must register the `AppModuleBasic` of light clients.
* (light-clients/07-tendermint) [\#2825](https://github.com/cosmos/ibc-go/pull/2825) Add `AppModuleBasic` for the 07-tendermint client and remove tendermint type registration from core IBC. Chains must register the `AppModuleBasic` of light clients.
* (light-clients/07-tendermint) [\#2800](https://github.com/cosmos/ibc-go/pull/2800) Add optional in-place store migration function to prune all expired tendermint consensus states.
* (core/24-host) [\#2820](https://github.com/cosmos/ibc-go/pull/2820) Add `MustParseClientStatePath` which parses the clientID from a client state key path.
* (testing/simapp) [\#2842](https://github.com/cosmos/ibc-go/pull/2842) Adding the new upgrade handler for v6 -> v7 to simapp which prunes expired Tendermint consensus states.
* (testing) [\#2829](https://github.com/cosmos/ibc-go/pull/2829) Add `AssertEvents` which asserts events against expected event map.

### Bug Fixes

* (testing) [\#3295](https://github.com/cosmos/ibc-go/pull/3295) The function `SetupWithGenesisValSet` will set the baseapp chainID before running `InitChain`
* (light-clients/solomachine) [\#1839](https://github.com/cosmos/ibc-go/pull/1839) Fixed usage of the new diversifier in validation of changing diversifiers for the solo machine. The current diversifier must sign over the new diversifier.
* (light-clients/07-tendermint) [\#1674](https://github.com/cosmos/ibc-go/pull/1674) Submitted ClientState is zeroed out before checking the proof in order to prevent the proposal from containing information governance is not actually voting on.
* (modules/core/02-client)[\#1676](https://github.com/cosmos/ibc-go/pull/1676) ClientState must be zeroed out for `UpgradeProposals` to pass validation. This prevents a proposal containing information governance is not actually voting on.
* (core/02-client) [\#2510](https://github.com/cosmos/ibc-go/pull/2510) Fix client ID validation regex to conform closer to spec.
* (apps/transfer) [\#3045](https://github.com/cosmos/ibc-go/pull/3045) Allow value with slashes in URL template.
* (apps/27-interchain-accounts) [\#2601](https://github.com/cosmos/ibc-go/pull/2601) Remove bech32 check from owner address on ICA controller msgs RegisterInterchainAccount and SendTx.
* (apps/transfer) [\#2651](https://github.com/cosmos/ibc-go/pull/2651) Skip emission of unpopulated memo field in ics20.
* (apps/27-interchain-accounts) [\#2682](https://github.com/cosmos/ibc-go/pull/2682) Avoid race conditions in ics27 handshakes.
* (light-clients/06-solomachine) [\#2741](https://github.com/cosmos/ibc-go/pull/2741) Added check for empty path in 06-solomachine.
* (light-clients/07-tendermint) [\#3022](https://github.com/cosmos/ibc-go/pull/3022) Correctly close iterator in `07-tendermint` store.
* (core/02-client) [\#3010](https://github.com/cosmos/ibc-go/pull/3010) Update `Paginate` to use `FilterPaginate` in `ClientStates` and `ConnectionChannels` grpc queries.

## [v6.2.0](https://github.com/cosmos/ibc-go/releases/tag/v6.2.0) - 2023-05-31

### Dependencies

* [\#3393](https://github.com/cosmos/ibc-go/pull/3393) Bump Cosmos SDK to v0.46.12 and replace Tendermint with CometBFT v0.34.37.

### Improvements

* (core) [\#3082](https://github.com/cosmos/ibc-go/pull/3082) Add `HasConnection` and `HasChannel` methods.
* (apps/transfer) [\#3454](https://github.com/cosmos/ibc-go/pull/3454) Support transfer authorization unlimited spending when the max `uint256` value is provided as limit.

### Features

* [\#3079](https://github.com/cosmos/ibc-go/pull/3079) Add authz support for ics20.

### Bug Fixes

* [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.

## [v6.1.1](https://github.com/cosmos/ibc-go/releases/tag/v6.1.1) - 2023-05-25

### Bug Fixes

* [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.

## [v6.1.0](https://github.com/cosmos/ibc-go/releases/tag/v6.1.0) - 2022-12-20

### Dependencies

* [\#2945](https://github.com/cosmos/ibc-go/pull/2945) Bump Cosmos SDK to v0.46.7 and Tendermint to v0.34.24.

### State Machine Breaking

* (apps/29-fee) [\#2942](https://github.com/cosmos/ibc-go/pull/2942) Check `x/bank` send enabled before escrowing fees.

## [v6.0.0](https://github.com/cosmos/ibc-go/releases/tag/v6.0.0) - 2022-12-09

### Dependencies

* [\#2868](https://github.com/cosmos/ibc-go/pull/2868) Bump ICS 23 to v0.9.0.
* [\#2458](https://github.com/cosmos/ibc-go/pull/2458) Bump Cosmos SDK to v0.46.2
* [\#2784](https://github.com/cosmos/ibc-go/pull/2784) Bump Cosmos SDK to v0.46.6 and Tendermint to v0.34.23.

### API Breaking

* (apps/27-interchain-accounts) [\#2607](https://github.com/cosmos/ibc-go/pull/2607) `SerializeCosmosTx` now takes in a `[]proto.Message` instead of `[]sdk.Msg`.
* (apps/transfer) [\#2446](https://github.com/cosmos/ibc-go/pull/2446) Remove `SendTransfer` function in favor of a private `sendTransfer` function. All IBC transfers must be initiated with `MsgTransfer`.
* (apps/29-fee) [\#2395](https://github.com/cosmos/ibc-go/pull/2395) Remove param space from ics29 NewKeeper function. The field was unused.
* (apps/27-interchain-accounts) [\#2133](https://github.com/cosmos/ibc-go/pull/2133) Generates genesis protos in a separate directory to avoid circular import errors. The protobuf package name has changed for the genesis types.
* (apps/27-interchain-accounts) [\#2638](https://github.com/cosmos/ibc-go/pull/2638) Interchain accounts host and controller Keepers now expects a keeper which fulfills the expected `exported.ScopedKeeper` interface for the capability keeper.
* (transfer) [\#2638](https://github.com/cosmos/ibc-go/pull/2638) Transfer Keeper now expects a keeper which fulfills the expected `exported.ScopedKeeper` interface for the capability keeper.
* (05-port) [\#2638](https://github.com/cosmos/ibc-go/pull/2638) Port Keeper now expects a keeper which fulfills the expected `exported.ScopedKeeper` interface for the capability keeper.
* (04-channel) [\#2638](https://github.com/cosmos/ibc-go/pull/2638) Channel Keeper now expects a keeper which fulfills the expected `exported.ScopedKeeper` interface for the capability keeper.
* (core/04-channel)[\#1703](https://github.com/cosmos/ibc-go/pull/1703) Update `SendPacket` API to take in necessary arguments and construct rest of packet rather than taking in entire packet. The generated packet sequence is returned by the `SendPacket` function.
* (modules/apps/27-interchain-accounts) [\#2433](https://github.com/cosmos/ibc-go/pull/2450) Renamed icatypes.PortPrefix to icatypes.ControllerPortPrefix & icatypes.PortID to icatypes.HostPortID
* (testing) [\#2567](https://github.com/cosmos/ibc-go/pull/2567) Modify `SendPacket` API of `Endpoint` to match the API of `SendPacket` in 04-channel.

### State Machine Breaking

* (apps/transfer) [\#2651](https://github.com/cosmos/ibc-go/pull/2651) Introduce `mustProtoMarshalJSON` for ics20 packet data marshalling which will skip emission (marshalling) of the memo field if unpopulated (empty).
* (27-interchain-accounts) [\#2590](https://github.com/cosmos/ibc-go/pull/2590) Removing port prefix requirement from the ICA host channel handshake
* (transfer) [\#2377](https://github.com/cosmos/ibc-go/pull/2377) Adding `sequence` to `MsgTransferResponse`.
* (light-clients/07-tendermint) [\#2555](https://github.com/cosmos/ibc-go/pull/2555) Forbid negative values for `TrustingPeriod`, `UnbondingPeriod` and `MaxClockDrift` (as specified in ICS-07).
* (core/04-channel) [\#2973](https://github.com/cosmos/ibc-go/pull/2973) Write channel state before invoking app callbacks in ack and confirm channel handshake steps.

### Improvements

* (apps/27-interchain-accounts) [\#2134](https://github.com/cosmos/ibc-go/pull/2134) Adding upgrade handler to ICS27 `controller` submodule for migration of channel capabilities. This upgrade handler migrates ownership of channel capabilities from the underlying application to the ICS27 `controller` submodule.
* (apps/27-interchain-accounts) [\#2102](https://github.com/cosmos/ibc-go/pull/2102) ICS27 controller middleware now supports a nil underlying application. This allows chains to make use of interchain accounts with existing auth mechanisms such as x/group and x/gov.
* (apps/27-interchain-accounts) [\#2157](https://github.com/cosmos/ibc-go/pull/2157) Adding `IsMiddlewareEnabled` functionality to enforce calls to ICS27 msg server to *not* route to the underlying application.
* (apps/27-interchain-accounts) [\#2146](https://github.com/cosmos/ibc-go/pull/2146) ICS27 controller now claims the channel capability passed via ibc core, and passes `nil` to the underlying app callback. The channel capability arg in `SendTx` is now ignored and looked up internally.
* (apps/27-interchain-accounts) [\#2177](https://github.com/cosmos/ibc-go/pull/2177) Adding `IsMiddlewareEnabled` flag to interchain accounts `ActiveChannel` genesis type.
* (apps/27-interchain-accounts) [\#2140](https://github.com/cosmos/ibc-go/pull/2140) Adding migration handler to ICS27 `controller` submodule to assert ownership of channel capabilities and set middleware enabled flag for existing channels. The ICS27 module consensus version has been bumped from 1 to 2.
* (core/04-channel) [\#2304](https://github.com/cosmos/ibc-go/pull/2304) Adding `GetAllChannelsWithPortPrefix` function which filters channels based on a provided port prefix.
* (apps/27-interchain-accounts) [\#2248](https://github.com/cosmos/ibc-go/pull/2248) Adding call to underlying app in `OnChanCloseConfirm` callback of the controller submodule and adding relevant unit tests.
* (apps/27-interchain-accounts) [\#2251](https://github.com/cosmos/ibc-go/pull/2251) Adding `msgServer` struct to controller submodule that embeds the `Keeper` struct.
* (apps/27-interchain-accounts) [\#2290](https://github.com/cosmos/ibc-go/pull/2290) Changed `DefaultParams` function in `host` submodule to allow all messages by default. Defined a constant named `AllowAllHostMsgs` for `host` module to keep wildcard "*" string which allows all messages.
* (apps/27-interchain-accounts) [\#2297](https://github.com/cosmos/ibc-go/pull/2297) Adding cli command to generate ICS27 packet data.
* (modules/core/keeper) [\#1728](https://github.com/cosmos/ibc-go/pull/2399) Updated channel callback errors to include portID & channelID for better identification of errors.
* (testing) [\#2657](https://github.com/cosmos/ibc-go/pull/2657) Carry `ProposerAddress` through commited blocks. Allow `DefaultGenTxGas` to be modified.
* (core/03-connection) [\#2745](https://github.com/cosmos/ibc-go/pull/2745) Adding `ConnectionParams` grpc query and CLI to 03-connection.
* (apps/29-fee) [\#2786](https://github.com/cosmos/ibc-go/pull/2786) Save gas by checking key existence with `KVStore`'s `Has` method.

### Features

* (apps/27-interchain-accounts) [\#2147](https://github.com/cosmos/ibc-go/pull/2147) Adding a `SubmitTx` gRPC endpoint for the ICS27 Controller module which allows owners of interchain accounts to submit transactions. This replaces the previously existing need for authentication modules to implement this standard functionality.
* (testing/simapp) [\#2190](https://github.com/cosmos/ibc-go/pull/2190) Adding the new `x/group` cosmos-sdk module to simapp.
* (apps/transfer) [\#2595](https://github.com/cosmos/ibc-go/pull/2595) Adding optional memo field to `FungibleTokenPacketData` and `MsgTransfer`.

### Bug Fixes

* (modules/core/keeper) [\#2403](https://github.com/cosmos/ibc-go/pull/2403) Added a function in keeper to cater for blank pointers.
* (apps/transfer) [\#2679](https://github.com/cosmos/ibc-go/pull/2679) Check `x/bank` send enabled.
* (modules/core/keeper) [\#2745](https://github.com/cosmos/ibc-go/pull/2745) Fix request wiring for `UpgradedConsensusState` in core query server.

## [v5.3.1](https://github.com/cosmos/ibc-go/releases/tag/v5.3.1) - 2023-05-25

### Bug Fixes

* [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.

## [v5.3.0](https://github.com/cosmos/ibc-go/releases/tag/v5.3.0) - 2023-05-04

### Dependencies

* [\#3354](https://github.com/cosmos/ibc-go/pull/3354) Bump Cosmos SDK to v0.46.12 and replace Tendermint with CometBFT v0.34.27.

## [v5.2.1](https://github.com/cosmos/ibc-go/releases/tag/v5.2.1) - 2023-05-25

### Bug Fixes

* [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.

## [v5.2.0](https://github.com/cosmos/ibc-go/releases/tag/v5.2.0) - 2022-12-20

### Dependencies

* [\#2868](https://github.com/cosmos/ibc-go/pull/2868) Bump ICS 23 to v0.9.0.
* [\#2944](https://github.com/cosmos/ibc-go/pull/2944) Bump Cosmos SDK to v0.46.7 and Tendermint to v0.34.24.

### State Machine Breaking

* (apps/29-fee) [\#2942](https://github.com/cosmos/ibc-go/pull/2942) Check `x/bank` send enabled before escrowing fees.

### Improvements

* (apps/29-fee) [\#2786](https://github.com/cosmos/ibc-go/pull/2786) Save gas by checking key existence with `KVStore`'s `Has` method.

## [v5.1.0](https://github.com/cosmos/ibc-go/releases/tag/v5.1.0) - 2022-11-09

### Dependencies

* [\#2647](https://github.com/cosmos/ibc-go/pull/2647) Bump Cosmos SDK to v0.46.4 and Tendermint to v0.34.22.

### State Machine Breaking

* (apps/transfer) [\#2651](https://github.com/cosmos/ibc-go/pull/2651) Introduce `mustProtoMarshalJSON` for ics20 packet data marshalling which will skip emission (marshalling) of the memo field if unpopulated (empty).
* (27-interchain-accounts) [\#2590](https://github.com/cosmos/ibc-go/pull/2590) Removing port prefix requirement from the ICA host channel handshake
* (transfer) [\#2377](https://github.com/cosmos/ibc-go/pull/2377) Adding `sequence` to `MsgTransferResponse`.

### Improvements

* (testing) [\#2657](https://github.com/cosmos/ibc-go/pull/2657) Carry `ProposerAddress` through committed blocks. Allow `DefaultGenTxGas` to be modified.

### Features

* (apps/transfer) [\#2595](https://github.com/cosmos/ibc-go/pull/2595) Adding optional memo field to `FungibleTokenPacketData` and `MsgTransfer`.

### Bug Fixes

* (apps/transfer) [\#2679](https://github.com/cosmos/ibc-go/pull/2679) Check `x/bank` send enabled.

## [v5.0.1](https://github.com/cosmos/ibc-go/releases/tag/v5.0.1) - 2022-10-27

### Dependencies

* [\#2623](https://github.com/cosmos/ibc-go/pull/2623) Bump SDK version to v0.46.3 and Tendermint version to v0.34.22.

## [v5.0.0](https://github.com/cosmos/ibc-go/releases/tag/v5.0.0) - 2022-09-28

### Dependencies

* [\#1653](https://github.com/cosmos/ibc-go/pull/1653) Bump SDK version to v0.46
* [\#2124](https://github.com/cosmos/ibc-go/pull/2124) Bump SDK version to v0.46.1

### API Breaking

* (testing)[\#2028](https://github.com/cosmos/ibc-go/pull/2028) New interface `ibctestingtypes.StakingKeeper` added and set for the testing app `StakingKeeper` setup.
* (core/04-channel) [\#1418](https://github.com/cosmos/ibc-go/pull/1418) `NewPacketId` has been renamed to `NewPacketID` to comply with go linting rules.
* (core/ante) [\#1418](https://github.com/cosmos/ibc-go/pull/1418) `AnteDecorator` has been renamed to `RedundancyDecorator` to comply with go linting rules and to give more clarity to the purpose of the Decorator.
* (core/ante) [\#1820](https://github.com/cosmos/ibc-go/pull/1418) `RedundancyDecorator` has been renamed to `RedundantRelayDecorator` to make the name for explicit.
* (testing) [\#1418](https://github.com/cosmos/ibc-go/pull/1418) `MockIBCApp` has been renamed to `IBCApp` and `MockEmptyAcknowledgement` has been renamed to `EmptyAcknowledgement` to comply with go linting rules
* (apps/27-interchain-accounts) [\#2058](https://github.com/cosmos/ibc-go/pull/2058) Added `MessageRouter` interface and replaced `*baseapp.MsgServiceRouter` with it. The controller and host keepers of apps/27-interchain-accounts have been updated to use it.
* (apps/27-interchain-accounts)[\#2302](https://github.com/cosmos/ibc-go/pull/2302) Handle unwrapping of channel version in interchain accounts channel reopening handshake flow. The `host` submodule `Keeper` now requires an `ICS4Wrapper` similarly to the `controller` submodule.

### Improvements

* (27-interchain-accounts) [\#1352](https://github.com/cosmos/ibc-go/pull/1352) Add support for Cosmos-SDK simulation to ics27 module.  
* (linting) [\#1418](https://github.com/cosmos/ibc-go/pull/1418) Fix linting errors, resulting compatiblity with go1.18 linting style, golangci-lint 1.46.2 and the revivie linter.  This caused breaking changes in core/04-channel, core/ante, and the testing library.

### Features

* (apps/27-interchain-accounts) [\#2193](https://github.com/cosmos/ibc-go/pull/2193) Adding `InterchainAccount` gRPC query endpont to ICS27 `controller` submodule to allow users to retrieve registered interchain account addresses.

### Bug Fixes

* (27-interchain-accounts) [\#2308](https://github.com/cosmos/ibc-go/pull/2308) Nil checks have been added to ensure services are not registered for nil host or controller keepers.
* (makefile) [\#1785](https://github.com/cosmos/ibc-go/pull/1785) Fetch the correct versions of protocol buffers dependencies from tendermint, cosmos-sdk, and ics23.
* (modules/core/04-channel)[\#1919](https://github.com/cosmos/ibc-go/pull/1919) Fixed formatting of sequence for packet "acknowledgement written" logs.

## [v4.4.2](https://github.com/cosmos/ibc-go/releases/tag/v4.4.2) - 2023-05-25

### Bug Fixes

* [\#3662](https://github.com/cosmos/ibc-go/pull/3662) Retract v4.1.2 and v4.2.1.

## [v4.4.1](https://github.com/cosmos/ibc-go/releases/tag/v4.4.1) - 2023-05-25

### Bug Fixes

* [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.

## [v4.4.0](https://github.com/cosmos/ibc-go/releases/tag/v4.4.0) - 2023-04-25

### Dependencies

* [\#3416](https://github.com/cosmos/ibc-go/pull/3416) Bump Cosmos SDK to v0.45.15 and replace Tendermint with CometBFT v0.34.27.

## [v4.3.1](https://github.com/cosmos/ibc-go/releases/tag/v4.3.1) - 2023-05-25

### Bug Fixes

* [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.

## [v4.3.0](https://github.com/cosmos/ibc-go/releases/tag/v4.3.0) - 2023-01-24

### Dependencies

* [\#3049](https://github.com/cosmos/ibc-go/pull/3049) Bump Cosmos SDK to v0.45.12.
* [\#2868](https://github.com/cosmos/ibc-go/pull/2868) Bump ics23 to v0.9.0.

### State Machine Breaking

* (core/04-channel) [\#2973](https://github.com/cosmos/ibc-go/pull/2973) Write channel state before invoking app callbacks in ack and confirm channel handshake steps.

### Improvements

* (apps/29-fee) [\#2786](https://github.com/cosmos/ibc-go/pull/2786) Save gas on `IsFeeEnabled`.

### Bug Fixes

* (apps/29-fee) [\#2942](https://github.com/cosmos/ibc-go/pull/2942) Check `x/bank` send enabled before escrowing fees.

### Documentation

* [\#2737](https://github.com/cosmos/ibc-go/pull/2737) Fix migration/docs for ICA controller middleware.

### Miscellaneous Tasks

* [\#2772](https://github.com/cosmos/ibc-go/pull/2772) Integrated git cliff into the code base to automate generation of changelogs.

## [v4.2.2](https://github.com/cosmos/ibc-go/releases/tag/v4.2.2) - 2023-05-25

### Bug Fixes

* [\#3661](https://github.com/cosmos/ibc-go/pull/3661) Revert state-machine breaking improvement from PR [#2786](https://github.com/cosmos/ibc-go/pull/2786).

## [v4.2.1](https://github.com/cosmos/ibc-go/releases/tag/v4.2.1) - 2023-05-25

### Dependencies

* [\#2868](https://github.com/cosmos/ibc-go/pull/2868) Bump ICS 23 to v0.9.0.

### Improvements

* (apps/29-fee) [\#2786](https://github.com/cosmos/ibc-go/pull/2786) Save gas by checking key existence with `KVStore`'s `Has` method.

### Bug Fixes

* [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.

## [v4.2.0](https://github.com/cosmos/ibc-go/releases/tag/v4.2.0) - 2022-11-07

### Dependencies

* [\#2588](https://github.com/cosmos/ibc-go/pull/2588) Bump SDK version to v0.45.10 and Tendermint to v0.34.22.

### State Machine Breaking

* (apps/transfer) [\#2651](https://github.com/cosmos/ibc-go/pull/2651) Introduce `mustProtoMarshalJSON` for ics20 packet data marshalling which will skip emission (marshalling) of the memo field if unpopulated (empty).
* (27-interchain-accounts) [\#2590](https://github.com/cosmos/ibc-go/pull/2590) Removing port prefix requirement from the ICA host channel handshake
* (transfer) [\#2377](https://github.com/cosmos/ibc-go/pull/2377) Adding `sequence` to `MsgTransferResponse`.

### Features

* (apps/transfer) [\#2595](https://github.com/cosmos/ibc-go/pull/2595) Adding optional memo field to `FungibleTokenPacketData` and `MsgTransfer`.

### Bug Fixes

* (apps/transfer) [\#2679](https://github.com/cosmos/ibc-go/pull/2679) Check `x/bank` send enabled.

## [v4.1.3](https://github.com/cosmos/ibc-go/releases/tag/v4.1.3) - 2023-05-25

### Bug Fixes

* [\#3660](https://github.com/cosmos/ibc-go/pull/3660) Revert state-machine breaking improvement from PR [#2786](https://github.com/cosmos/ibc-go/pull/2786).

## [v4.1.2](https://github.com/cosmos/ibc-go/releases/tag/v4.1.2) - 2023-05-25

### Dependencies

* [\#2868](https://github.com/cosmos/ibc-go/pull/2868) Bump ICS 23 to v0.9.0.

### Improvements

* (apps/29-fee) [\#2786](https://github.com/cosmos/ibc-go/pull/2786) Save gas by checking key existence with `KVStore`'s `Has` method.

### Bug Fixes

* [\#3346](https://github.com/cosmos/ibc-go/pull/3346) Properly handle ordered channels in `UnreceivedPackets` query.

## [v4.1.1](https://github.com/cosmos/ibc-go/releases/tag/v4.1.1) - 2022-10-27

### Dependencies

* [\#2624](https://github.com/cosmos/ibc-go/pull/2624) Bump SDK version to v0.45.10 and Tendermint to v0.34.22.

## [v4.1.0](https://github.com/cosmos/ibc-go/releases/tag/v4.1.0) - 2022-09-20

### Dependencies

* [\#2288](https://github.com/cosmos/ibc-go/pull/2288) Bump SDK version to v0.45.8 and Tendermint to v0.34.21.

### Features

* (apps/27-interchain-accounts) [\#2193](https://github.com/cosmos/ibc-go/pull/2193) Adding `InterchainAccount` gRPC query endpont to ICS27 `controller` submodule to allow users to retrieve registered interchain account addresses.

### Bug Fixes

* (27-interchain-accounts) [\#2308](https://github.com/cosmos/ibc-go/pull/2308) Nil checks have been added to ensure services are not registered for nil host or controller keepers.

## [v4.0.1](https://github.com/cosmos/ibc-go/releases/tag/v4.0.1) - 2022-09-15

### Dependencies

* [\#2287](https://github.com/cosmos/ibc-go/pull/2287) Bump SDK version to v0.45.8 and Tendermint to v0.34.21.

## [v4.0.0](https://github.com/cosmos/ibc-go/releases/tag/v4.0.0) - 2022-08-12

### Dependencies

* [\#1627](https://github.com/cosmos/ibc-go/pull/1627) Bump Go version to 1.18
* [\#1905](https://github.com/cosmos/ibc-go/pull/1905) Bump SDK version to v0.45.7

### API Breaking

* (core/04-channel) [\#1792](https://github.com/cosmos/ibc-go/pull/1792) Remove `PreviousChannelID` from `NewMsgChannelOpenTry` arguments. `MsgChannelOpenTry.ValidateBasic()` returns error if the deprecated `PreviousChannelID` is not empty.
* (core/03-connection) [\#1797](https://github.com/cosmos/ibc-go/pull/1797) Remove `PreviousConnectionID` from `NewMsgConnectionOpenTry` arguments. `MsgConnectionOpenTry.ValidateBasic()` returns error if the deprecated `PreviousConnectionID` is not empty.
* (modules/core/03-connection) [\#1672](https://github.com/cosmos/ibc-go/pull/1672) Remove crossing hellos from connection handshakes. The `PreviousConnectionId` in `MsgConnectionOpenTry` has been deprecated.
* (modules/core/04-channel) [\#1317](https://github.com/cosmos/ibc-go/pull/1317) Remove crossing hellos from channel handshakes. The `PreviousChannelId` in `MsgChannelOpenTry` has been deprecated.  
* (transfer) [\#1250](https://github.com/cosmos/ibc-go/pull/1250) Deprecate `GetTransferAccount` since the `transfer` module account is never used.
* (channel) [\#1283](https://github.com/cosmos/ibc-go/pull/1283) The `OnChanOpenInit` application callback now returns a version string in line with the latest [spec changes](https://github.com/cosmos/ibc/pull/629).  
* (modules/29-fee)[\#1338](https://github.com/cosmos/ibc-go/pull/1338) Renaming `Result` field in `IncentivizedAcknowledgement` to `AppAcknowledgement`.
* (modules/29-fee)[\#1343](https://github.com/cosmos/ibc-go/pull/1343) Renaming `KeyForwardRelayerAddress` to `KeyRelayerAddressForAsyncAck`, and `ParseKeyForwardRelayerAddress` to `ParseKeyRelayerAddressForAsyncAck`.
* (apps/27-interchain-accounts)[\#1432](https://github.com/cosmos/ibc-go/pull/1432) Updating `RegisterInterchainAccount` to include an additional `version` argument, supporting ICS29 fee middleware functionality in ICS27 interchain accounts.
* (apps/27-interchain-accounts)[\#1565](https://github.com/cosmos/ibc-go/pull/1565) Removing `NewErrorAcknowledgement` in favour of `channeltypes.NewErrorAcknowledgement`.
* (transfer)[\#1565](https://github.com/cosmos/ibc-go/pull/1565) Removing `NewErrorAcknowledgement` in favour of `channeltypes.NewErrorAcknowledgement`.
* (channel)[\#1565](https://github.com/cosmos/ibc-go/pull/1565) Updating `NewErrorAcknowledgement` to accept an error instead of a string and removing the possibility of non-deterministic writes to application state.
* (core/04-channel)[\#1636](https://github.com/cosmos/ibc-go/pull/1636) Removing `SplitChannelVersion` and `MergeChannelVersions` functions since they are not used.

### State Machine Breaking

* (apps/transfer) [\#1907](https://github.com/cosmos/ibc-go/pull/1907) Blocked module account addresses are no longer allowed to send IBC transfers.
* (apps/27-interchain-accounts) [\#1882](https://github.com/cosmos/ibc-go/pull/1882) Explicitly check length of interchain account packet data in favour of nil check.

### Improvements

* (app/20-transfer) [\#1680](https://github.com/cosmos/ibc-go/pull/1680) Adds migration to correct any malformed trace path information of tokens with denoms that contains slashes. The transfer module consensus version has been bumped to 2.
* (app/20-transfer) [\#1730](https://github.com/cosmos/ibc-go/pull/1730) parse the ics20 denomination provided via a packet using the channel identifier format specified by ibc-go.
* (cleanup) [\#1335](https://github.com/cosmos/ibc-go/pull/1335/) `gofumpt -w -l .` to standardize the code layout more strictly than `go fmt ./...`
* (middleware) [\#1022](https://github.com/cosmos/ibc-go/pull/1022) Add `GetAppVersion` to the ICS4Wrapper interface. This function should be used by IBC applications to obtain their own version since the version set in the channel structure may be wrapped many times by middleware.
* (modules/core/04-channel) [\#1232](https://github.com/cosmos/ibc-go/pull/1232) Updating params on `NewPacketId` and moving to bottom of file.
* (app/29-fee) [\#1305](https://github.com/cosmos/ibc-go/pull/1305) Change version string for fee module to `ics29-1`
* (app/29-fee) [\#1341](https://github.com/cosmos/ibc-go/pull/1341) Check if the fee module is locked and if the fee module is enabled before refunding all fees
* (transfer) [\#1414](https://github.com/cosmos/ibc-go/pull/1414) Emitting Sender address from `fungible_token_packet` events in `OnRecvPacket` and `OnAcknowledgementPacket`.
* (testing/simapp) [\#1397](https://github.com/cosmos/ibc-go/pull/1397) Adding mock module to maccperms and adding check to ensure mock module is not a blocked account address.
* (core/02-client) [\#1570](https://github.com/cosmos/ibc-go/pull/1570) Emitting an event when handling an upgrade client proposal.
* (modules/light-clients/07-tendermint) [\#1713](https://github.com/cosmos/ibc-go/pull/1713) Allow client upgrade proposals to update `TrustingPeriod`. See ADR-026 for context.
* (core/client) [\#1740](https://github.com/cosmos/ibc-go/pull/1740) Add `cosmos_proto.implements_interface` to adhere to guidelines in [Cosmos SDK ADR 019](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-019-protobuf-state-encoding.md#safe-usage-of-any) for annotating `google.protobuf.Any` types

### Features

* [\#276](https://github.com/cosmos/ibc-go/pull/276) Adding the Fee Middleware module v1
* (apps/29-fee) [\#1229](https://github.com/cosmos/ibc-go/pull/1229) Adding CLI commands for getting all unrelayed incentivized packets and packet by packet-id.
* (apps/29-fee) [\#1224](https://github.com/cosmos/ibc-go/pull/1224) Adding Query/CounterpartyAddress and CLI to ICS29 fee middleware
* (apps/29-fee) [\#1225](https://github.com/cosmos/ibc-go/pull/1225) Adding Query/FeeEnabledChannel and Query/FeeEnabledChannels with CLIs to ICS29 fee middleware.
* (modules/apps/29-fee) [\#1230](https://github.com/cosmos/ibc-go/pull/1230) Adding CLI command for getting incentivized packets for a specific channel-id.

### Bug Fixes

* (apps/29-fee) [\#1774](https://github.com/cosmos/ibc-go/pull/1774) Change non nil relayer assertion to non empty to avoid import/export issues for genesis upgrades.
* (apps/29-fee) [\#1278](https://github.com/cosmos/ibc-go/pull/1278) The URI path for the query to get all incentivized packets for a specific channel did not follow the same format as the rest of queries.
* (modules/core/04-channel)[\#1919](https://github.com/cosmos/ibc-go/pull/1919) Fixed formatting of sequence for packet "acknowledgement written" logs.

## [v3.4.0](https://github.com/cosmos/ibc-go/releases/tag/v3.4.0) - 2022-11-07

### Dependencies

* [\#2589](https://github.com/cosmos/ibc-go/pull/2589) Bump SDK version to v0.45.10 and Tendermint to v0.34.22.

### State Machine Breaking

* (apps/transfer) [\#2651](https://github.com/cosmos/ibc-go/pull/2651) Introduce `mustProtoMarshalJSON` for ics20 packet data marshalling which will skip emission (marshalling) of the memo field if unpopulated (empty).
* (27-interchain-accounts) [\#2590](https://github.com/cosmos/ibc-go/pull/2590) Removing port prefix requirement from the ICA host channel handshake
* (transfer) [\#2377](https://github.com/cosmos/ibc-go/pull/2377) Adding `sequence` to `MsgTransferResponse`.

### Features

* (apps/transfer) [\#2595](https://github.com/cosmos/ibc-go/pull/2595) Adding optional memo field to `FungibleTokenPacketData` and `MsgTransfer`.

### Bug Fixes

* (apps/transfer) [\#2679](https://github.com/cosmos/ibc-go/pull/2679) Check `x/bank` send enabled.

## [v3.3.1](https://github.com/cosmos/ibc-go/releases/tag/v3.3.1) - 2022-10-27

### Dependencies

* [\#2621](https://github.com/cosmos/ibc-go/pull/2621) Bump SDK version to v0.45.10 and Tendermint to v0.34.22.

## [v3.3.0](https://github.com/cosmos/ibc-go/releases/tag/v3.3.0) - 2022-09-20

### Dependencies

* [\#2286](https://github.com/cosmos/ibc-go/pull/2286) Bump SDK version to v0.45.8 and Tendermint to v0.34.21.

### Features

* (apps/27-interchain-accounts) [\#2193](https://github.com/cosmos/ibc-go/pull/2193) Adding `InterchainAccount` gRPC query endpont to ICS27 `controller` submodule to allow users to retrieve registered interchain account addresses.

### Bug Fixes

* (27-interchain-accounts) [\#2308](https://github.com/cosmos/ibc-go/pull/2308) Nil checks have been added to ensure services are not registered for nil host or controller keepers.

## [v3.2.1](https://github.com/cosmos/ibc-go/releases/tag/v3.2.1) - 2022-09-15

### Dependencies

* [\#2285](https://github.com/cosmos/ibc-go/pull/2285) Bump SDK version to v0.45.8 and Tendermint to v0.34.21.

## [v3.2.0](https://github.com/cosmos/ibc-go/releases/tag/v3.2.0) - 2022-08-12

### Dependencies

* [\#1627](https://github.com/cosmos/ibc-go/pull/1627) Bump Go version to 1.18
* [\#1905](https://github.com/cosmos/ibc-go/pull/1905) Bump SDK version to v0.45.7

### State Machine Breaking

* (apps/transfer) [\#1907](https://github.com/cosmos/ibc-go/pull/1907) Blocked module account addresses are no longer allowed to send IBC transfers.
* (apps/27-interchain-accounts) [\#1882](https://github.com/cosmos/ibc-go/pull/1882) Explicitly check length of interchain account packet data in favour of nil check.

### Improvements

* (core/02-client) [\#1570](https://github.com/cosmos/ibc-go/pull/1570) Emitting an event when handling an upgrade client proposal.
* (modules/light-clients/07-tendermint) [\#1713](https://github.com/cosmos/ibc-go/pull/1713) Allow client upgrade proposals to update `TrustingPeriod`. See ADR-026 for context.
* (app/20-transfer) [\#1680](https://github.com/cosmos/ibc-go/pull/1680) Adds migration to correct any malformed trace path information of tokens with denoms that contains slashes. The transfer module consensus version has been bumped to 2.
* (app/20-transfer) [\#1730](https://github.com/cosmos/ibc-go/pull/1730) parse the ics20 denomination provided via a packet using the channel identifier format specified by ibc-go.
* (core/client) [\#1740](https://github.com/cosmos/ibc-go/pull/1740) Add `cosmos_proto.implements_interface` to adhere to guidelines in [Cosmos SDK ADR 019](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-019-protobuf-state-encoding.md#safe-usage-of-any) for annotating `google.protobuf.Any` types

### Bug Fixes

* (modules/core/04-channel)[\#1919](https://github.com/cosmos/ibc-go/pull/1919) Fixed formatting of sequence for packet "acknowledgement written" logs.

## [v3.1.1](https://github.com/cosmos/ibc-go/releases/tag/v3.1.1) - 2022-08-02

### Dependencies

* [\#1525](https://github.com/cosmos/ibc-go/pull/1525) Bump SDK version to v0.45.5

### Improvements

* (core/02-client) [\#1570](https://github.com/cosmos/ibc-go/pull/1570) Emitting an event when handling an upgrade client proposal.
* (core/client) [\#1740](https://github.com/cosmos/ibc-go/pull/1740) Add `cosmos_proto.implements_interface` to adhere to guidelines in [Cosmos SDK ADR 019](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-019-protobuf-state-encoding.md#safe-usage-of-any) for annotating `google.protobuf.Any` types

## [v3.1.0](https://github.com/cosmos/ibc-go/releases/tag/v3.1.0) - 2022-06-14

### Dependencies

* [\#1300](https://github.com/cosmos/ibc-go/pull/1300) Bump SDK version to v0.45.4

### Improvements

* (transfer) [\#1342](https://github.com/cosmos/ibc-go/pull/1342) `DenomTrace` grpc now takes in either an `ibc denom` or a `hash` instead of only accepting a `hash`.
* (modules/core/04-channel) [\#1160](https://github.com/cosmos/ibc-go/pull/1160) Improve `uint64 -> string` performance in `Logger`.
* (modules/core/04-channel) [\#1279](https://github.com/cosmos/ibc-go/pull/1279) Add selected channel version to MsgChanOpenInitResponse and MsgChanOpenTryResponse. Emit channel version during OpenInit/OpenTry
* (modules/core/keeper) [\#1284](https://github.com/cosmos/ibc-go/pull/1284) Add sanity check for the keepers passed into `ibckeeper.NewKeeper`. `ibckeeper.NewKeeper` now panics if any of the keepers passed in is empty.
* (transfer) [\#1414](https://github.com/cosmos/ibc-go/pull/1414) Emitting Sender address from `fungible_token_packet` events in `OnRecvPacket` and `OnAcknowledgementPacket`.
* (modules/core/04-channel) [\#1464](https://github.com/cosmos/ibc-go/pull/1464) Emit a channel close event when an ordered channel is closed.
* (modules/light-clients/07-tendermint) [\#1118](https://github.com/cosmos/ibc-go/pull/1118) Deprecating `AllowUpdateAfterExpiry` and `AllowUpdateAfterMisbehaviour`. See ADR-026 for context.

### Features

* (modules/core/02-client) [\#1336](https://github.com/cosmos/ibc-go/pull/1336) Adding Query/ConsensusStateHeights gRPC for fetching the height of every consensus state associated with a client.
* (modules/apps/transfer) [\#1416](https://github.com/cosmos/ibc-go/pull/1416) Adding gRPC endpoint for getting an escrow account for a given port-id and channel-id.
* (modules/apps/27-interchain-accounts) [\#1512](https://github.com/cosmos/ibc-go/pull/1512) Allowing ICA modules to handle all message types with "*".

### Bug Fixes

* (modules/core/04-channel) [\#1130](https://github.com/cosmos/ibc-go/pull/1130) Call `packet.GetSequence()` rather than passing func in `WriteAcknowledgement` log output
* (apps/transfer) [\#1451](https://github.com/cosmos/ibc-go/pull/1451) Fixing the support for base denoms that contain slashes.

## [v3.0.2](https://github.com/cosmos/ibc-go/releases/tag/v3.0.2) - 2022-08-02

### Improvements

* (core/02-client) [\#1570](https://github.com/cosmos/ibc-go/pull/1570) Emitting an event when handling an upgrade client proposal.
* (core/client) [\#1740](https://github.com/cosmos/ibc-go/pull/1740) Add `cosmos_proto.implements_interface` to adhere to guidelines in [Cosmos SDK ADR 019](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-019-protobuf-state-encoding.md#safe-usage-of-any) for annotating `google.protobuf.Any` types

## [v3.0.1](https://github.com/cosmos/ibc-go/releases/tag/v3.0.1) - 2022-06-14

### Dependencies

* [\#1300](https://github.com/cosmos/ibc-go/pull/1300) Bump SDK version to v0.45.4

### Improvements

* (transfer) [\#1342](https://github.com/cosmos/ibc-go/pull/1342) `DenomTrace` grpc now takes in either an `ibc denom` or a `hash` instead of only accepting a `hash`.
* (modules/core/04-channel) [\#1160](https://github.com/cosmos/ibc-go/pull/1160) Improve `uint64 -> string` performance in `Logger`.
* (modules/core/keeper) [\#1284](https://github.com/cosmos/ibc-go/pull/1284) Add sanity check for the keepers passed into `ibckeeper.NewKeeper`. `ibckeeper.NewKeeper` now panics if any of the keepers passed in is empty.
* (transfer) [\#1414](https://github.com/cosmos/ibc-go/pull/1414) Emitting Sender address from `fungible_token_packet` events in `OnRecvPacket` and `OnAcknowledgementPacket`.
* (modules/core/04-channel) [\#1464](https://github.com/cosmos/ibc-go/pull/1464) Emit a channel close event when an ordered channel is closed.

### Bug Fixes

* (modules/core/04-channel) [\#1130](https://github.com/cosmos/ibc-go/pull/1130) Call `packet.GetSequence()` rather than passing func in `WriteAcknowledgement` log output

## [v3.0.0](https://github.com/cosmos/ibc-go/releases/tag/v3.0.0) - 2022-03-15

### Dependencies

* [\#404](https://github.com/cosmos/ibc-go/pull/404) Bump Go version to 1.17
* [\#851](https://github.com/cosmos/ibc-go/pull/851) Bump SDK version to v0.45.1
* [\#948](https://github.com/cosmos/ibc-go/pull/948) Bump ics23/go to v0.7
* (core) [\#709](https://github.com/cosmos/ibc-go/pull/709) Replace github.com/pkg/errors with stdlib errors

### API Breaking

* (testing) [\#939](https://github.com/cosmos/ibc-go/pull/939) Support custom power reduction for testing.
* (modules/core/05-port) [\#1086](https://github.com/cosmos/ibc-go/pull/1086) Added `counterpartyChannelID` argument to IBCModule.OnChanOpenAck
* (channel) [\#848](https://github.com/cosmos/ibc-go/pull/848) Added `ChannelId` to MsgChannelOpenInitResponse
* (testing) [\#813](https://github.com/cosmos/ibc-go/pull/813) The `ack` argument to the testing function `RelayPacket` has been removed as it is no longer needed.
* (testing) [\#774](https://github.com/cosmos/ibc-go/pull/774) Added `ChainID` arg to `SetupWithGenesisValSet` on the testing app. `Coordinator` generated ChainIDs now starts at index 1
* (transfer) [\#675](https://github.com/cosmos/ibc-go/pull/675) Transfer `NewKeeper` now takes in an ICS4Wrapper. The ICS4Wrapper may be the IBC Channel Keeper when ICS20 is not used in a middleware stack. The ICS4Wrapper is required for applications wishing to connect middleware to ICS20.
* (core) [\#650](https://github.com/cosmos/ibc-go/pull/650) Modify `OnChanOpenTry` IBC application module callback to return the negotiated app version. The version passed into the `MsgChanOpenTry` has been deprecated and will be ignored by core IBC.
* (core) [\#629](https://github.com/cosmos/ibc-go/pull/629) Removes the `GetProofSpecs` from the ClientState interface. This function was previously unused by core IBC.
* (transfer) [\#517](https://github.com/cosmos/ibc-go/pull/517) Separates the ICS 26 callback functions from `AppModule` into a new type `IBCModule` for ICS 20 transfer.
* (modules/core/02-client) [\#536](https://github.com/cosmos/ibc-go/pull/536) `GetSelfConsensusState` return type changed from bool to error.
* (channel) [\#644](https://github.com/cosmos/ibc-go/pull/644) Removes `CounterpartyHops` function from the ChannelKeeper.
* (testing) [\#776](https://github.com/cosmos/ibc-go/pull/776) Adding helper fn to generate capability name for testing callbacks
* (testing) [\#892](https://github.com/cosmos/ibc-go/pull/892) IBC Mock modules store the scoped keeper and portID within the IBCMockApp. They also maintain reference to the AppModule to update the AppModule's list of IBC applications it references. Allows for the mock module to be reused as a base application in middleware stacks.
* (channel) [\#882](https://github.com/cosmos/ibc-go/pull/882) The `WriteAcknowledgement` API now takes `exported.Acknowledgement` instead of a byte array
* (modules/core/ante) [\#950](https://github.com/cosmos/ibc-go/pull/950) Replaces the channel keeper with the IBC keeper in the IBC `AnteDecorator` in order to execute the entire message and be able to reject redundant messages that are in the same block as the non-redundant messages.

### State Machine Breaking

* (transfer) [\#818](https://github.com/cosmos/ibc-go/pull/818) Error acknowledgements returned from Transfer `OnRecvPacket` now include a deterministic ABCI code and error message.

### Improvements

* (client) [\#888](https://github.com/cosmos/ibc-go/pull/888) Add `GetTimestampAtHeight` to `ClientState`
* (interchain-accounts) [\#1037](https://github.com/cosmos/ibc-go/pull/1037) Add a function `InitModule` to the interchain accounts `AppModule`. This function should be called within the upgrade handler when adding the interchain accounts module to a chain. It should be called in place of InitGenesis (set the consensus version in the version map).
* (testing) [\#942](https://github.com/cosmos/ibc-go/pull/942) `NewTestChain` will create 4 validators in validator set by default. A new constructor function `NewTestChainWithValSet` is provided for test writers who want custom control over the validator set of test chains.
* (testing) [\#904](https://github.com/cosmos/ibc-go/pull/904) Add `ParsePacketFromEvents` function to the testing package. Useful when sending/relaying packets via the testing package.
* (testing) [\#893](https://github.com/cosmos/ibc-go/pull/893) Support custom private keys for testing.
* (testing) [\#810](https://github.com/cosmos/ibc-go/pull/810) Additional testing function added to `Endpoint` type called `RecvPacketWithResult`. Performs the same functionality as the existing `RecvPacket` function but also returns the message result. `path.RelayPacket` no longer uses the provided acknowledgement argument and instead obtains the acknowledgement via MsgRecvPacket events.
* (connection) [\#721](https://github.com/cosmos/ibc-go/pull/721) Simplify connection handshake error messages when unpacking client state.
* (channel) [\#692](https://github.com/cosmos/ibc-go/pull/692) Minimize channel logging by only emitting the packet sequence, source port/channel, destination port/channel upon packet receives, acknowledgements and timeouts.
* [\#383](https://github.com/cosmos/ibc-go/pull/383) Adds helper functions for merging and splitting middleware versions from the underlying app version.
* (modules/core/05-port) [\#288](https://github.com/cosmos/ibc-go/pull/288) Making the 05-port keeper function IsBound public. The IsBound function checks if the provided portID is already binded to a module.
* (client) [\#724](https://github.com/cosmos/ibc-go/pull/724) `IsRevisionFormat` and `IsClientIDFormat` have been updated to disallow newlines before the dash used to separate the chainID and revision number, and the client type and client sequence.
* (channel) [\#644](https://github.com/cosmos/ibc-go/pull/644) Adds `GetChannelConnection` to the ChannelKeeper. This function returns the connectionID and connection state associated with a channel.
* (channel) [\647](https://github.com/cosmos/ibc-go/pull/647) Reorganizes channel handshake handling to set channel state after IBC application callbacks.
* (interchain-accounts) [\#1466](https://github.com/cosmos/ibc-go/pull/1466) Emit event when there is an acknowledgement during `OnRecvPacket`.

### Features

* [\#432](https://github.com/cosmos/ibc-go/pull/432) Introduce `MockIBCApp` struct to the mock module. Allows the mock module to be reused to perform custom logic on each IBC App interface function. This might be useful when testing out IBC applications written as middleware.
* [\#380](https://github.com/cosmos/ibc-go/pull/380) Adding the Interchain Accounts module v1
* [\#679](https://github.com/cosmos/ibc-go/pull/679) New CLI command `query ibc-transfer denom-hash <denom trace>` to get the denom hash for a denom trace; this might be useful for debug

### Bug Fixes

* (testing) [\#884](https://github.com/cosmos/ibc-go/pull/884) Add and use in simapp a custom ante handler that rejects redundant transactions
* (transfer) [\#978](https://github.com/cosmos/ibc-go/pull/978) Support base denoms with slashes in denom validation
* (client) [\#941](https://github.com/cosmos/ibc-go/pull/941) Classify client states without consensus states as expired
* (channel) [\#995](https://github.com/cosmos/ibc-go/pull/995) Call `packet.GetSequence()` rather than passing func in `AcknowledgePacket` log output

## [v2.5.0](https://github.com/cosmos/ibc-go/releases/tag/v2.5.0) - 2022-11-07

### Dependencies

* [\#2578](https://github.com/cosmos/ibc-go/pull/2578) Bump SDK version to v0.45.10 and Tendermint to v0.34.22.

### State Machine Breaking

* (apps/transfer) [\#2651](https://github.com/cosmos/ibc-go/pull/2651) Introduce `mustProtoMarshalJSON` for ics20 packet data marshalling which will skip emission (marshalling) of the memo field if unpopulated (empty).
* (transfer) [\#2377](https://github.com/cosmos/ibc-go/pull/2377) Adding `sequence` to `MsgTransferResponse`.

### Features

* (apps/transfer) [\#2595](https://github.com/cosmos/ibc-go/pull/2595) Adding optional memo field to `FungibleTokenPacketData` and `MsgTransfer`.

### Bug Fixes

* (apps/transfer) [\#2679](https://github.com/cosmos/ibc-go/pull/2679) Check `x/bank` send enabled.

## [v2.4.2](https://github.com/cosmos/ibc-go/releases/tag/v2.4.2) - 2022-10-27

### Dependencies

* [\#2622](https://github.com/cosmos/ibc-go/pull/2622) Bump SDK version to v0.45.10 and Tendermint to v0.34.22.

## [v2.4.1](https://github.com/cosmos/ibc-go/releases/tag/v2.4.1) - 2022-09-15

### Dependencies

* [\#2284](https://github.com/cosmos/ibc-go/pull/2284) Bump SDK version to v0.45.8 and Tendermint to v0.34.21.

## [v2.4.0](https://github.com/cosmos/ibc-go/releases/tag/v2.4.0) - 2022-08-12

### Dependencies

* [\#1627](https://github.com/cosmos/ibc-go/pull/1627) Bump Go version to 1.18
* [\#1905](https://github.com/cosmos/ibc-go/pull/1905) Bump SDK version to v0.45.7

### State Machine Breaking

* (apps/transfer) [\#1907](https://github.com/cosmos/ibc-go/pull/1907) Blocked module account addresses are no longer allowed to send IBC transfers.

### Improvements

* (modules/light-clients/07-tendermint) [\#1713](https://github.com/cosmos/ibc-go/pull/1713) Allow client upgrade proposals to update `TrustingPeriod`. See ADR-026 for context.
* (core/02-client) [\#1570](https://github.com/cosmos/ibc-go/pull/1570) Emitting an event when handling an upgrade client proposal.
* (app/20-transfer) [\#1680](https://github.com/cosmos/ibc-go/pull/1680) Adds migration to correct any malformed trace path information of tokens with denoms that contains slashes. The transfer module consensus version has been bumped to 2.
* (app/20-transfer) [\#1730](https://github.com/cosmos/ibc-go/pull/1730) parse the ics20 denomination provided via a packet using the channel identifier format specified by ibc-go.
* (core/client) [\#1740](https://github.com/cosmos/ibc-go/pull/1740) Add `cosmos_proto.implements_interface` to adhere to guidelines in [Cosmos SDK ADR 019](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-019-protobuf-state-encoding.md#safe-usage-of-any) for annotating `google.protobuf.Any` types

### Bug Fixes

* (modules/core/04-channel)[\#1919](https://github.com/cosmos/ibc-go/pull/1919) Fixed formatting of sequence for packet "acknowledgement written" logs.

## [v2.3.1](https://github.com/cosmos/ibc-go/releases/tag/v2.3.1) - 2022-08-02

### Dependencies

* [\#1525](https://github.com/cosmos/ibc-go/pull/1525) Bump SDK version to v0.45.5

### Improvements

* (core/02-client) [\#1570](https://github.com/cosmos/ibc-go/pull/1570) Emitting an event when handling an upgrade client proposal.
* (core/client) [\#1740](https://github.com/cosmos/ibc-go/pull/1740) Add `cosmos_proto.implements_interface` to adhere to guidelines in [Cosmos SDK ADR 019](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-019-protobuf-state-encoding.md#safe-usage-of-any) for annotating `google.protobuf.Any` types

## [v2.3.0](https://github.com/cosmos/ibc-go/releases/tag/v2.3.0) - 2022-06-14

### Dependencies

* [\#404](https://github.com/cosmos/ibc-go/pull/404) Bump Go version to 1.17
* [\#1300](https://github.com/cosmos/ibc-go/pull/1300) Bump SDK version to v0.45.4

### Improvements

* (transfer) [\#1342](https://github.com/cosmos/ibc-go/pull/1342) `DenomTrace` grpc now takes in either an `ibc denom` or a `hash` instead of only accepting a `hash`.
* (modules/core/04-channel) [\#1160](https://github.com/cosmos/ibc-go/pull/1160) Improve `uint64 -> string` performance in `Logger`.
* (modules/core/keeper) [\#1284](https://github.com/cosmos/ibc-go/pull/1284) Add sanity check for the keepers passed into `ibckeeper.NewKeeper`. `ibckeeper.NewKeeper` now panics if any of the keepers passed in is empty.
* (transfer) [\#1414](https://github.com/cosmos/ibc-go/pull/1414) Emitting Sender address from `fungible_token_packet` events in `OnRecvPacket` and `OnAcknowledgementPacket`.
* (modules/core/04-channel) [\#1464](https://github.com/cosmos/ibc-go/pull/1464) Emit a channel close event when an ordered channel is closed.
* (modules/light-clients/07-tendermint) [\#1118](https://github.com/cosmos/ibc-go/pull/1118) Deprecating `AllowUpdateAfterExpiry` and `AllowUpdateAfterMisbehaviour`. See ADR-026 for context.

### Features

* (modules/core/02-client) [\#1336](https://github.com/cosmos/ibc-go/pull/1336) Adding Query/ConsensusStateHeights gRPC for fetching the height of every consensus state associated with a client.
* (modules/apps/transfer) [\#1416](https://github.com/cosmos/ibc-go/pull/1416) Adding gRPC endpoint for getting an escrow account for a given port-id and channel-id.

### Bug Fixes

* (modules/core/04-channel) [\#1130](https://github.com/cosmos/ibc-go/pull/1130) Call `packet.GetSequence()` rather than passing func in `WriteAcknowledgement` log output
* (apps/transfer) [\#1451](https://github.com/cosmos/ibc-go/pull/1451) Fixing the support for base denoms that contain slashes.

## [v2.2.2](https://github.com/cosmos/ibc-go/releases/tag/v2.2.2) - 2022-08-02

### Improvements

* (core/02-client) [\#1570](https://github.com/cosmos/ibc-go/pull/1570) Emitting an event when handling an upgrade client proposal.
* (core/client) [\#1740](https://github.com/cosmos/ibc-go/pull/1740) Add `cosmos_proto.implements_interface` to adhere to guidelines in [Cosmos SDK ADR 019](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-019-protobuf-state-encoding.md#safe-usage-of-any) for annotating `google.protobuf.Any` types

## [v2.2.1](https://github.com/cosmos/ibc-go/releases/tag/v2.2.1) - 2022-06-14

### Improvements

* (transfer) [\#1342](https://github.com/cosmos/ibc-go/pull/1342) `DenomTrace` grpc now takes in either an `ibc denom` or a `hash` instead of only accepting a `hash`.
* (modules/core/04-channel) [\#1160](https://github.com/cosmos/ibc-go/pull/1160) Improve `uint64 -> string` performance in `Logger`.
* (modules/core/keeper) [\#1284](https://github.com/cosmos/ibc-go/pull/1284) Add sanity check for the keepers passed into `ibckeeper.NewKeeper`. `ibckeeper.NewKeeper` now panics if any of the keepers passed in is empty.
* (transfer) [\#1414](https://github.com/cosmos/ibc-go/pull/1414) Emitting Sender address from `fungible_token_packet` events in `OnRecvPacket` and `OnAcknowledgementPacket`.
* (modules/core/04-channel) [\#1464](https://github.com/cosmos/ibc-go/pull/1464) Emit a channel close event when an ordered channel is closed.

### Bug Fixes

* (modules/core/04-channel) [\#1130](https://github.com/cosmos/ibc-go/pull/1130) Call `packet.GetSequence()` rather than passing func in `WriteAcknowledgement` log output

## [v2.2.0](https://github.com/cosmos/ibc-go/releases/tag/v2.2.0) - 2022-03-15

### Dependencies

* [\#851](https://github.com/cosmos/ibc-go/pull/851) Bump SDK version to v0.45.1

## [v2.1.2](https://github.com/cosmos/ibc-go/releases/tag/v2.1.2) - 2022-08-02

### Improvements

* (core/02-client) [\#1570](https://github.com/cosmos/ibc-go/pull/1570) Emitting an event when handling an upgrade client proposal.
* (core/client) [\#1740](https://github.com/cosmos/ibc-go/pull/1740) Add `cosmos_proto.implements_interface` to adhere to guidelines in [Cosmos SDK ADR 019](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-019-protobuf-state-encoding.md#safe-usage-of-any) for annotating `google.protobuf.Any` types

## [v2.1.1](https://github.com/cosmos/ibc-go/releases/tag/v2.1.1) - 2022-06-14

### Dependencies

* [\#1268](https://github.com/cosmos/ibc-go/pull/1268) Bump SDK version to v0.44.8 and Tendermint to version 0.34.19

### Improvements

* (transfer) [\#1342](https://github.com/cosmos/ibc-go/pull/1342) `DenomTrace` grpc now takes in either an `ibc denom` or a `hash` instead of only accepting a `hash`.
* (modules/core/keeper) [\#1284](https://github.com/cosmos/ibc-go/pull/1284) Add sanity check for the keepers passed into `ibckeeper.NewKeeper`. `ibckeeper.NewKeeper` now panics if any of the keepers passed in is empty.
* (transfer) [\#1414](https://github.com/cosmos/ibc-go/pull/1414) Emitting Sender address from `fungible_token_packet` events in `OnRecvPacket` and `OnAcknowledgementPacket`.
* (modules/core/04-channel) [\#1464](https://github.com/cosmos/ibc-go/pull/1464) Emit a channel close event when an ordered channel is closed.

### Bug Fixes

* (modules/core/04-channel) [\#1130](https://github.com/cosmos/ibc-go/pull/1130) Call `packet.GetSequence()` rather than passing func in `WriteAcknowledgement` log output

## [v2.1.0](https://github.com/cosmos/ibc-go/releases/tag/v2.1.0) - 2022-03-15

### Dependencies

* [\#1084](https://github.com/cosmos/ibc-go/pull/1084) Bump SDK version to v0.44.6
* [\#948](https://github.com/cosmos/ibc-go/pull/948) Bump ics23/go to v0.7

### State Machine Breaking

* (transfer) [\#818](https://github.com/cosmos/ibc-go/pull/818) Error acknowledgements returned from Transfer `OnRecvPacket` now include a deterministic ABCI code and error message.

### Features

* [\#679](https://github.com/cosmos/ibc-go/pull/679) New CLI command `query ibc-transfer denom-hash <denom trace>` to get the denom hash for a denom trace; this might be useful for debug

### Bug Fixes

* (client) [\#941](https://github.com/cosmos/ibc-go/pull/941) Classify client states without consensus states as expired
* (transfer) [\#978](https://github.com/cosmos/ibc-go/pull/978) Support base denoms with slashes in denom validation
* (channel) [\#995](https://github.com/cosmos/ibc-go/pull/995) Call `packet.GetSequence()` rather than passing func in `AcknowledgePacket` log output

## [v2.0.3](https://github.com/cosmos/ibc-go/releases/tag/v2.0.3) - 2022-02-03

### Improvements

* (channel) [\#692](https://github.com/cosmos/ibc-go/pull/692) Minimize channel logging by only emitting the packet sequence, source port/channel, destination port/channel upon packet receives, acknowledgements and timeouts.

## [v2.0.2](https://github.com/cosmos/ibc-go/releases/tag/v2.0.2) - 2021-12-15

### Dependencies

* [\#589](https://github.com/cosmos/ibc-go/pull/589) Bump SDK version to v0.44.5

### Bug Fixes

* (modules/core) [\#603](https://github.com/cosmos/ibc-go/pull/603) Fix module name emitted as part of `OnChanOpenInit` event. Replacing `connection` module name with `channel`.

## [v2.0.1](https://github.com/cosmos/ibc-go/releases/tag/v2.0.1) - 2021-12-05

### Dependencies

* [\#567](https://github.com/cosmos/ibc-go/pull/567) Bump SDK version to v0.44.4

### Improvements

* (02-client) [\#568](https://github.com/cosmos/ibc-go/pull/568) In IBC `transfer` cli command use local clock time as reference for relative timestamp timeout if greater than the block timestamp queried from the latest consensus state corresponding to the counterparty channel.
* [\#583](https://github.com/cosmos/ibc-go/pull/583) Move third_party/proto/confio/proofs.proto to third_party/proto/proofs.proto to enable proto service reflection. Migrate `buf` from v1beta1 to v1.

### Bug Fixes

* (02-client) [\#500](https://github.com/cosmos/ibc-go/pull/500) Fix IBC `update-client proposal` cli command to expect correct number of args.

## [v2.0.0](https://github.com/cosmos/ibc-go/releases/tag/v2.0.0) - 2021-11-09

### Dependencies

* [\#489](https://github.com/cosmos/ibc-go/pull/489) Bump Tendermint to v0.34.14
* [\#503](https://github.com/cosmos/ibc-go/pull/503) Bump SDK version to v0.44.3

### API Breaking

* (core) [\#227](https://github.com/cosmos/ibc-go/pull/227) Remove sdk.Result from application callbacks
* (transfer) [\#350](https://github.com/cosmos/ibc-go/pull/350) Change FungibleTokenPacketData to use a string for the Amount field. This enables token transfers with amounts previously restricted by uint64. Up to the maximum uint256 value is supported.

### Features

* [\#384](https://github.com/cosmos/ibc-go/pull/384) Added `NegotiateAppVersion` method to `IBCModule` interface supported by a gRPC query service in `05-port`. This provides routing of requests to the desired application module callback, which in turn performs application version negotiation.

## [v1.5.0](https://github.com/cosmos/ibc-go/releases/tag/v1.5.0) - 2022-06-14

### Dependencies

* [\#404](https://github.com/cosmos/ibc-go/pull/404) Bump Go version to 1.17
* [\#1300](https://github.com/cosmos/ibc-go/pull/1300) Bump SDK version to v0.45.4

### Improvements

* (transfer) [\#1342](https://github.com/cosmos/ibc-go/pull/1342) `DenomTrace` grpc now takes in either an `ibc denom` or a `hash` instead of only accepting a `hash`.
* (modules/core/04-channel) [\#1160](https://github.com/cosmos/ibc-go/pull/1160) Improve `uint64 -> string` performance in `Logger`.
* (modules/core/keeper) [\#1284](https://github.com/cosmos/ibc-go/pull/1284) Add sanity check for the keepers passed into `ibckeeper.NewKeeper`. `ibckeeper.NewKeeper` now panics if any of the keepers passed in is empty.
* (transfer) [\#1414](https://github.com/cosmos/ibc-go/pull/1414) Emitting Sender address from `fungible_token_packet` events in `OnRecvPacket` and `OnAcknowledgementPacket`.
* (modules/core/04-channel) [\#1464](https://github.com/cosmos/ibc-go/pull/1464) Emit a channel close event when an ordered channel is closed.
* (modules/light-clients/07-tendermint) [\#1118](https://github.com/cosmos/ibc-go/pull/1118) Deprecating `AllowUpdateAfterExpiry` and `AllowUpdateAfterMisbehaviour`. See ADR-026 for context.

### Features

* (modules/core/02-client) [\#1336](https://github.com/cosmos/ibc-go/pull/1336) Adding Query/ConsensusStateHeights gRPC for fetching the height of every consensus state associated with a client.
* (modules/apps/transfer) [\#1416](https://github.com/cosmos/ibc-go/pull/1416) Adding gRPC endpoint for getting an escrow account for a given port-id and channel-id.

### Bug Fixes

* (modules/core/04-channel) [\#1130](https://github.com/cosmos/ibc-go/pull/1130) Call `packet.GetSequence()` rather than passing func in `WriteAcknowledgement` log output
* (apps/transfer) [\#1451](https://github.com/cosmos/ibc-go/pull/1451) Fixing the support for base denoms that contain slashes.

## [v1.4.1](https://github.com/cosmos/ibc-go/releases/tag/v1.4.1) - 2022-06-14

### Improvements

* (transfer) [\#1342](https://github.com/cosmos/ibc-go/pull/1342) `DenomTrace` grpc now takes in either an `ibc denom` or a `hash` instead of only accepting a `hash`.
* (modules/core/04-channel) [\#1160](https://github.com/cosmos/ibc-go/pull/1160) Improve `uint64 -> string` performance in `Logger`.
* (modules/core/keeper) [\#1284](https://github.com/cosmos/ibc-go/pull/1284) Add sanity check for the keepers passed into `ibckeeper.NewKeeper`. `ibckeeper.NewKeeper` now panics if any of the keepers passed in is empty.
* (transfer) [\#1414](https://github.com/cosmos/ibc-go/pull/1414) Emitting Sender address from `fungible_token_packet` events in `OnRecvPacket` and `OnAcknowledgementPacket`.
* (modules/core/04-channel) [\#1464](https://github.com/cosmos/ibc-go/pull/1464) Emit a channel close event when an ordered channel is closed.

### Bug Fixes

* (modules/core/04-channel) [\#1130](https://github.com/cosmos/ibc-go/pull/1130) Call `packet.GetSequence()` rather than passing func in `WriteAcknowledgement` log output

## [v1.4.0](https://github.com/cosmos/ibc-go/releases/tag/v1.4.0) - 2022-03-15

### Dependencies

* [\#851](https://github.com/cosmos/ibc-go/pull/851) Bump SDK version to v0.45.1

## [v1.3.1](https://github.com/cosmos/ibc-go/releases/tag/v1.3.1) - 2022-06-14

### Dependencies

* [\#1267](https://github.com/cosmos/ibc-go/pull/1267) Bump SDK version to v0.44.8 and Tendermint to version 0.34.19

### Improvements

* (transfer) [\#1342](https://github.com/cosmos/ibc-go/pull/1342) `DenomTrace` grpc now takes in either an `ibc denom` or a `hash` instead of only accepting a `hash`.
* (modules/core/04-channel) [\#1160](https://github.com/cosmos/ibc-go/pull/1160) Improve `uint64 -> string` performance in `Logger`.
* (modules/core/keeper) [\#1284](https://github.com/cosmos/ibc-go/pull/1284) Add sanity check for the keepers passed into `ibckeeper.NewKeeper`. `ibckeeper.NewKeeper` now panics if any of the keepers passed in is empty.
* (transfer) [\#1414](https://github.com/cosmos/ibc-go/pull/1414) Emitting Sender address from `fungible_token_packet` events in `OnRecvPacket` and `OnAcknowledgementPacket`.
* (modules/core/04-channel) [\#1464](https://github.com/cosmos/ibc-go/pull/1464) Emit a channel close event when an ordered channel is closed.

### Bug Fixes

* (modules/core/04-channel) [\#1130](https://github.com/cosmos/ibc-go/pull/1130) Call `packet.GetSequence()` rather than passing func in `WriteAcknowledgement` log output

## [v1.3.0](https://github.com/cosmos/ibc-go/releases/tag/v1.3.0) - 2022-03-15

### Dependencies

* [\#1073](https://github.com/cosmos/ibc-go/pull/1073) Bump SDK version to v0.44.6
* [\#948](https://github.com/cosmos/ibc-go/pull/948) Bump ics23/go to v0.7

### State Machine Breaking

* (transfer) [\#818](https://github.com/cosmos/ibc-go/pull/818) Error acknowledgements returned from Transfer `OnRecvPacket` now include a deterministic ABCI code and error message.

### Features

* [\#679](https://github.com/cosmos/ibc-go/pull/679) New CLI command `query ibc-transfer denom-hash <denom trace>` to get the denom hash for a denom trace; this might be useful for debug

### Bug Fixes

* (client) [\#941](https://github.com/cosmos/ibc-go/pull/941) Classify client states without consensus states as expired
* (transfer) [\#978](https://github.com/cosmos/ibc-go/pull/978) Support base denoms with slashes in denom validation
* (channel) [\#995](https://github.com/cosmos/ibc-go/pull/995) Call `packet.GetSequence()` rather than passing func in `AcknowledgePacket` log output

## [v1.2.6](https://github.com/cosmos/ibc-go/releases/tag/v1.2.6) - 2022-02-03

### Improvements

* (channel) [\#692](https://github.com/cosmos/ibc-go/pull/692) Minimize channel logging by only emitting the packet sequence, source port/channel, destination port/channel upon packet receives, acknowledgements and timeouts.

## [v1.2.5](https://github.com/cosmos/ibc-go/releases/tag/v1.2.5) - 2021-12-15

### Dependencies

* [\#589](https://github.com/cosmos/ibc-go/pull/589) Bump SDK version to v0.44.5

### Bug Fixes

* (modules/core) [\#603](https://github.com/cosmos/ibc-go/pull/603) Fix module name emitted as part of `OnChanOpenInit` event. Replacing `connection` module name with `channel`.

## [v1.2.4](https://github.com/cosmos/ibc-go/releases/tag/v1.2.4) - 2021-12-05

### Dependencies

* [\#567](https://github.com/cosmos/ibc-go/pull/567) Bump SDK version to v0.44.4

### Improvements

* [\#583](https://github.com/cosmos/ibc-go/pull/583) Move third_party/proto/confio/proofs.proto to third_party/proto/proofs.proto to enable proto service reflection. Migrate `buf` from v1beta1 to v1.

## [v1.2.3](https://github.com/cosmos/ibc-go/releases/tag/v1.2.3) - 2021-11-09

### Dependencies

* [\#489](https://github.com/cosmos/ibc-go/pull/489) Bump Tendermint to v0.34.14
* [\#503](https://github.com/cosmos/ibc-go/pull/503) Bump SDK version to v0.44.3

## [v1.2.2](https://github.com/cosmos/ibc-go/releases/tag/v1.2.2) - 2021-10-15

### Dependencies

* [\#485](https://github.com/cosmos/ibc-go/pull/485) Bump SDK version to v0.44.2

## [v1.2.1](https://github.com/cosmos/ibc-go/releases/tag/v1.2.1) - 2021-10-04

### Dependencies

* [\#455](https://github.com/cosmos/ibc-go/pull/455) Bump SDK version to v0.44.1

## [v1.2.0](https://github.com/cosmos/ibc-go/releases/tag/v1.2.0) - 2021-09-10

### State Machine Breaking

* (24-host) [\#344](https://github.com/cosmos/ibc-go/pull/344) Increase port identifier limit to 128 characters.

### Improvements

* [\#373](https://github.com/cosmos/ibc-go/pull/375) Added optional field `PacketCommitmentSequences` to `QueryPacketAcknowledgementsRequest` to provide filtering of packet acknowledgements.

### Features

* [\#372](https://github.com/cosmos/ibc-go/pull/372) New CLI command `query ibc client status <client id>` to get the current activity status of a client.

### Dependencies

* [\#386](https://github.com/cosmos/ibc-go/pull/386) Bump [tendermint](https://github.com/tendermint/tendermint) from v0.34.12 to v0.34.13.

## [v1.1.6](https://github.com/cosmos/ibc-go/releases/tag/v1.1.6) - 2022-01-25

### Improvements

* (channel) [\#692](https://github.com/cosmos/ibc-go/pull/692) Minimize channel logging by only emitting the packet sequence, source port/channel, destination port/channel upon packet receives, acknowledgements and timeouts.

## [v1.1.5](https://github.com/cosmos/ibc-go/releases/tag/v1.1.5) - 2021-12-15

### Dependencies

* [\#589](https://github.com/cosmos/ibc-go/pull/589) Bump SDK version to v0.44.5

### Bug Fixes

* (modules/core) [\#603](https://github.com/cosmos/ibc-go/pull/603) Fix module name emitted as part of `OnChanOpenInit` event. Replacing `connection` module name with `channel`.

## [v1.1.4](https://github.com/cosmos/ibc-go/releases/tag/v1.1.4) - 2021-12-05

### Dependencies

* [\#567](https://github.com/cosmos/ibc-go/pull/567) Bump SDK version to v0.44.4

### Improvements

* [\#583](https://github.com/cosmos/ibc-go/pull/583) Move third_party/proto/confio/proofs.proto to third_party/proto/proofs.proto to enable proto service reflection. Migrate `buf` from v1beta1 to v1.

## [v1.1.3](https://github.com/cosmos/ibc-go/releases/tag/v1.1.3) - 2021-11-09

### Dependencies

* [\#489](https://github.com/cosmos/ibc-go/pull/489) Bump Tendermint to v0.34.14
* [\#503](https://github.com/cosmos/ibc-go/pull/503) Bump SDK version to v0.44.3

## [v1.1.2](https://github.com/cosmos/ibc-go/releases/tag/v1.1.2) - 2021-10-15

* [\#485](https://github.com/cosmos/ibc-go/pull/485) Bump SDK version to v0.44.2

## [v1.1.1](https://github.com/cosmos/ibc-go/releases/tag/v1.1.1) - 2021-10-04

### Dependencies

* [\#455](https://github.com/cosmos/ibc-go/pull/455) Bump SDK version to v0.44.1

## [v1.1.0](https://github.com/cosmos/ibc-go/releases/tag/v1.1.0) - 2021-09-03

### Dependencies

* [\#367](https://github.com/cosmos/ibc-go/pull/367) Bump [cosmos-sdk](https://github.com/cosmos/cosmos-sdk) from 0.43 to 0.44.

## [v1.0.1](https://github.com/cosmos/ibc-go/releases/tag/v1.0.1) - 2021-08-25

### Improvements

* [\#343](https://github.com/cosmos/ibc-go/pull/343) Create helper functions for publishing of packet sent and acknowledgement sent events.

## [v1.0.0](https://github.com/cosmos/ibc-go/releases/tag/v1.0.0) - 2021-08-10

### Bug Fixes

* (07-tendermint) [\#241](https://github.com/cosmos/ibc-go/pull/241) Ensure tendermint client state latest height revision number matches chain id revision number.
* (07-tendermint) [\#234](https://github.com/cosmos/ibc-go/pull/234) Use sentinel value for the consensus state root set during a client upgrade. This prevents genesis validation from failing.
* (modules) [\#223](https://github.com/cosmos/ibc-go/pull/223) Use correct Prometheus format for metric labels.
* (06-solomachine) [\#214](https://github.com/cosmos/ibc-go/pull/214) Disable defensive timestamp check in SendPacket for solo machine clients.
* (07-tendermint) [\#210](https://github.com/cosmos/ibc-go/pull/210) Export all consensus metadata on genesis restarts for tendermint clients.
* (core) [\#200](https://github.com/cosmos/ibc-go/pull/200) Fixes incorrect export of IBC identifier sequences. Previously, the next identifier sequence for clients/connections/channels was not set during genesis export. This resulted in the next identifiers being generated on the new chain to reuse old identifiers (the sequences began again from 0).
* (02-client) [\#192](https://github.com/cosmos/ibc-go/pull/192) Fix IBC `query ibc client header` cli command. Support historical queries for query header/node-state commands.
* (modules/light-clients/06-solomachine) [\#153](https://github.com/cosmos/ibc-go/pull/153) Fix solo machine proof height sequence mismatch bug.
* (modules/light-clients/06-solomachine) [\#122](https://github.com/cosmos/ibc-go/pull/122) Fix solo machine merkle prefix casting bug.
* (modules/light-clients/06-solomachine) [\#120](https://github.com/cosmos/ibc-go/pull/120) Fix solo machine handshake verification bug.
* (modules/light-clients/06-solomachine) [\#153](https://github.com/cosmos/ibc-go/pull/153) fix solo machine connection handshake failure at `ConnectionOpenAck`.

### API Breaking

* (04-channel) [\#220](https://github.com/cosmos/ibc-go/pull/220) Channel legacy handler functions were removed. Please use the MsgServer functions or directly call the channel keeper's handshake function.
* (modules) [\#206](https://github.com/cosmos/ibc-go/pull/206) Expose `relayer sdk.AccAddress` on `OnRecvPacket`, `OnAcknowledgementPacket`, `OnTimeoutPacket` module callbacks to enable incentivization.
* (02-client) [\#181](https://github.com/cosmos/ibc-go/pull/181) Remove 'InitialHeight' from UpdateClient Proposal. Only copy over latest consensus state from substitute client.
* (06-solomachine) [\#169](https://github.com/cosmos/ibc-go/pull/169) Change FrozenSequence to boolean in solomachine ClientState. The solo machine proto package has been bumped from `v1` to `v2`.
* (module/core/02-client) [\#165](https://github.com/cosmos/ibc-go/pull/165) Remove GetFrozenHeight from the ClientState interface.
* (modules) [\#166](https://github.com/cosmos/ibc-go/pull/166) Remove GetHeight from the misbehaviour interface. The `consensus_height` attribute has been removed from Misbehaviour events.
* (modules) [\#162](https://github.com/cosmos/ibc-go/pull/162) Remove deprecated Handler types in core IBC and the ICS 20 transfer module.
* (modules/core) [\#161](https://github.com/cosmos/ibc-go/pull/161) Remove Type(), Route(), GetSignBytes() from 02-client, 03-connection, and 04-channel messages.
* (modules) [\#140](https://github.com/cosmos/ibc-go/pull/140) IsFrozen() client state interface changed to Status(). gRPC `ClientStatus` route added.
* (modules/core) [\#109](https://github.com/cosmos/ibc-go/pull/109) Remove connection and channel handshake CLI commands.
* (modules) [\#107](https://github.com/cosmos/ibc-go/pull/107) Modify OnRecvPacket callback to return an acknowledgement which indicates if it is successful or not. Callback state changes are discarded for unsuccessful acknowledgements only.
* (modules) [\#108](https://github.com/cosmos/ibc-go/pull/108) All message constructors take the signer as a string to prevent upstream bugs. The `String()` function for an SDK Acc Address relies on external context.
* (transfer) [\#275](https://github.com/cosmos/ibc-go/pull/275) Remove 'ChanCloseInit' function from transfer keeper. ICS20 does not close channels.

### State Machine Breaking

* (modules/light-clients/07-tendermint) [\#99](https://github.com/cosmos/ibc-go/pull/99) Enforce maximum chain-id length for tendermint client.
* (modules/light-clients/07-tendermint) [\#141](https://github.com/cosmos/ibc-go/pull/141) Allow a new form of misbehaviour that proves counterparty chain breaks time monotonicity, automatically enforce monotonicity in UpdateClient and freeze client if monotonicity is broken.
* (modules/light-clients/07-tendermint) [\#141](https://github.com/cosmos/ibc-go/pull/141) Freeze the client if there's a conflicting header submitted for an existing consensus state.
* (modules/core/02-client) [\#8405](https://github.com/cosmos/cosmos-sdk/pull/8405) Refactor IBC client update governance proposals to use a substitute client to update a frozen or expired client.
* (modules/core/02-client) [\#8673](https://github.com/cosmos/cosmos-sdk/pull/8673) IBC upgrade logic moved to 02-client and an IBC UpgradeProposal is added.
* (modules/core/03-connection) [\#171](https://github.com/cosmos/ibc-go/pull/171) Introduces a new parameter `MaxExpectedTimePerBlock` to allow connections to calculate and enforce a block delay that is proportional to time delay set by connection.
* (core) [\#268](https://github.com/cosmos/ibc-go/pull/268) Perform a no-op on redundant relay messages. Previous behaviour returned an error. Now no state change will occur and no error will be returned.

### Improvements

* (04-channel) [\#220](https://github.com/cosmos/ibc-go/pull/220) Channel handshake events are now emitted with the channel keeper.
* (core/02-client) [\#205](https://github.com/cosmos/ibc-go/pull/205) Add in-place and genesis migrations from SDK v0.42.0 to ibc-go v1.0.0. Solo machine protobuf defintions are migrated from v1 to v2. All solo machine consensus states are pruned. All expired tendermint consensus states are pruned.
* (modules/core) [\#184](https://github.com/cosmos/ibc-go/pull/184) Improve error messages. Uses unique error codes to indicate already relayed packets.
* (07-tendermint) [\#182](https://github.com/cosmos/ibc-go/pull/182) Remove duplicate checks in upgrade logic.
* (modules/core/04-channel) [\#7949](https://github.com/cosmos/cosmos-sdk/issues/7949) Standardized channel `Acknowledgement` moved to its own file. Codec registration redundancy removed.
* (modules/core/04-channel) [\#144](https://github.com/cosmos/ibc-go/pull/144) Introduced a `packet_data_hex` attribute to emit the hex-encoded packet data in events. This allows for raw binary (proto-encoded message) to be sent over events and decoded correctly on relayer. Original `packet_data` is DEPRECATED. All relayers and IBC event consumers are encouraged to switch to `packet_data_hex` as soon as possible.
* (core/04-channel) [\#197](https://github.com/cosmos/ibc-go/pull/197) Introduced a `packet_ack_hex` attribute to emit the hex-encoded acknowledgement in events. This allows for raw binary (proto-encoded message) to be sent over events and decoded correctly on relayer. Original `packet_ack` is DEPRECATED. All relayers and IBC event consumers are encouraged to switch to `packet_ack_hex` as soon as possible.
* (modules/light-clients/07-tendermint) [\#125](https://github.com/cosmos/ibc-go/pull/125) Implement efficient iteration of consensus states and pruning of earliest expired consensus state on UpdateClient.
* (modules/light-clients/07-tendermint) [\#141](https://github.com/cosmos/ibc-go/pull/141) Return early in case there's a duplicate update call to save Gas.
* (modules/core/ante) [\#235](https://github.com/cosmos/ibc-go/pull/235) Introduces a new IBC Antedecorator that will reject transactions that only contain redundant packet messages (and accompany UpdateClient msgs). This will prevent relayers from wasting fees by submitting messages for packets that have already been processed by previous relayer(s). The Antedecorator is only applied on CheckTx and RecheckTx and is therefore optional for each node.

### Features

* [\#198](https://github.com/cosmos/ibc-go/pull/198) New CLI command `query ibc-transfer escrow-address <port> <channel id>` to get the escrow address for a channel; can be used to then query balance of escrowed tokens

### Client Breaking Changes

* (02-client/cli) [\#196](https://github.com/cosmos/ibc-go/pull/196) Rename `node-state` cli command to `self-consensus-state`.

## IBC in the Cosmos SDK Repository

The IBC module was originally released in [v0.40.0](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.40.0) of the SDK.
Please see the [Release Notes](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/RELEASE_NOTES.md).

The IBC module is also contained in the releases for [v0.41.x](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.41.0) and [v0.42.x](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.42.0).
Please see the Release Notes for [v0.41.x](https://github.com/cosmos/cosmos-sdk/blob/v0.41.0/RELEASE_NOTES.md) and [v0.42.x](https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/RELEASE_NOTES.md).

The IBC module was removed in the commit hash [da064e13d56add466548135739c5860a9f7ed842](https://github.com/cosmos/cosmos-sdk/commit/da064e13d56add466548135739c5860a9f7ed842) on the SDK. The release for SDK v0.43.0 will be the first release without the IBC module.

Backports should be made to the [release/v0.42.x](https://github.com/cosmos/cosmos-sdk/tree/release/v0.42.x) branch on the SDK.
