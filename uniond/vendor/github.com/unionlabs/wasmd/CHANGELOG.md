# Changelog

## [Unreleased](https://github.com/CosmWasm/wasmd/tree/HEAD)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.53.0...HEAD)

## [v0.53.0](https://github.com/CosmWasm/wasmd/tree/v0.53.0) (2024-08-21)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.52.0...v0.53.0)

- Fix CWA-2024-005
- Fix CWA-2024-006
- Fix submit-proposal instantiate-contract-2 command [\#1934](https://github.com/CosmWasm/wasmd/pull/1934)

## [v0.52.0](https://github.com/CosmWasm/wasmd/tree/v0.52.0) (2024-07-11)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.51.0...v0.52.0)

- Update wasmvm to 2.1 release [#1927](https://github.com/CosmWasm/wasmd/pull/1927)
- Validate number of addresses in msg [#1926](https://github.com/CosmWasm/wasmd/pull/1926)
- Add `cosmwasm_2_1` capability [#1925](https://github.com/CosmWasm/wasmd/pull/1925)
- Migrate Version checks [#1924](https://github.com/CosmWasm/wasmd/pull/1924)
- Pinned Metrics [#1922](https://github.com/CosmWasm/wasmd/pull/1922)
- Mark wasm queries with module_query_safe [#1915](https://github.com/CosmWasm/wasmd/pull/1915)
- Add codespace to error acknowledgement [#1911](https://github.com/CosmWasm/wasmd/pull/1911)
- Upgrade cosmos-sdk to v0.50.7 [#1905](https://github.com/CosmWasm/wasmd/pull/1905)
- Mark QuerySmart contract error as deterministic [#1904](https://github.com/CosmWasm/wasmd/pull/1904)
- Async Ack [#1876](https://github.com/CosmWasm/wasmd/pull/1876)
- IBC Callbacks [#1817](https://github.com/CosmWasm/wasmd/pull/1817)

### Notable changes:

- Upgrade to [CosmWasm 2.1.0](https://github.com/CosmWasm/wasmvm/releases/tag/v2.1.0)

### Migration notes:

- This release does not include any state migrations but breaking changes that require a coordinated chain upgrade.

# [v0.51.0](https://github.com/CosmWasm/wasmd/tree/v0.51.0) (2024-04-22)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.50.0...v0.51.0)

- Bump cosmos-sdk to v0.50.6 [#1865](https://github.com/CosmWasm/wasmd/pull/1865)
- Bump tx module to v0.13.3 [#1864](https://github.com/CosmWasm/wasmd/pull/1864)
- Fix Amino JSON representation [\#1844](https://github.com/CosmWasm/wasmd/pull/1844)
- Add BuiltInCapabilities() to x/wasm/keeper and deprecate AllCapabilities() [#1855](https://github.com/CosmWasm/wasmd/pull/1855)
- Move contrib/ in scripts/ [\#1848](https://github.com/CosmWasm/wasmd/pull/1848)
- Remove contract_tests [\#1847](https://github.com/CosmWasm/wasmd/pull/1847)
- Upgrade Alpine in Dockerfile to 3.18 [\#1837](https://github.com/CosmWasm/wasmd/pull/1837)
- Bump wasmvm to 2.0.0 [\#1832](https://github.com/CosmWasm/wasmd/pull/1832)
- Rename gas to gasLeft [\#1820](https://github.com/CosmWasm/wasmd/pull/1820)
- Fix misleading error message for duplicate address on instantiate2 [\#1816](https://github.com/CosmWasm/wasmd/pull/1816)
- Rewrite compatibility section in README [\#1805](https://github.com/CosmWasm/wasmd/pull/1805)
- Generalize "pinned" to "discount" for cases where contract is in memory [\#1799](https://github.com/CosmWasm/wasmd/pull/1799)
- Rename InstantiateContractCosts -> SetupContractCost; Remove NewContractInstanceCosts [\#1797](https://github.com/CosmWasm/wasmd/pull/1797)
- Bump cosmos-sdk proto to v0.50.0 [\#1795](https://github.com/CosmWasm/wasmd/pull/1795)
- Fix grant contract cli command description [\#1778](https://github.com/CosmWasm/wasmd/issues/1778)
- Add Build Address gRPC Query [\#1753](https://github.com/CosmWasm/wasmd/pull/1753)

### Notable changes:

- Upgrade to [CosmWasm 2.0.0](https://github.com/CosmWasm/wasmvm/releases/tag/v2.0.0)
- Upgrade to [SDK v0.50.6](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.50.6) patch release of Cosmos SDK Eden

### Migration notes:

- This release does not include any state migrations but breaking changes that require a coordinated chain upgrade.

## [v0.50.0](https://github.com/CosmWasm/wasmd/tree/v0.50.0) (2023-11-23)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.45.0...v0.50.0)

- Fix module migration fails when code upload access is set to `OnlyAddress` [\#1725](https://github.com/CosmWasm/wasmd/issues/1725)
- Upgrade to SDK v0.50.1 [\#1696](https://github.com/CosmWasm/wasmd/pull/1696)
- Bump bufbuild/buf-setup-action from 1.27.1 to 1.27.2 [\#1684](https://github.com/CosmWasm/wasmd/pull/1684)
- Bump google.golang.org/grpc from 1.58.1 to 1.58.3 in /tests/system [\#1682](https://github.com/CosmWasm/wasmd/pull/1682)
- Rename some more "features" to "capabilities" [\#1679](https://github.com/CosmWasm/wasmd/pull/1679)
- Bump actions/checkout from 4.1.0 to 4.1.1 [\#1678](https://github.com/CosmWasm/wasmd/pull/1678)
- Bump bufbuild/buf-setup-action from 1.27.0 to 1.27.1 [\#1677](https://github.com/CosmWasm/wasmd/pull/1677)
- Upgrade to SDK v0.50 + IBC v8.0 [\#1611](https://github.com/CosmWasm/wasmd/pull/1611)

### Notable changes:

- Upgrade to [SDK v0.50.1](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.50.1) Eden release
- Upgrade to [IBC v8.0.0](https://github.com/cosmos/ibc-go/releases/tag/v8.0.0) release

### Migration notes:

- This release includes state migrations! Please pay careful attention to the doc provided by the [SDK](https://github.com/cosmos/cosmos-sdk/blob/v0.50.1/UPGRADING.md#v050x) team
  and the [IBC](https://github.com/cosmos/ibc-go/releases/tag/v8.0.0) team with their releases
- We have some example upgrade code in the `app/upgrade/v0.50` dir, assuming that you come from a wasmd v0.4x version.
- We also tested a chain upgrade with wasmd v0.33 to v0.50 directly with a fix to [app.go](https://github.com/CosmWasm/wasmd/blame/b02a4723618629b5bb9603d8298621f6ef449f92/app/app.go#L927)
  to work around an issue.
- Disclaimer: the upgrade codes are examples and may require more customization for your chain. Please do proper tests before applying on mainnets
- Please share your experience and success stories on CosmWasm [discord](https://discord.com/invite/cPjEnPd)

## [v0.45.0](https://github.com/CosmWasm/wasmd/tree/v0.45.0) (2023-11-15)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.44.0...v0.45.0)

- Retract wasmd v0.43.0 and v0.44.0 [\#1720](https://github.com/CosmWasm/wasmd/pull/1720)
- Allow empty attribute values [\#1618](https://github.com/CosmWasm/wasmd/pull/1618)
- Fix gov v1beta1 support for legacy proposals [\#1715](https://github.com/CosmWasm/wasmd/pull/1715)
- Prevent empty channel version [\#1697](https://github.com/CosmWasm/wasmd/pull/1697)

### Notable changes:

- Wasmd v0.43.0 and 0.44.0 releases were retracted because of a bug related to legacy proposals. See https://github.com/CosmWasm/wasmd/issues/1713 for more details.

### Migration notes:

- This release does not include any state migrations but breaking changes that require a coordinated chain upgrade.

## [v0.44.0](https://github.com/CosmWasm/wasmd/tree/v0.44.0) (2023-11-06)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.43.0...v0.44.0)

- Upgrade to wasmvm v1.5.0 [\#1675](https://github.com/CosmWasm/wasmd/issues/1675)
- Prevent non printable characters in label [\#1623](https://github.com/CosmWasm/wasmd/issues/1623)

### Notable changes:

- Upgrade to wasmvm [v1.5.0](https://github.com/CosmWasm/wasmvm/releases/tag/v1.5.0)

### Migration notes:

- This release does not include any state migrations but breaking changes that require a coordinated chain upgrade.

## [v0.43.0](https://github.com/CosmWasm/wasmd/tree/v0.43.0) (2023-10-10)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.42.0...v0.43.0)

- Make contract label updatable [\#1601](https://github.com/CosmWasm/wasmd/issues/1601)
- Remove gov v1beta1 dependencies and deprecated proposals [\#1608](https://github.com/CosmWasm/wasmd/issues/1608)
- Retract v0.42.0 release [\#1651](https://github.com/CosmWasm/wasmd/issues/1651)
- Bump wasmvm to v1.4.1 [\#1657](https://github.com/CosmWasm/wasmd/issues/1657)

### Notable changes:

- Wasmd v0.42.0 was retracted because an issue was found in CosmWasm 1.4.0. See https://twitter.com/CosmWasm/status/1709507168448229497
- This release bundles CosmWasm v1.4.1 patch release. If you are using wasmd v0.42.0 please upgrade to wasmd 0.43.0 as soon as possible.
- Deprecated wasm gov proposals have been removed completely from the codebase together with all the govv1beta1 dependencies.
- MsgUpdateContractLabel was introduced to allow contract label updates.

### Migration notes:

- This release does not include any state migrations but breaking changes that require a coordinated chain upgrade.

## [v0.42.0](https://github.com/CosmWasm/wasmd/tree/v0.42.0) (2023-09-20)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.41.0...v0.42.0)

- Fix label validation error [\#1555](https://github.com/CosmWasm/wasmd/pull/1555)
- Improve ToWasmVMGas/FromWasmVMGas code level documentation [\#1564](https://github.com/CosmWasm/wasmd/pull/1564)
- Fix gas calculation [\#1567](https://github.com/CosmWasm/wasmd/pull/1567)
- Bump cosmos-sdk to v0.47.5 [\#1467](https://github.com/CosmWasm/wasmd/issues/1467)
- fix: make sure wasmvm cache metrics collector registration after VM setup[\#1575](https://github.com/CosmWasm/wasmd/pull/1575)
- Remove legacy gov proposal dependencies [\#1587](https://github.com/CosmWasm/wasmd/pull/1587)
- Upgrade to wasmvm 1.4 [\#1586](https://github.com/CosmWasm/wasmd/issues/1586)
- Custom StoreCode Authorization for authz module [\#1584](https://github.com/CosmWasm/wasmd/issues/1584)
- Bump github.com/cosmos/ibc-go/v7 from 7.2.0 to 7.3.0 [\#1594](https://github.com/CosmWasm/wasmd/pull/1594)
- Log query error before redacting [\#1593](https://github.com/CosmWasm/wasmd/issues/1593)
- Restrict pagination on all-state-query [\#1619](https://github.com/CosmWasm/wasmd/pull/1619)
- Bug in IbcQuery::ListChannels implementation when port is unset [\#1597](https://github.com/CosmWasm/wasmd/issues/1597)
- Ensure some contraints and limits on pin/unpin code ids [\#1624](https://github.com/CosmWasm/wasmd/pull/1624)
- Ensure genesis import works with both address generators [\#1629](https://github.com/CosmWasm/wasmd/issues/1629)
- Set default query limit and ensure constraints [\#1632](https://github.com/CosmWasm/wasmd/pull/1632)

### Notable changes:

- Fix gas calculation [\#1567](https://github.com/CosmWasm/wasmd/pull/1567)
- Upgrade to wasmvm 1.4 [\#1586](https://github.com/CosmWasm/wasmd/issues/1586)
- Bug in IbcQuery::ListChannels implementation when port is unset [\#1597](https://github.com/CosmWasm/wasmd/issues/1597)
  - If `port_id` is omitted, all channels bound to the contract's port will be listed.
- Restrict pagination on all-state-query [\#1619](https://github.com/CosmWasm/wasmd/pull/1619)
  - Pagination limit is set to 100 for all-state-query. See also [\#1632](https://github.com/CosmWasm/wasmd/pull/1632)
- Ensure some contraints and limits on pin/unpin code ids [\#1624](https://github.com/CosmWasm/wasmd/pull/1624)
  - Total number of code ids is limited to 50 for pin/unpin operations
- Custom StoreCode Authorization for authz module [\#1584](https://github.com/CosmWasm/wasmd/issues/1584)

### Migration notes:

- This release does not include any state migrations but breaking changes that require a coordinated chain upgrade.

## [v0.41.0](https://github.com/CosmWasm/wasmd/tree/v0.41.0) (2023-07-28)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.40.2...v0.41.0)

- Bump IBC-Go to 7.2.0 [\#1459](https://github.com/CosmWasm/wasmd/pull/1459)
- Ensure history position does not conflict [\#1462](https://github.com/CosmWasm/wasmd/pull/1462)
- Bump sequence in ibctests for failed messages [\#1464](https://github.com/CosmWasm/wasmd/pull/1464)
- Accept wasm data from state-sync as valid [\#1467](https://github.com/CosmWasm/wasmd/issues/1467)
- Add updates to ibctesting framework for MS [\#1472](https://github.com/CosmWasm/wasmd/pull/1472)
- Bump CometBFT to v0.37.2 - security fix [\#1479](https://github.com/CosmWasm/wasmd/pull/1479)
- Gov: support contracts that dynamically instantiate other contracts [\#1207](https://github.com/CosmWasm/wasmd/issues/1207)
- Deprecate alias.go [\#1484](https://github.com/CosmWasm/wasmd/pull/1484)
- Upgrade to wasmvm v1.3 [\#1296](https://github.com/CosmWasm/wasmd/issues/1296)
- System tests dependency cleanup [\#1489](https://github.com/CosmWasm/wasmd/pull/1489)
- Allow better override of wasmVM in x/wasm keeper [\#1494](https://github.com/CosmWasm/wasmd/pull/1494)
- Make system tests extendable for other app binaries [\#1503](https://github.com/CosmWasm/wasmd/pull/1503)
- Build release + artifact on CI [\#1515](https://github.com/CosmWasm/wasmd/pull/1515)
- Add/Remove addresses for code upload params [\#1493](https://github.com/CosmWasm/wasmd/pull/1493)
- Bump SDK to v0.47.4 [\#1525](https://github.com/CosmWasm/wasmd/pull/1525)

### Notable changes:

- Bump CometBFT to [v0.37.2](https://github.com/cometbft/cometbft/releases/tag/v0.37.2) because of security issues. See https://github.com/advisories/GHSA-w24w-wp77-qffm
- Support for new wasmvm v1.3.0 capabilities [\#1296](https://github.com/CosmWasm/wasmd/issues/1296)
- Deprecate alias.go [\#1484](https://github.com/CosmWasm/wasmd/pull/1484)
- Gov: support contracts that dynamically instantiate other contracts [\#1207](https://github.com/CosmWasm/wasmd/issues/1207)
- Add/Remove addresses for code upload params [\#1493](https://github.com/CosmWasm/wasmd/pull/1493)

### Migration notes:

- This release does not include any state migrations but breaking changes that require a coordinated chain upgrade.
- This release contains an ibc-go version bump from v7.0.1 to v7.2.0. Please read the [migration guide](https://github.com/cosmos/ibc-go/blob/v7.2.0/docs/migrations/v7-to-v7_1.md) carefully.

## [v0.40.2](https://github.com/CosmWasm/wasmd/tree/v0.40.2) (2023-06-09)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.40.1...v0.40.2)

This patch release includes only the cosmos-sdk upgrade to [v0.47.3](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.47.3).
It solves [barberry](https://forum.cosmos.network/t/cosmos-sdk-security-advisory-barberry/10825) security vulnerability.

### Notable changes:

- Upgrade cosmos-sdk to [v0.47.3](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.47.3)

### Migration notes:

- This release does not include any state migrations or breaking changes, therefore a coordinated chain upgrade is not required.

## [v0.40.1](https://github.com/CosmWasm/wasmd/tree/v0.40.1) (2023-06-06)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.40.0...v0.40.1)

This patch release includes only the wasmvm upgrade to [v1.2.4](https://github.com/CosmWasm/wasmvm/releases/tag/v1.2.4).
It solves some incompatibility problems by improving cache invalidation. See [\#426](https://github.com/CosmWasm/wasmvm/issues/426)

### Notable changes:

- Upgrade wasmvm to v1.2.4. [\#1430](https://github.com/CosmWasm/wasmd/issues/1430)

### Migration notes:

- This release does not include any state migrations or breaking changes, therefore a coordinated chain upgrade is not required.

## [v0.40.0](https://github.com/CosmWasm/wasmd/tree/v0.40.0) (2023-05-25)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.32.0...v0.40.0)

Wasmd 0.40 has a large dependency upgrade of the Cosmos SDK version from 0.45 to 0.47. Please read notable changes and migration notes
below to learn more!

- Bump IBC-Go to v7.0.1 to include the fix for the huckleberry security advisory.[\#1418](https://github.com/CosmWasm/wasmd/pull/1418)
- Fix cli update-instantiate-config command [/#1415](https://github.com/CosmWasm/wasmd/pull/1415)
- Import export simulation test for `x/wasm` is missing [\#1372](https://github.com/CosmWasm/wasmd/issues/1372)
- Better tracking of CosmWasm capabilities [\#1341](https://github.com/CosmWasm/wasmd/issues/1341)
- Rename `lastIDKey` key [\#1182](https://github.com/CosmWasm/wasmd/issues/1182)
- Use ICS4Wrapper to send raw IBC packets & fix Fee middleware in wasm stack \(backport \#1375\) [\#1379](https://github.com/CosmWasm/wasmd/pull/1379)
- Add wasm store to import-export sims [\#1374](https://github.com/CosmWasm/wasmd/pull/1374)
- Bumped SDK to 0.47.2 and CometBFT to 0.37.1 [\#1369](https://github.com/CosmWasm/wasmd/pull/1369)
- Remove starport config [\#1359](https://github.com/CosmWasm/wasmd/pull/1359)
- Proper v1 gov support for wasm msg types [\#1301](https://github.com/CosmWasm/wasmd/issues/1301)
- Cleanup `ErrNotFound` cases [\#1258](https://github.com/CosmWasm/wasmd/issues/1258)
- New proto annotations [\#1157](https://github.com/CosmWasm/wasmd/issues/1157)
- Simulations with '--dry-run' return an error [\#713](https://github.com/CosmWasm/wasmd/issues/713)
- Add wasmvm decorator option [\#1348](https://github.com/CosmWasm/wasmd/pull/1348)
- More verbose error message [\#1354](https://github.com/CosmWasm/wasmd/pull/1354)
- Remove gogo/protobuf from the 47 build's dependencies [\#1281](https://github.com/CosmWasm/wasmd/issues/1281)
- Set final ibc-go version [\#1271](https://github.com/CosmWasm/wasmd/issues/1271)
- Upgrade to cosmos-sdk proto 0.47.x [\#1148](https://github.com/CosmWasm/wasmd/issues/1148)

### Notable changes:

- If you are not coming from v0.32.0, please see the "Notables changes" below, first. Especially about CometBFT.
- IBC-Go is a new major version including the "hucklebery" security fix. See [v7.0.1](https://github.com/cosmos/ibc-go/releases/tag/v7.0.1).
- SDK 47 support is a big step from the SDK 45 version supported before. Make sure to read the upgrade guide for the SDK
  before applying any changes. Links below.
- Some advice from working with SDK 47 that may affect you, too:
  - The SDK version includes some key store migration for the CLI. Make sure you backup your private keys before
    testing this! You can not switch back to v0.45 afaik
  - Take care that you use the goleveldb version used in the SDK. A transitive dependency may change it which caused
    failing queries on a running server: `Error: rpc error: code = InvalidArgument desc = failed to load state at height 1; version does not exist (latest height: 1): invalid request`
    Ensure this in go.mod:
    `github.com/syndtr/goleveldb => github.com/syndtr/goleveldb v1.0.1-0.20210819022825-2ae1ddf74ef7`
  - With custom modules, use the new proto-builder version (Makefile) to let proto types register with the correct registry
  - Ensure that all `ParameterChangeProposal` are completed before the upgrade or migrate them to `v1.gov`. SDK and wasm
    modules execute a migration before so that these proposals would not have an affect.
  - Attribute keys/ values in events are strings and not bytes in CometBFT. This may break clients
  - CLI: `add-genesis-account`, `gentx,add-genesis-account`, `collect-gentxs` and others are now under genesis command as parent
  - CLI: `--broadcast-mode block` was removed. You need to query the result for a TX with `wasmd q tx <hash>` instead

### Migration notes:

- This release contains a [state migration](./x/wasm/migrations/v2) for the wasmd module that stores
  the params in the module store.
- SDK v0.47 comes with a lot of api/state braking changes to previous versions. Please see their [upgrade guide](https://github.com/cosmos/cosmos-sdk/blob/main/UPGRADING.md#v047x)
  which contains a lot of helpful details.
- Please read the [migration guide](https://github.com/cosmos/ibc-go/tree/v7.0.0/docs/migrations) for IBC-Go [v7.0.0](https://github.com/cosmos/ibc-go/releases/tag/v7.0.0) carefully

## [v0.32.0](https://github.com/CosmWasm/wasmd/tree/v0.32.0) (2023-05-11)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.31.0...v0.32.0)

- Redesign IBC on packet recv error/ result.Err handling [\#1358](https://github.com/CosmWasm/wasmd/pull/1358)
- Use ICS4Wrapper to send raw IBC packets & fix Fee middleware in wasm stack [\#1375](https://github.com/CosmWasm/wasmd/pull/1375)
- Better configuration for CosmWasm capabilities [\#1361](https://github.com/CosmWasm/wasmd/pull/1361)
- Remove old starport config - unused [\#1359](https://github.com/CosmWasm/wasmd/pull/1359)
- Better error message for wasm file limit exceeded [\#1354](https://github.com/CosmWasm/wasmd/pull/1354)
- CLI param to bypass version check for wasm lib [\#1338](https://github.com/CosmWasm/wasmd/pull/1338)
- Cleanup ErrNotFound cases [\#1343](https://github.com/CosmWasm/wasmd/pull/1343)
- Add wasmvm decorator option [\#1350](https://github.com/CosmWasm/wasmd/pull/1350)
- Bump github.com/prometheus/client_golang from 1.14.0 to 1.15.0 [/#1336](https://github.com/CosmWasm/wasmd/pull/1336)
- Update OnRecvPacket method to panic when an error is returned by the VM [/#1303](https://github.com/CosmWasm/wasmd/pull/1303)
- Removed the unnecessary usage of ErrInvalidMsg [\#1317](https://github.com/CosmWasm/wasmd/pull/1317)
- Upgrade wasmvm to v1.2.3 [\#1355](https://github.com/CosmWasm/wasmd/pull/1355), see [wasmvm v1.2.3](https://github.com/CosmWasm/wasmvm/releases/tag/v1.2.3)
- Upgrade to Cosmos-SDK v0.45.15 including CometBFT [\#1284](https://github.com/CosmWasm/wasmd/pull/1284)

### Notable changes:

- New CLI param to skip checkLibwasmVersion `--wasm.skip_wasmvm_version_check`
- The wasmvm version includes the [Cherry](https://github.com/CosmWasm/advisories/blob/main/CWAs/CWA-2023-002.md) bugfix
- New behaviour for Contracts returning errors on IBC packet receive.
  - Let contract fully abort IBC receive in certain case [\#1220](https://github.com/CosmWasm/wasmd/issues/1220)
  - Return non redacted error content on IBC packet recv [\#1289](https://github.com/CosmWasm/wasmd/issues/1289)
  - Wasm and submessage events follow SDK transaction behaviour. Not persisted on state rollback
  - Full error message is stored in event [\#1288](https://github.com/CosmWasm/wasmd/issues/1288)
  - See updates in cosmwasm [doc](https://github.com/CosmWasm/cosmwasm/pull/1646/files?short_path=f9839d7#diff-f9839d73197185aaec052064f43a324bd9309413f3ad36183c3247580b1b6669) for more details.
- The SDK v0.45.15 replaces Tendermint with CometBFT. This requires a `replace` statement in `go.mod`.
  Please read their [release notes](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.45.15) carefully for details
- The SDK v0.45.x line reached its end-of-life.
- CometBFT includes some [breaking changes](https://github.com/cometbft/cometbft/blob/v0.34.27/CHANGELOG.md#breaking-changes)

### Migration notes:

- This release does not include any state migrations but breaking changes that require a coordinated chain upgrade

## [v0.31.0](https://github.com/CosmWasm/wasmd/tree/v0.31.0) (2023-03-13)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.30.0...v0.31.0)

- Upgrade wasmvm to v1.2.1 [\#1245](https://github.com/CosmWasm/wasmd/pull/1245), see [wasmvm v1.2.1](https://github.com/CosmWasm/wasmvm/releases/tag/v1.2.1)
- Fix checksum check for zipped gov store proposals [\#1232](https://github.com/CosmWasm/wasmd/issues/1232)
- Return IBC packet sequence number in the handler plugin [\#1154](https://github.com/CosmWasm/wasmd/issues/1154)
- Add Windows client-side support [\#1169](https://github.com/CosmWasm/wasmd/issues/1169)
- Upgrade Cosmos-SDK to [v0.45.14](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.45.14)
- Add missing events for admin-related methods [\#1173](https://github.com/CosmWasm/wasmd/issues/1173)
- Disallow storing new codes with ACCESS_TYPE_ONLY_ADDRESS [\#1144](https://github.com/CosmWasm/wasmd/issues/1144)
- Support builds without CGO [\#1129](https://github.com/CosmWasm/wasmd/issues/1129)
- Wasmd does not sort coins when converting from CosmWasm Coins to SDK
  Coins [\#1118](https://github.com/CosmWasm/wasmd/issues/1118)
- Add InstantiateContract2Proposal [\#1062](https://github.com/CosmWasm/wasmd/issues/1062)
- CLI: Allow using key name for --admin [\#1039](https://github.com/CosmWasm/wasmd/issues/1039)
- More gov proposal simulations [\#1107](https://github.com/CosmWasm/wasmd/pull/1107)
- Remove genesis messages [\#987](https://github.com/CosmWasm/wasmd/issues/987)
- Update instantiate config command [\#843](https://github.com/CosmWasm/wasmd/issues/843)
- Upgrade IBC-go to [v4.3.0](https://github.com/cosmos/ibc-go/releases/tag/v4.3.0) [\#1180](https://github.com/CosmWasm/wasmd/pull/1180)
- Upgrade ICA to [v0.2.6](https://github.com/cosmos/interchain-accounts-demo/releases/tag/v0.2.6) [\#1192](https://github.com/CosmWasm/wasmd/pull/1192)

### Notable changes:

- Genesis messages were deprecated before and are removed with this release
- New `cosmwasm_1_2` [capability](https://github.com/CosmWasm/cosmwasm/blob/main/docs/CAPABILITIES-BUILT-IN.md) to
  enable new features:
  - Support for `gov.MsgVoteWeighted`, `wasm.Instantiate2` messages
  - code info query for contracts
- See "State Machine Breaking" changes in [IBC-go](https://github.com/cosmos/ibc-go/releases/tag/v4.3.0)
- See notes about the "store fix" in [Cosmos-sdk](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.45.12)
- Wasmd can now be used as a library without CGO
- Wasmd client can now be used on Windows

### Migration notes:

- This release does not include any state migrations but breaking changes that require a coordinated chain upgrade

## [v0.30.0](https://github.com/CosmWasm/wasmd/tree/v0.30.0) (2022-12-02)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.29.2...v0.30.0)

- Provide source, builder and codehash information in store code proposal message[\#1072](https://github.com/CosmWasm/wasmd/pull/1072)
- Add new CLI query/ endpoint to get contracts by creator address [\#998](https://github.com/CosmWasm/wasmd/pull/998)
- Upgrade to Go v1.19 [\#1044](https://github.com/CosmWasm/wasmd/pull/1044)
- Upgrade to Cosmos-sdk to v0.45.11 [/#1096](https://github.com/CosmWasm/wasmd/pull/1096/)
- Upgrade to IBC v4.2.0 with interchain-accounts v0.2.4 [\#1088](https://github.com/CosmWasm/wasmd/pull/1088)
- Preserve contract history/ created date on genesis import [\#878](https://github.com/CosmWasm/wasmd/issues/878)
- Authz module integration - more granularity for WasmExecuteMsg authorizations [\#803](https://github.com/CosmWasm/wasmd/issues/803)
- StoreAndInstantiate gov proposal [\#785](https://github.com/CosmWasm/wasmd/issues/785)
- Start developer guide for contributors [\#654](https://github.com/CosmWasm/wasmd/issues/654)

### Notable changes:

- IBC fee middleware is setup in `app.go`. Please note that it can be enabled with new channels only. A nice read is this [article](https://medium.com/the-interchain-foundation/ibc-relaying-as-a-service-the-in-protocol-incentivization-story-2c008861a957).
- Authz for wasm contracts can be granted via `wasmd tx wasm grant` and executed via `wasmd tx authz exec` command
- Go v1.19 required to prevent a mixed chain setup with older versions. Just to be on the safe side.
- Store code proposal types have new metadata fields added that can help to build client side tooling to verify the wasm contract in the proposal

### Migration notes:

- The wasmd module version was bumped and a [state migration](https://github.com/CosmWasm/wasmd/pull/1021/files#diff-4357c2137e24f583b8f852cc210320cb71af18e2fdfb8c21b55d8667cfe54690R20) registered.
- See ibc-go [migration notes](https://github.com/cosmos/ibc-go/blob/v4.2.0/docs/migrations)
- See interchain-accounts [`MsgRegisterAccount.Version` field](https://github.com/cosmos/interchain-accounts-demo/compare/v0.1.0...v0.2.4#diff-ac8bca25810de6d3eef95f74fc9acf2223f3687822e6227b584e0d3b40db6566). Full diff [v0.1.0 to v0.2.4](https://github.com/cosmos/interchain-accounts-demo/compare/v0.1.0...v0.2.4)

## [v0.29.2](https://github.com/CosmWasm/wasmd/tree/v0.29.2) (2022-11-08)

- Fixes missing instantiate-anyof-addresses flag declaration for gov [/#1084](https://github.com/CosmWasm/wasmd/issues/1084)

## [v0.29.1](https://github.com/CosmWasm/wasmd/tree/v0.29.1) (2022-10-14)

- Upgrade to Cosmos-sdk to v45.9 [/#1052](https://github.com/CosmWasm/wasmd/pull/1052/)

## [v0.29.0](https://github.com/CosmWasm/wasmd/tree/v0.29.0) (2022-10-10)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.28.0...v0.29.0)

- Add dependencies for protobuf and remove third_party forlder [/#1030](https://github.com/CosmWasm/wasmd/pull/1030)
- Check wasmvm version on startup [\#1029](https://github.com/CosmWasm/wasmd/pull/1029/)
- Allow AccessConfig to use a list of addresses instead of just a single address [\#945](https://github.com/CosmWasm/wasmd/issues/945)
- Make contract addresses predictable \("deterministic"\) [\#942](https://github.com/CosmWasm/wasmd/issues/942)
- Add query for the total supply of a coin [\#903](https://github.com/CosmWasm/wasmd/pull/903) ([larry0x](https://github.com/larry0x))
- Upgrade go to v1.18 [\#866]https://github.com/CosmWasm/wasmd/pull/866/) ([faddat](https://github.com/faddat))
- Upgrade to ibc-go v3.3.0 REQUIRES [MIGRATION](https://github.com/cosmos/ibc-go/blob/v3.2.3/docs/migrations/support-denoms-with-slashes.md) [\#1016](https://github.com/CosmWasm/wasmd/pull/1016)
- Upgrade to cosmos-sdk v0.45.8 [\#964](https://github.com/CosmWasm/wasmd/pull/964/) ([faddat](https://github.com/faddat))
- Upgrade wasmvm to v1.1.1 [\#1012](https://github.com/CosmWasm/wasmd/pull/1012), see [wasmvm v1.1.1](https://github.com/CosmWasm/wasmvm/releases/tag/v1.1.1)
- Add documentation how to add x/wasm to a new Cosmos SDK chain [\#876](https://github.com/CosmWasm/wasmd/issues/876)
- Upgrade keyring / go-keychain dependencies (removes deprecate warning) [\#957](https://github.com/CosmWasm/wasmd/issues/957)
- Make contract pinning an optional field in StoreCode proposals [\#972](https://github.com/CosmWasm/wasmd/issues/972)
- Add gRPC query for WASM params [\#889](https://github.com/CosmWasm/wasmd/issues/889)
- Expose Keepers in app.go? [\#881](https://github.com/CosmWasm/wasmd/issues/881)
- Remove unused `flagProposalType` flag in gov proposals [\#849](https://github.com/CosmWasm/wasmd/issues/849)
- Restrict code access config modifications [\#901](https://github.com/CosmWasm/wasmd/pull/901)
- Prevent migration to a restricted code [\#900](https://github.com/CosmWasm/wasmd/pull/900)
- Charge gas to unzip wasm code [\#898](https://github.com/CosmWasm/wasmd/pull/898)

### Notable changes:

- BaseAccount and pruned vesting account types can be re-used for contracts addresses
- A new [MsgInstantiateContract2](https://github.com/CosmWasm/wasmd/pull/1014/files#diff-bf58b9da4b674719f07dd5421c532c1ead13a15f8896b59c1f724215d2064b73R75) was introduced which is an additional value for `message` type events
- Store event contains a new attribute with the code checksum now
- New `wasmd tx wasm instantiate2` CLI command for predictable addresses on instantiation
- New `cosmwasm_1_1` CosmWasm capability (former "feature") was introduced in [cosmwasm/#1356](https://github.com/CosmWasm/cosmwasm/pull/1356) to support total supply queries
- Protobuf files are published to [buf.build](https://buf.build/cosmwasm/wasmd/docs/main:cosmwasm.wasm.v1)

### Migration notes:

- See ibc-go [migration notes](https://github.com/cosmos/ibc-go/blob/v3.3.0/docs/migrations/support-denoms-with-slashes.md)

## [v0.28.0](https://github.com/CosmWasm/wasmd/tree/v0.28.0) (2022-07-29)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.27.0...v0.28.0)

**API Breaking**

No

**Fixed Bugs**

- Fix: Make events in reply completely determinisitic by stripping out anything coming from Cosmos SDK (not CosmWasm codebase) [\#917](https://github.com/CosmWasm/wasmd/pull/917) ([assafmo](https://github.com/assafmo))

Migration notes:

- Contracts can no longer parse events from any calls except if they call another contract (or instantiate it, migrate it, etc).
  The main issue here is likely "Custom" queries from a blockchain, which want to send info (eg. how many tokens were swapped).
  Since those custom bindings are maintained by the chain, they can use the data field to pass any deterministic information
  back to the contract. We recommend using JSON encoding there with some documented format the contracts can parse out easily.
- For possible non-determinism issues, we also sort all attributes in events. Better safe than sorry.

## [v0.27.0](https://github.com/CosmWasm/wasmd/tree/v0.27.0) (2022-05-19)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.26.0...v0.27.0)

**API Breaking**
Yes

**Fixed bugs:**

- Fix: allow no admin in instantiation through proposal [\#828](https://github.com/CosmWasm/wasmd/pull/828) ([jhernandezb](https://github.com/jhernandezb))
- Fix SudoContractProposal and ExecuteContractProposal [\#808](https://github.com/CosmWasm/wasmd/pull/808) ([the-frey](https://github.com/the-frey))

**Implemented Enhancements**

- Add UpdateInstantiateConfig governance proposal [\#820](https://github.com/CosmWasm/wasmd/pull/796) ([jhernandezb](https://github.com/jhernandezb))
- Upgrade wasmvm to v1.0.0 [\#844](https://github.com/CosmWasm/wasmd/pull/844) and [\#858](https://github.com/CosmWasm/wasmd/pull/858)
- Support state sync [\#478](https://github.com/CosmWasm/wasmd/issues/478)
- Upgrade to ibc-go v3 [\#806](https://github.com/CosmWasm/wasmd/issues/806)
- Initial ICA integration [\#837](https://github.com/CosmWasm/wasmd/pull/837) ([ethanfrey](https://github.com/ethanfrey))
- Consolidate MaxWasmSize constraints into a single var [\#826](https://github.com/CosmWasm/wasmd/pull/826)
- Add AccessConfig to CodeInfo query response [\#829](https://github.com/CosmWasm/wasmd/issues/829)
- Bump sdk to v0.45.4 [\#818](https://github.com/CosmWasm/wasmd/pull/818) ([alpe](https://github.com/alpe))
- Bump buf docker image to fix proto generation issues [\#820](https://github.com/CosmWasm/wasmd/pull/820) ([alpe](https://github.com/alpe))
- Add MsgStoreCode and MsgInstantiateContract support to simulations [\#831](https://github.com/CosmWasm/wasmd/pull/831) ([pinosu](https://github.com/pinosu))

**Implemented Enhancements**

- Make MaxLabelSize a var not const [\#822](https://github.com/CosmWasm/wasmd/pull/822)

## [v0.26.0](https://github.com/CosmWasm/wasmd/tree/v0.26.0) (2022-04-21)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.25.0...v0.26.0)

**Fixed bugs:**

- Unpack contract details from genesis [\#802](https://github.com/CosmWasm/wasmd/pull/802)

**Closed issues:**

- Issue Updating uploadAccess Param [\#804](https://github.com/CosmWasm/wasmd/issues/804)
- Add tx query to wasmd QueryPlugins for smart contract [\#788](https://github.com/CosmWasm/wasmd/issues/788)

**Merged pull requests:**

- Disable stargate queries [\#812](https://github.com/CosmWasm/wasmd/pull/812)
- Gov param change examples [\#805](https://github.com/CosmWasm/wasmd/pull/805)
- Create link to SECURITY.md in other repo [\#801](https://github.com/CosmWasm/wasmd/pull/801)
- Tests some event edge cases [\#799](https://github.com/CosmWasm/wasmd/pull/799)

## [v0.25.0](https://github.com/CosmWasm/wasmd/tree/v0.25.0) (2022-04-06)

**API Breaking**

- Upgrade wasmvm to v1.0.0-beta10 [\#790](https://github.com/CosmWasm/wasmd/pull/790), [\#800](https://github.com/CosmWasm/wasmd/pull/800)

**Implemented Enhancements**

- Fix: close iterators [\#792](https://github.com/CosmWasm/wasmd/pull/792)
- Use callback pattern for contract state iterator [\#794](https://github.com/CosmWasm/wasmd/pull/794)
- Bump github.com/stretchr/testify from 1.7.0 to 1.7.1 [\#787](https://github.com/CosmWasm/wasmd/pull/787)
- Bump github.com/cosmos/ibc-go/v2 from 2.0.3 to 2.2.0 [\#786](https://github.com/CosmWasm/wasmd/pull/786)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.24.0...v0.25.0)

## [v0.24.0](https://github.com/CosmWasm/wasmd/tree/v0.24.0) (2022-03-09)

**API Breaking**

- Add cosmwasm project prefix to REST query paths [\#743](https://github.com/CosmWasm/wasmd/issues/743)
- Add support for old contract addresses of length 20 [\#758](https://github.com/CosmWasm/wasmd/issues/758)
- Update wasmvm to 1.0.0-beta7 (incl wasmer 2.2) [\#774](https://github.com/CosmWasm/wasmd/issues/774)

**Fixed bugs**

- Add missing colons in String of some proposals [\#752](https://github.com/CosmWasm/wasmd/pull/752)
- Replace custom codec with SDK codec (needed for rosetta) [\#760](https://github.com/CosmWasm/wasmd/pull/760)
- Support `--no-admin` flag on cli for gov instantiation [\#771](https://github.com/CosmWasm/wasmd/pull/771)

**Implemented Enhancements**

- Add support for Buf Build [\#753](https://github.com/CosmWasm/wasmd/pull/753), [\#755](https://github.com/CosmWasm/wasmd/pull/755), [\#756](https://github.com/CosmWasm/wasmd/pull/756)
- Redact most errors sent to contracts, for better determinism guarantees [\#765](https://github.com/CosmWasm/wasmd/pull/765), [\#775](https://github.com/CosmWasm/wasmd/pull/775)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.23.0...v0.24.0)

## [v0.23.0](https://github.com/CosmWasm/wasmd/tree/v0.23.0) (2022-01-28)

**Fixed bugs**

- Set end block order [\#736](https://github.com/CosmWasm/wasmd/issues/736)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.22.0...v0.23.0)

## [v0.22.0](https://github.com/CosmWasm/wasmd/tree/v0.22.0) (2022-01-20)

**Api Breaking:**

- Upgrade to cosmos-sdk v0.45.0 [\#717](https://github.com/CosmWasm/wasmd/pull/717)
- Upgrade wasmvm to v1.0.0-beta5 [\#714](https://github.com/CosmWasm/wasmd/pull/714)

**Implemented Enhancements:**

- Use proper SystemError::NoSuchContract on ContractInfo if missing [\#687](https://github.com/CosmWasm/wasmd/issues/687)
- Benchmark tests flickering: directory not empty [\#659](https://github.com/CosmWasm/wasmd/issues/659)
- Implement PinCode and UnpinCode proposal client handlers [\#707](https://github.com/CosmWasm/wasmd/pull/707) ([orkunkl](https://github.com/orkunkl))
- Use replace statements to enforce consistent versioning. [\#692](https://github.com/CosmWasm/wasmd/pull/692) ([faddat](https://github.com/faddat))
- Fixed circleci by removing the golang executor from a docker build
- Go 1.17 provides a much clearer go.mod file [\#679](https://github.com/CosmWasm/wasmd/pull/679) ([faddat](https://github.com/faddat))
- Autopin wasm code uploaded by gov proposal [\#726](https://github.com/CosmWasm/wasmd/pull/726) ([ethanfrey](https://github.com/ethanfrey))
- You must explicitly declare --no-admin on cli instantiate if that is what you want [\#727](https://github.com/CosmWasm/wasmd/pull/727) ([ethanfrey](https://github.com/ethanfrey))
- Add governance proposals for Wasm Execute and Sudo [\#730](https://github.com/CosmWasm/wasmd/pull/730) ([ethanfrey](https://github.com/ethanfrey))
- Remove unused run-as flag from Wasm Migrate proposals [\#730](https://github.com/CosmWasm/wasmd/pull/730) ([ethanfrey](https://github.com/ethanfrey))
- Expose wasm/Keeper.SetParams [\#732](https://github.com/CosmWasm/wasmd/pull/732) ([ethanfrey](https://github.com/ethanfrey))

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.21.0...v0.22.0)

## [v0.21.0](https://github.com/CosmWasm/wasmd/tree/v0.21.0) (2021-11-17)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.20.0...v0.21.0)

**Fixed bugs + Api Breaking:**

- Prevent infinite gas consumption in simulation queries [\#670](https://github.com/CosmWasm/wasmd/issues/670)
- Amino JSON representation of inner message in Msg{Instantiate,Migrate,Execute}Contract [\#642](https://github.com/CosmWasm/wasmd/issues/642)

**Implemented Enhancements:**

- Bump wasmvm to v1.0.0-beta2 [\#676](https://github.com/CosmWasm/wasmd/pull/676)
- Add Benchmarks to compare with native modules [\#635](https://github.com/CosmWasm/wasmd/issues/635)
- Document M1 is not supported [\#653](https://github.com/CosmWasm/wasmd/issues/653)
- Open read access to sequences [\#669](https://github.com/CosmWasm/wasmd/pull/669)
- Remove unused flags from command prompt for storing contract [\#647](https://github.com/CosmWasm/wasmd/issues/647)
- Ran `make format` [\#649](https://github.com/CosmWasm/wasmd/issues/649)
- Add golangci lint check to circleci jobs [\620](https://github.com/CosmWasm/wasmd/issues/620)
- Updated error log statements in initGenesis for easier debugging: [\#643](https://github.com/CosmWasm/wasmd/issues/643)
- Bump github.com/cosmos/iavl from 0.17.1 to 0.17.2 [\#673](https://github.com/CosmWasm/wasmd/pull/673)
- Bump github.com/rs/zerolog from 1.25.0 to 1.26.0 [\#666](https://github.com/CosmWasm/wasmd/pull/666)

## [v0.20.0](https://github.com/CosmWasm/wasmd/tree/v0.20.0) (2021-10-08)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.19.0...v0.20.0)

**Fixed bugs:**

- Add capabilities to begin block [\#626](https://github.com/CosmWasm/wasmd/pull/626)

**Api Breaking:**

- Update to wasmvm 1.0.0-soon2 [\#624](https://github.com/CosmWasm/wasmd/issues/624)

**Implemented Enhancements:**

- Upgrade Cosmos-sdk v0.42.10 [\#627](https://github.com/CosmWasm/wasmd/pull/627) ([alpe](https://github.com/alpe))
- Add transaction index implemented as counter [\#601](https://github.com/CosmWasm/wasmd/issues/601)
- Fix inconsistent return of `contractAddress` from `keeper/init()`? [\#616](https://github.com/CosmWasm/wasmd/issues/616)
- Query pinned wasm codes [\#596](https://github.com/CosmWasm/wasmd/issues/596)
- Doc IBC Events [\#593](https://github.com/CosmWasm/wasmd/issues/593)
- Allow contract Info query from the contract [\#584](https://github.com/CosmWasm/wasmd/issues/584)
- Revisit reply gas costs for submessages. [\#450](https://github.com/CosmWasm/wasmd/issues/450)
- Benchmarks for gas pricing [\#634](https://github.com/CosmWasm/wasmd/pull/634)
- Treat all contracts as pinned for gas costs in reply [\#630](https://github.com/CosmWasm/wasmd/pull/630)
- Bump github.com/spf13/viper from 1.8.1 to 1.9.0 [\#615](https://github.com/CosmWasm/wasmd/pull/615)

## [v0.19.0](https://github.com/CosmWasm/wasmd/tree/v0.19.0) (2021-09-15)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.18.0...v0.19.0)

**Fixed bugs:**

- Ensure Queries are executed read only [\#610](https://github.com/CosmWasm/wasmd/issues/610)
- Fix bug in query handler initialization on reply [\#604](https://github.com/CosmWasm/wasmd/issues/604)

**Api Breaking:**

- Bump Go version to 1.16 [\#612](https://github.com/CosmWasm/wasmd/pull/612)

**Implemented Enhancements:**

- Ensure query isolation [\#611](https://github.com/CosmWasm/wasmd/pull/611)
- Optimize BalanceQuery [\#609](https://github.com/CosmWasm/wasmd/pull/609)
- Bump wasmvm to v0.16.1 [\#605](https://github.com/CosmWasm/wasmd/pull/605)
- Bump github.com/rs/zerolog from 1.23.0 to 1.25.0 [\#603](https://github.com/CosmWasm/wasmd/pull/603)
- Add decorator options [\#598](https://github.com/CosmWasm/wasmd/pull/598)
- Bump github.com/spf13/cast from 1.4.0 to 1.4.1 [\#592](https://github.com/CosmWasm/wasmd/pull/592)

## [v0.18.0](https://github.com/CosmWasm/wasmd/tree/v0.18.0) (2021-08-16)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.17.0...v0.18.0)

**Api Breaking:**

- Events documented and refactored [\#448](https://github.com/CosmWasm/wasmd/issues/448), [\#589](https://github.com/CosmWasm/wasmd/pull/589), [\#587](https://github.com/CosmWasm/wasmd/issues/587)
- Add organisation to grpc gateway path [\#578](https://github.com/CosmWasm/wasmd/pull/578)
- Move Proto version from `v1beta1` to `v1` for all cosmwasm.wasm.\* types
  [\#563](https://github.com/CosmWasm/wasmd/pull/563)
- Renamed InitMsg and MigrateMsg fields to Msg. This applies to protobuf Msg
  and Proposals, as well as REST and CLI [\#563](https://github.com/CosmWasm/wasmd/pull/563)
- Removed source and builder fields from StoreCode and CodeInfo. They were rarely used.
  [\#564](https://github.com/CosmWasm/wasmd/pull/564)
- Changed contract address derivation function. If you hardcoded the first contract
  addresses anywhere (in scripts?), please update them.
  [\#565](https://github.com/CosmWasm/wasmd/pull/565)

**Implemented Enhancements:**

- Cosmos SDK 0.42.9, wasmvm 0.16.0 [\#582](https://github.com/CosmWasm/wasmd/pull/582)
- Better ibc contract interface [\#570](https://github.com/CosmWasm/wasmd/pull/570) ([ethanfrey](https://github.com/ethanfrey))
- Reject invalid events/attributes returned from contracts [\#560](https://github.com/CosmWasm/wasmd/pull/560)
- IBC Query methods from Wasm contracts only return OPEN channels [\#568](https://github.com/CosmWasm/wasmd/pull/568)
- Extendable gas costs [\#525](https://github.com/CosmWasm/wasmd/issues/525)
- Limit init/migrate/execute payload message size [\#203](https://github.com/CosmWasm/wasmd/issues/203)
- Add cli alias [\#496](https://github.com/CosmWasm/wasmd/issues/496)
- Remove max gas limit [\#529](https://github.com/CosmWasm/wasmd/pull/529) ([alpe](https://github.com/alpe))
- Add SECURITY.md [\#303](https://github.com/CosmWasm/wasmd/issues/303)

## [v0.17.0](https://github.com/CosmWasm/wasmd/tree/v0.17.0) (2021-05-26)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.16.0...v0.17.0)

**Features:**

- Remove json type cast for contract msgs [\#520](https://github.com/CosmWasm/wasmd/pull/520) ([alpe](https://github.com/alpe))
- Bump github.com/cosmos/cosmos-sdk from 0.42.4 to 0.42.5 [\#519](https://github.com/CosmWasm/wasmd/pull/519) ([dependabot-preview[bot]](https://github.com/apps/dependabot-preview))

## [v0.16.0](https://github.com/CosmWasm/wasmd/tree/v0.16.0) (2021-04-30)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.15.1...v0.16.0)

**Features:**

- Upgrade to wasmvm v0.14.0-rc1 [\#508](https://github.com/CosmWasm/wasmd/pull/508) ([alpe](https://github.com/alpe))
- Use the cache metrics from WasmVM [\#500](https://github.com/CosmWasm/wasmd/issues/500)
- Update IBC.md [\#494](https://github.com/CosmWasm/wasmd/pull/494) ([ethanfrey](https://github.com/ethanfrey))
- Extend ContractInfo for custom data [\#492](https://github.com/CosmWasm/wasmd/pull/492) ([alpe](https://github.com/alpe))
- Reply response on submessages can overwrite "caller" result [\#495](https://github.com/CosmWasm/wasmd/issues/495)
- Update to sdk 0.42.4 [\#485](https://github.com/CosmWasm/wasmd/issues/485)
- Add extension points to the CLI [\#477](https://github.com/CosmWasm/wasmd/pull/477) ([alpe](https://github.com/alpe))
- Simplify staking reward query [\#399](https://github.com/CosmWasm/wasmd/issues/399)
- Update IBC.md [\#398](https://github.com/CosmWasm/wasmd/issues/398)
- Add IBCQuery support [\#434](https://github.com/CosmWasm/wasmd/issues/434)
- Follow proto dir best practice \(in cosmos eco\) [\#342](https://github.com/CosmWasm/wasmd/issues/342)
- Remove internal package [\#464](https://github.com/CosmWasm/wasmd/pull/464) ([alpe](https://github.com/alpe))
- Introduce new interfaces for extendability [\#471](https://github.com/CosmWasm/wasmd/pull/471) ([alpe](https://github.com/alpe))
- Handle non default IBC transfer port in message encoder [\#396](https://github.com/CosmWasm/wasmd/issues/396)
- Collect Contract Metrics [\#387](https://github.com/CosmWasm/wasmd/issues/387)
- Add Submessages for IBC callbacks [\#449](https://github.com/CosmWasm/wasmd/issues/449)
- Handle wasmvm Burn message [\#489](https://github.com/CosmWasm/wasmd/pull/489) ([alpe](https://github.com/alpe))
- Add telemetry [\#463](https://github.com/CosmWasm/wasmd/pull/463) ([alpe](https://github.com/alpe))
- Handle non default transfer port id [\#462](https://github.com/CosmWasm/wasmd/pull/462) ([alpe](https://github.com/alpe))
- Allow subsecond block times [\#453](https://github.com/CosmWasm/wasmd/pull/453) ([ethanfrey](https://github.com/ethanfrey))
- Submsg and replies [\#441](https://github.com/CosmWasm/wasmd/pull/441) ([ethanfrey](https://github.com/ethanfrey))
- Ibc query support [\#439](https://github.com/CosmWasm/wasmd/pull/439) ([ethanfrey](https://github.com/ethanfrey))
- Pin/Unpin contract in cache [\#436](https://github.com/CosmWasm/wasmd/pull/436) ([alpe](https://github.com/alpe))
- Stargate msg and query [\#435](https://github.com/CosmWasm/wasmd/pull/435) ([ethanfrey](https://github.com/ethanfrey))
- Sudo entry point [\#433](https://github.com/CosmWasm/wasmd/pull/433) ([ethanfrey](https://github.com/ethanfrey))
- Add custom message handler option [\#402](https://github.com/CosmWasm/wasmd/pull/402) ([alpe](https://github.com/alpe))
- Expose contract pinning [\#401](https://github.com/CosmWasm/wasmd/issues/401)
- Add support for Stargate CosmosMsg/QueryRequest [\#388](https://github.com/CosmWasm/wasmd/issues/388)
- Add MsgInstantiateContractResponse.data [\#385](https://github.com/CosmWasm/wasmd/issues/385)
- Added randomized simulation parameters generation [\#389](https://github.com/CosmWasm/wasmd/pull/389) ([bragaz](https://github.com/bragaz))
- Implement IBC contract support [\#394](https://github.com/CosmWasm/wasmd/pull/394) ([alpe](https://github.com/alpe))

**Api breaking:**

- Improve list contracts by code query [\#497](https://github.com/CosmWasm/wasmd/pull/497) ([alpe](https://github.com/alpe))
- Rename to just `funds` [/#423](https://github.com/CosmWasm/wasmd/issues/423)

**Fixed bugs:**

- Correct order for migrated contracts [\#323](https://github.com/CosmWasm/wasmd/issues/323)
- Keeper Send Coins does not perform expected validation [\#414](https://github.com/CosmWasm/wasmd/issues/414)

## [v0.15.1](https://github.com/CosmWasm/wasmd/tree/v0.15.1) (2021-02-18)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.15.0...v0.15.1)

**Implemented enhancements:**

- Support custom MessageHandler in wasm [\#327](https://github.com/CosmWasm/wasmd/issues/327)

**Fixed bugs:**

- Fix Parameter change via proposal [\#392](https://github.com/CosmWasm/wasmd/issues/392)

## [v0.15.0](https://github.com/CosmWasm/wasmd/tree/v0.15.0) (2021-01-27)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.14.1...v0.15.0)

**Features:**

- Upgrade to cosmos-sdk v0.41.0 [\#390](https://github.com/CosmWasm/wasmd/pull/390)

## [v0.14.1](https://github.com/CosmWasm/wasmd/tree/v0.14.1) (2021-01-20)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.14.0...v0.14.1)

**Features:**

- Upgrade to cosmos-sdk v0.40.1 final + Tendermint 0.34.3 [\#380](https://github.com/CosmWasm/wasmd/pull/380)

## [v0.14.0](https://github.com/CosmWasm/wasmd/tree/v0.14.0) (2021-01-11)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.13.0...v0.14.0)

**Features:**

- Upgrade to cosmos-sdk v0.40.0 final [\#354](https://github.com/CosmWasm/wasmd/pull/369)
- Refactor to GRPC message server [\#366](https://github.com/CosmWasm/wasmd/pull/366)
- Make it easy to initialize contracts in genesis file with new CLI commands[\#326](https://github.com/CosmWasm/wasmd/issues/326)
- Upgrade to WasmVM v0.13.0 [\#358](https://github.com/CosmWasm/wasmd/pull/358)
- Upgrade to cosmos-sdk v0.40.0-rc6 [\#354](https://github.com/CosmWasm/wasmd/pull/354)
- Upgrade to cosmos-sdk v0.40.0-rc5 [\#344](https://github.com/CosmWasm/wasmd/issues/344)
- Add Dependabot to keep dependencies secure and up-to-date [\#336](https://github.com/CosmWasm/wasmd/issues/336)

**Fixed bugs:**

- Dependabot can't resolve your Go dependency files [\#339](https://github.com/CosmWasm/wasmd/issues/339)
- Errors in `InitGenesis` [\#335](https://github.com/CosmWasm/wasmd/issues/335)
- Invalid homeDir for export command [\#334](https://github.com/CosmWasm/wasmd/issues/334)

## [v0.13.0](https://github.com/CosmWasm/wasmd/tree/v0.13.0) (2020-12-04)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.12.1...v0.13.0)

**Fixed bugs:**

- REST handler wrong `Sender` source [\#324](https://github.com/CosmWasm/wasmd/issues/324)

**Closed issues:**

- Change proto package to match \<organisation\>.\<module\>.\<version\> [\#329](https://github.com/CosmWasm/wasmd/issues/329)
- Out of gas causes panic when external contract store query executed [\#321](https://github.com/CosmWasm/wasmd/issues/321)
- Check codecov report [\#298](https://github.com/CosmWasm/wasmd/issues/298)
- cosmwasm.GoAPI will not work on sdk.ValAddress [\#264](https://github.com/CosmWasm/wasmd/issues/264)
- Stargate: Add pagination support for queries [\#242](https://github.com/CosmWasm/wasmd/issues/242)

**Merged pull requests:**

- Rename protobuf package [\#330](https://github.com/CosmWasm/wasmd/pull/330) ([alpe](https://github.com/alpe))
- Use base request data for sender [\#325](https://github.com/CosmWasm/wasmd/pull/325) ([alpe](https://github.com/alpe))
- Handle panics in query contract smart [\#322](https://github.com/CosmWasm/wasmd/pull/322) ([alpe](https://github.com/alpe))

## [v0.12.1](https://github.com/CosmWasm/wasmd/tree/v0.12.1) (2020-11-23)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.12.0...v0.12.1)

**Closed issues:**

- Complete IBC Mock testing [\#255](https://github.com/CosmWasm/wasmd/issues/255)
- Idea: do multiple queries in one API call [\#72](https://github.com/CosmWasm/wasmd/issues/72)

**Merged pull requests:**

- Exclude generate proto code files in coverage [\#320](https://github.com/CosmWasm/wasmd/pull/320) ([alpe](https://github.com/alpe))
- Upgrade wasmvm to 0.12.0 [\#319](https://github.com/CosmWasm/wasmd/pull/319) ([webmaster128](https://github.com/webmaster128))
- Fix chain id setup in contrib/local/setup_wasmd.sh [\#318](https://github.com/CosmWasm/wasmd/pull/318) ([orkunkl](https://github.com/orkunkl))
- Add pagination to grpc queries [\#317](https://github.com/CosmWasm/wasmd/pull/317) ([alpe](https://github.com/alpe))

## [v0.12.0](https://github.com/CosmWasm/wasmd/tree/v0.12.0) (2020-11-17)

[Full Changelog](https://github.com/CosmWasm/wasmd/compare/v0.12.0-alpha1...v0.12.0)

**Closed issues:**

- Merge wasmd and wasmcli into a single binary [\#308](https://github.com/CosmWasm/wasmd/issues/308)
- Change bech32 prefix for wasmd [\#313](https://github.com/CosmWasm/wasmd/issues/313)
- Upgrade go-cowasmwasm to wasmvm 0.12 [\#309](https://github.com/CosmWasm/wasmd/issues/309)
- Use string type for AccAddresses in proto [\#306](https://github.com/CosmWasm/wasmd/issues/306)
- Upgrade to cosmos/sdk v0.40.0-rc2 [\#296](https://github.com/CosmWasm/wasmd/issues/296)
- Generate protobuf outputs in a container [\#295](https://github.com/CosmWasm/wasmd/issues/295)
- Instantiate contract process ordering [\#292](https://github.com/CosmWasm/wasmd/issues/292)
- Make Wasm maxSize a configuration option [\#289](https://github.com/CosmWasm/wasmd/issues/289)
- Return error if wasm to big [\#287](https://github.com/CosmWasm/wasmd/issues/287)

**Merged pull requests:**

- Set bech32 prefix [\#316](https://github.com/CosmWasm/wasmd/pull/316) ([alpe](https://github.com/alpe))
- Replace sdk.AccAddress with bech32 string [\#314](https://github.com/CosmWasm/wasmd/pull/314) ([alpe](https://github.com/alpe))
- Integrate wasmcli into wasmd [\#312](https://github.com/CosmWasm/wasmd/pull/312) ([alpe](https://github.com/alpe))
- Upgrade wasmvm aka go-cosmwasm [\#311](https://github.com/CosmWasm/wasmd/pull/311) ([alpe](https://github.com/alpe))
- Upgrade to Stargate RC3 [\#305](https://github.com/CosmWasm/wasmd/pull/305) ([alpe](https://github.com/alpe))
- Containerized Protobuf generation [\#304](https://github.com/CosmWasm/wasmd/pull/304) ([alpe](https://github.com/alpe))
- Reject wasm code exceeding limit [\#302](https://github.com/CosmWasm/wasmd/pull/302) ([alpe](https://github.com/alpe))
- Support self calling contract on instantiation [\#300](https://github.com/CosmWasm/wasmd/pull/300) ([alpe](https://github.com/alpe))
- Upgrade to Stargate RC2 [\#299](https://github.com/CosmWasm/wasmd/pull/299) ([alpe](https://github.com/alpe))
