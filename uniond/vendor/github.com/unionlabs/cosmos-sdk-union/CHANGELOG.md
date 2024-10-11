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
appropriate stanza (see below). Each entry is required to include a tag and
the Github issue reference in the following format:

* (<tag>) \#<issue-number> message

The tag should consist of where the change is being made ex. (x/staking), (store)
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
"State Machine Breaking" for any changes that result in a different AppState given same genesisState and txList.
Ref: https://keepachangelog.com/en/1.0.0/
-->

# Changelog

## [Unreleased]

Every module contains its own CHANGELOG.md. Please refer to the module you are interested in.

### Features

### Improvements

### Bug Fixes 

### API Breaking Changes

## [v0.52.0](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.52.0) - 2024-XX-XX

Every module contains its own CHANGELOG.md. Please refer to the module you are interested in.

### Features

* (client) [#17513](https://github.com/cosmos/cosmos-sdk/pull/17513) Allow overwriting `client.toml`. Use `client.CreateClientConfig` in place of `client.ReadFromClientConfig` and provide a custom template and a custom config.
* (tests) [#17868](https://github.com/cosmos/cosmos-sdk/pull/17868) Added helper method `SubmitTestTx` in testutil to broadcast test txns to test e2e tests.
* (client) [#18101](https://github.com/cosmos/cosmos-sdk/pull/18101) Add a `keyring-default-keyname` in `client.toml` for specifying a default key name, and skip the need to use the `--from` flag when signing transactions.
* (runtime) [#18475](https://github.com/cosmos/cosmos-sdk/pull/18475) Adds an implementation for core.branch.Service.
* (baseapp) [#18499](https://github.com/cosmos/cosmos-sdk/pull/18499) Add `MsgRouter` response type from message name function.
* (client) [#18557](https://github.com/cosmos/cosmos-sdk/pull/18557) Add `--qrcode` flag to `keys show` command to support displaying keys address QR code.
* (types) [#18768](https://github.com/cosmos/cosmos-sdk/pull/18768) Add MustValAddressFromBech32 function.
* (gRPC) [#19049](https://github.com/cosmos/cosmos-sdk/pull/19049) Add debug log prints for each gRPC request.
* (types) [#19164](https://github.com/cosmos/cosmos-sdk/pull/19164) Add a ValueCodec for the math.Uint type that can be used in collections maps.
* (types) [#19281](https://github.com/cosmos/cosmos-sdk/pull/19281) Added a new method, `IsGT`, for `types.Coin`. This method is used to check if a `types.Coin` is greater than another `types.Coin`.
* (runtime) [#19571](https://github.com/cosmos/cosmos-sdk/pull/19571) Implement `core/router.Service` in runtime. This service is present in all modules (when using depinject).
* (types) [#19759](https://github.com/cosmos/cosmos-sdk/pull/19759) Align SignerExtractionAdapter in PriorityNonceMempool Remove.
* (client) [#19870](https://github.com/cosmos/cosmos-sdk/pull/19870) Add new query command `wait-tx`. Alias `event-query-tx-for` to `wait-tx` for backward compatibility.
* (client) [#19905](https://github.com/cosmos/cosmos-sdk/pull/19905) Add grpc client config to `client.toml`.
* (genutil) [#19971](https://github.com/cosmos/cosmos-sdk/pull/19971) Allow manually setting the consensus key type in genesis
* (runtime) [#19953](https://github.com/cosmos/cosmos-sdk/pull/19953) Implement `core/transaction.Service` in runtime.
* (runtime) [#18475](https://github.com/cosmos/cosmos-sdk/pull/18475) Adds an implementation for `core.branch.Service`.
* (runtime) [#19004](https://github.com/cosmos/cosmos-sdk/pull/19004) Adds an implementation for `core/header.Service` in runtime.
* (runtime) [#20238](https://github.com/cosmos/cosmos-sdk/pull/20238) Adds an implementation for `core/comet.Service` in runtime.
* (tests) [#20013](https://github.com/cosmos/cosmos-sdk/pull/20013) Introduce system tests to run multi node local testnet in CI
* (crypto/keyring) [#20212](https://github.com/cosmos/cosmos-sdk/pull/20212) Expose the db keyring used in the keystore.
* (client/tx) [#20870](https://github.com/cosmos/cosmos-sdk/pull/20870) Add `timeout-timestamp` field for tx body defines time based timeout.Add `WithTimeoutTimestamp` to tx factory. Increased gas cost for processing newly added timeout timestamp field in tx body.
* (client) [#21074](https://github.com/cosmos/cosmos-sdk/pull/21074) Add auto cli for node service
* (x/validate) [#21822](https://github.com/cosmos/cosmos-sdk/pull/21822) New module solely responsible for providing ante/post handlers and tx validators for v2. It can be extended by the app developer to provide extra tx validators.
    * In comparison to x/auth/tx/config, there is no app config to skip ante/post handlers, as overwriting them in baseapp or not injecting the x/validate module has the same effect.
* (baseapp) [#21979](https://github.com/cosmos/cosmos-sdk/pull/21979) Create CheckTxHandler to allow extending the logic of CheckTx.

### Improvements

* (all) [#16537](https://github.com/cosmos/cosmos-sdk/pull/16537) Properly propagated `fmt.Errorf` errors and using `errors.New` where appropriate.
* (client) [#17503](https://github.com/cosmos/cosmos-sdk/pull/17503) Add `client.Context{}.WithAddressCodec`, `WithValidatorAddressCodec`, `WithConsensusAddressCodec` to provide address codecs to the client context. See the [UPGRADING.md](./UPGRADING.md) for more details.
* (crypto/keyring) [#17503](https://github.com/cosmos/cosmos-sdk/pull/17503) Simplify keyring interfaces to use `[]byte` instead of `sdk.Address` for addresses.
* (rpc) [#17470](https://github.com/cosmos/cosmos-sdk/pull/17470) Avoid open 0.0.0.0 to public by default and add `listen-ip-address` argument for `testnet init-files` cmd.
* (types) [#17670](https://github.com/cosmos/cosmos-sdk/pull/17670) Use `ctx.CometInfo` in place of `ctx.VoteInfos`
* [#17733](https://github.com/cosmos/cosmos-sdk/pull/17733) Ensure `buf export` exports all proto dependencies
* (crypto/keys) [#18026](https://github.com/cosmos/cosmos-sdk/pull/18026) Made public key generation constant time on `secp256k1`
* (crypto | x/auth) [#14372](https://github.com/cosmos/cosmos-sdk/pull/18194) Key checks on signatures antehandle.
* (types) [#18440](https://github.com/cosmos/cosmos-sdk/pull/18440) Add `AmountOfNoValidation` to `sdk.DecCoins`.
* (client/keys) [#18663](https://github.com/cosmos/cosmos-sdk/pull/18663) Improve `<appd> keys add` by displaying mnemonic discreetly on an alternate screen and adding `--indiscreet` option to disable it.
* (client/keys) [#18684](https://github.com/cosmos/cosmos-sdk/pull/18684) Improve `<appd> keys export` by displaying unarmored hex private key discreetly on an alternate screen and adding `--indiscreet` option to disable it.
* (client/keys) [#18687](https://github.com/cosmos/cosmos-sdk/pull/18687) Improve `<appd> keys mnemonic` by displaying mnemonic discreetly on an alternate screen and adding `--indiscreet` option to disable it.
* (client/keys) [#18703](https://github.com/cosmos/cosmos-sdk/pull/18703) Improve `<appd> keys add` and `<appd> keys show` by checking whether there are duplicate keys in the multisig case.
    * Usage of `Must...` kind of functions are avoided in keeper methods.
* (client/keys) [#18743](https://github.com/cosmos/cosmos-sdk/pull/18743) Improve `<appd> keys add -i` by hiding inputting of bip39 passphrase.
* (client/keys) [#18745](https://github.com/cosmos/cosmos-sdk/pull/18745) Improve `<appd> keys export` and `<appd> keys mnemonic` by adding --yes option to skip interactive confirmation.
* (client/keys) [#18950](https://github.com/cosmos/cosmos-sdk/pull/18950) Improve `<appd> keys add`, `<appd> keys import` and `<appd> keys rename` by checking name validation.
* (types) [#18963](https://github.com/cosmos/cosmos-sdk/pull/18963) Swap out amino json encoding of `ABCIMessageLogs` for std lib json encoding
* (types) [#19512](https://github.com/cosmos/cosmos-sdk/pull/19512) The notion of basic manager does not exist anymore (and all related helpers).
    * The module manager now can do everything that the basic manager was doing.
    * `AppModuleBasic` has been deprecated for extension interfaces.
    * Modules can now implement `appmodule.HasRegisterInterfaces`, `module.HasGRPCGateway` and `module.HasAminoCodec` when relevant.
    * SDK modules now directly implement those extension interfaces on `AppModule` instead of `AppModuleBasic`.
* (server) [#19455](https://github.com/cosmos/cosmos-sdk/pull/19455) Allow calling back into the application struct in PostSetup.
* (types) [#19672](https://github.com/cosmos/cosmos-sdk/pull/19672) `PreBlock` now returns only an error for consistency with server/v2. The SDK has upgraded x/upgrade accordingly. `ResponsePreBlock` hence has been removed.
* (x/auth) [#19651](https://github.com/cosmos/cosmos-sdk/pull/19651) Allow empty public keys in `GetSignBytesAdapter`.
* (x/genutil) [#19735](https://github.com/cosmos/cosmos-sdk/pull/19735) Update genesis api to match new `appmodule.HasGenesis` interface.
* (types) [#19869](https://github.com/cosmos/cosmos-sdk/pull/19869) Removed `Any` type from `codec/types` and replaced it with an alias for `cosmos/gogoproto/types/any`.
* (server) [#19854](https://github.com/cosmos/cosmos-sdk/pull/19854) Add customizability to start command.
    * Add `StartCmdOptions` in `server.AddCommands` instead of `servertypes.ModuleInitFlags`. To set custom flags set them in the `StartCmdOptions` struct on the `AddFlags` field.
    * Add `StartCommandHandler` to `StartCmdOptions` to allow custom start command handlers. Users now have total control over how the app starts.
* (server) [#19966](https://github.com/cosmos/cosmos-sdk/pull/19966) Return BlockHeader by shallow copy in server Context.
* (codec) [#20122](https://github.com/cosmos/cosmos-sdk/pull/20122) Added a cache to address codec.
* (proto) [#20098](https://github.com/cosmos/cosmos-sdk/pull/20098) Use cosmos_proto added_in annotation instead of // Since comments.
* (baseapp) [#20208](https://github.com/cosmos/cosmos-sdk/pull/20208) Skip running validateBasic for rechecking txs.
* (baseapp) [#20380](https://github.com/cosmos/cosmos-sdk/pull/20380) Enhanced OfferSnapshot documentation.
* (client) [#20771](https://github.com/cosmos/cosmos-sdk/pull/20771) Remove `ReadDefaultValuesFromDefaultClientConfig` from `client` package. (It was introduced in `v0.50.6` as a quick fix).
* (grpcserver) [#20945](https://github.com/cosmos/cosmos-sdk/pull/20945) Adds error handling for out-of-gas panics in grpc query handlers.
* (internal) [#21412](https://github.com/cosmos/cosmos-sdk/pull/21412) Using unsafe.String and unsafe.SliceData.
* (client) [#21436](https://github.com/cosmos/cosmos-sdk/pull/21436) Use `address.Codec` from client.Context in `tx.Sign`.
* (x/genutil) [#21249](https://github.com/cosmos/cosmos-sdk/pull/21249) Incremental JSON parsing for AppGenesis where possible.
* (genutil) [#21701](https://github.com/cosmos/cosmos-sdk/pull/21701) Improved error messages for genesis validation.
* (runtime) [#21704](https://github.com/cosmos/cosmos-sdk/pull/21704) Move `upgradetypes.StoreLoader` to runtime and alias it in upgrade for backward compatibility.
* (sims)[#21613](https://github.com/cosmos/cosmos-sdk/pull/21613) Add sims2 framework and factory methods for simpler message factories in modules
* (modules) [#21963](https://github.com/cosmos/cosmos-sdk/pull/21963) Duplicatable metrics are no more collected in modules. They were unecessary overhead.
* (crypto/ledger) [#22116](https://github.com/cosmos/cosmos-sdk/pull/22116) Improve error message when deriving paths using index >100

### Bug Fixes

* (baseapp) [#18383](https://github.com/cosmos/cosmos-sdk/pull/18383) Fixed a data race inside BaseApp.getContext, found by end-to-end (e2e) tests.
* (client/server) [#18345](https://github.com/cosmos/cosmos-sdk/pull/18345) Consistently set viper prefix in client and server. It defaults for the binary name for both client and server.
* (baseapp) [#18551](https://github.com/cosmos/cosmos-sdk/pull/18551) Fix SelectTxForProposal the calculation method of tx bytes size is inconsistent with CometBFT
* (client/keys) [#18562](https://github.com/cosmos/cosmos-sdk/pull/18562) `keys delete` won't terminate when a key is not found.
* (client) [#18622](https://github.com/cosmos/cosmos-sdk/pull/18622) Fixed a potential under/overflow from `uint64->int64` when computing gas fees as a LegacyDec.
* (baseapp) [#18727](https://github.com/cosmos/cosmos-sdk/pull/18727) Ensure that `BaseApp.Init` firstly returns any errors from a nil commit multistore instead of panicking on nil dereferencing and before sealing the app.
* (server) [#18994](https://github.com/cosmos/cosmos-sdk/pull/18994) Update server context directly rather than a reference to a sub-object
* [#19833](https://github.com/cosmos/cosmos-sdk/pull/19833) Fix some places in which we call Remove inside a Walk.
* [#19851](https://github.com/cosmos/cosmos-sdk/pull/19851) Fix some places in which we call Remove inside a Walk (x/staking and x/gov).
* (sims) [#21952](https://github.com/cosmos/cosmos-sdk/pull/21952) Use liveness matrix for validator sign status in sims
* (baseapp) [#21003](https://github.com/cosmos/cosmos-sdk/pull/21003) Align block header when query with latest height.

### API Breaking Changes

* (baseapp) [#16244](https://github.com/cosmos/cosmos-sdk/pull/16244) `SetProtocolVersion` has been renamed to `SetAppVersion`. It now updates the consensus params in baseapp's `ParamStore`.
* (types) [#16918](https://github.com/cosmos/cosmos-sdk/pull/16918) Remove `IntProto` and `DecProto`. Instead, `math.Int` and `math.LegacyDec` should be used respectively. Both types support `Marshal` and `Unmarshal` which should be used for binary marshaling.
* (client) [#17215](https://github.com/cosmos/cosmos-sdk/pull/17215) `server.StartCmd`,`server.ExportCmd`,`server.NewRollbackCmd`,`pruning.Cmd`,`genutilcli.InitCmd`,`genutilcli.GenTxCmd`,`genutilcli.CollectGenTxsCmd`,`genutilcli.AddGenesisAccountCmd`, do not take a home directory anymore. It is inferred from the root command.
* (client) [#17259](https://github.com/cosmos/cosmos-sdk/pull/17259) Remove deprecated `clientCtx.PrintObjectLegacy`. Use `clientCtx.PrintProto` or `clientCtx.PrintRaw` instead.
* (types) [#17348](https://github.com/cosmos/cosmos-sdk/pull/17348) Remove the `WrapServiceResult` function.
    * The `*sdk.Result` returned by the msg server router will not contain the `.Data` field. 
* (types) [#17426](https://github.com/cosmos/cosmos-sdk/pull/17426) `NewContext` does not take a `cmtproto.Header{}` any longer.
    * `WithChainID` / `WithBlockHeight` / `WithBlockHeader` must be used to set values on the context
* (client/keys) [#17503](https://github.com/cosmos/cosmos-sdk/pull/17503) `clientkeys.NewKeyOutput`, `MkConsKeyOutput`, `MkValKeyOutput`, `MkAccKeyOutput`, `MkAccKeysOutput` now take their corresponding address codec instead of using the global SDK config.
* (types/simulation) [#17737](https://github.com/cosmos/cosmos-sdk/pull/17737) Remove unused parameter from `RandomFees`
* (types) [#17738](https://github.com/cosmos/cosmos-sdk/pull/17738) `WithBlockTime()` was removed & `BlockTime()` were deprecated in favor of `WithHeaderInfo()` & `HeaderInfo()`. `BlockTime` now gets data from `HeaderInfo()` instead of `BlockHeader()`.
* (client) [#17746](https://github.com/cosmos/cosmos-sdk/pull/17746) `txEncodeAmino` & `txDecodeAmino` txs via grpc and rest were removed
* (app) [#17838](https://github.com/cosmos/cosmos-sdk/pull/17838) Params module was removed from simapp and all imports of the params module removed throughout the repo.
    * The Cosmos SDK has migrated away from using params, if your app still uses it, then you can leave it plugged into your app
* (x/bank/testutil) [#17868](https://github.com/cosmos/cosmos-sdk/pull/17868) `MsgSendExec` has been removed because of AutoCLI migration.
* (types) [#17885](https://github.com/cosmos/cosmos-sdk/pull/17885) `InitGenesis` & `ExportGenesis` now take `context.Context` instead of `sdk.Context`
* (x/gov/testutil) [#17986](https://github.com/cosmos/cosmos-sdk/pull/18036) `MsgDeposit` has been removed because of AutoCLI migration.
* (x/staking/testutil) [#17986](https://github.com/cosmos/cosmos-sdk/pull/17986) `MsgRedelegateExec`, `MsgUnbondExec` has been removed because of AutoCLI migration.
* (x/group) [#17937](https://github.com/cosmos/cosmos-sdk/pull/17937) Groups module was moved to its own go.mod `cosmossdk.io/x/group`
* (x/gov) [#18197](https://github.com/cosmos/cosmos-sdk/pull/18197) Gov module was moved to its own go.mod `cosmossdk.io/x/gov`
* (x/distribution) [#18199](https://github.com/cosmos/cosmos-sdk/pull/18199) Distribution module was moved to its own go.mod `cosmossdk.io/x/distribution`
* (x/slashing) [#18201](https://github.com/cosmos/cosmos-sdk/pull/18201) Slashing module was moved to its own go.mod `cosmossdk.io/x/slashing`
* (x/staking) [#18257](https://github.com/cosmos/cosmos-sdk/pull/18257) Staking module was moved to its own go.mod `cosmossdk.io/x/staking`
* (types) [#18268](https://github.com/cosmos/cosmos-sdk/pull/18268) Remove global setting of basedenom. Use the staking module parameter instead
* (x/authz) [#18265](https://github.com/cosmos/cosmos-sdk/pull/18265) Authz module was moved to its own go.mod `cosmossdk.io/x/authz`
* (x/mint) [#18283](https://github.com/cosmos/cosmos-sdk/pull/18283) Mint module was moved to its own go.mod `cosmossdk.io/x/mint`
* (server) [#18303](https://github.com/cosmos/cosmos-sdk/pull/18303) `x/genutil` now handles the application export. `server.AddCommands` does not take an `AppExporter` but instead `genutilcli.Commands` does.
* (types) [#18372](https://github.com/cosmos/cosmos-sdk/pull/18372) Removed global configuration for coin type and purpose. Setters and getters should be removed and access directly to defined types.
* (types) [#18607](https://github.com/cosmos/cosmos-sdk/pull/18607) Removed address verifier from global config, moved verifier function to bech32 codec.
* (types) [#18695](https://github.com/cosmos/cosmos-sdk/pull/18695) Removed global configuration for txEncoder.
* (server) [#18909](https://github.com/cosmos/cosmos-sdk/pull/18909) Remove configuration endpoint on grpc reflection endpoint in favour of auth module bech32prefix endpoint already exposed.
* (crypto) [#19541](https://github.com/cosmos/cosmos-sdk/pull/19541) The deprecated `FromTmProtoPublicKey`, `ToTmProtoPublicKey`, `FromTmPubKeyInterface` and `ToTmPubKeyInterface` functions have been removed. Use their replacements (`Cmt` instead of `Tm`) instead.
* (types) [#19512](https://github.com/cosmos/cosmos-sdk/pull/19512) Remove basic manager and all related functions (`module.BasicManager`, `module.NewBasicManager`, `module.NewBasicManagerFromManager`, `NewGenesisOnlyAppModule`).
    * The module manager now can do everything that the basic manager was doing.
    * When using runtime, just inject the module manager when needed using your app config.
    * All `AppModuleBasic` structs have been removed.
* (types) [#19627](https://github.com/cosmos/cosmos-sdk/pull/19627) and [#19735](https://github.com/cosmos/cosmos-sdk/pull/19735) All genesis interfaces now don't take `codec.JsonCodec`:
    * Every module has the codec already, passing it created an unneeded dependency.
    * Additionally, to reflect this change, the module manager does not take a codec either.
* (types) [#19652](https://github.com/cosmos/cosmos-sdk/pull/19652) and [#19758](https://github.com/cosmos/cosmos-sdk/pull/19758)
    * Moved`types/module.HasRegisterInterfaces` to `cosmossdk.io/core`.
    * Moved `RegisterInterfaces` and `RegisterImplementations` from `InterfaceRegistry` to `cosmossdk.io/core/registry.InterfaceRegistrar` interface.
* (types) [#19742](https://github.com/cosmos/cosmos-sdk/pull/19742) Removes the use of `Accounts.String`
    * `SimulationState` now has address and validator codecs as fields.
* (runtime) [#19747](https://github.com/cosmos/cosmos-sdk/pull/19747) `runtime.ValidatorAddressCodec` and `runtime.ConsensusAddressCodec` have been moved to `core`.
* (all) [#19726](https://github.com/cosmos/cosmos-sdk/pull/19726) Integrate comet v1
* [#19833](https://github.com/cosmos/cosmos-sdk/pull/19833) Fix some places in which we call Remove inside a Walk.
* [#19839](https://github.com/cosmos/cosmos-sdk/pull/19839) `Tx.GetMsgsV2` has been replaced with `Tx.GetReflectMessages`, and `Codec.GetMsgV1Signers` and `Codec.GetMsgV2Signers` have been replaced with `GetMsgSigners` and `GetReflectMsgSigners` respectively. These API changes clear up confusion as to the use and purpose of these methods.
* [#19851](https://github.com/cosmos/cosmos-sdk/pull/19851) Fix some places in which we call Remove inside a Walk (x/staking and x/gov).
* (server) [#19854](https://github.com/cosmos/cosmos-sdk/pull/19854) Remove `servertypes.ModuleInitFlags` types and from `server.AddCommands` as `StartCmdOptions` already achieves the same goal.
* (x/genutil) [#19926](https://github.com/cosmos/cosmos-sdk/pull/19926) Removal of the `Address.String()` method and related changes:
    * Added an address codec as an argument to `CollectTxs`, `GenAppStateFromConfig`, and `AddGenesisAccount`.
    * Removed the `ValidatorAddressCodec` argument from `CollectGenTxsCmd`, now utilizing the context for this purpose.
    * Changed `ValidateAccountInGenesis` to accept a string instead of an `AccAddress`.
* (baseapp) [#19993](https://github.com/cosmos/cosmos-sdk/pull/19993) Indicate pruning with error code "not found" rather than "invalid request".
* (x/consensus) [#20010](https://github.com/cosmos/cosmos-sdk/pull/20010) Move consensus module to be its own go.mod
* (x/crisis) [#20043](https://github.com/cosmos/cosmos-sdk/pull/20043) Changed `NewMsgVerifyInvariant` to accept a string as argument instead of an `AccAddress`.
* (x/simulation)[#20056](https://github.com/cosmos/cosmos-sdk/pull/20056) `SimulateFromSeed` now takes an address codec as argument.
* (server) [#20140](https://github.com/cosmos/cosmos-sdk/pull/20140) Remove embedded grpc-web proxy in favor of standalone grpc-web proxy. [Envoy Proxy](https://www.envoyproxy.io/docs/envoy/latest/start/start)
* (client) [#20255](https://github.com/cosmos/cosmos-sdk/pull/20255) Use comet proofOp proto type instead of sdk version to avoid needing to translate to later be proven in the merkle proof runtime. 
* (types)[#20369](https://github.com/cosmos/cosmos-sdk/pull/20369) The signature of `HasAminoCodec` has changed to accept a `core/legacy.Amino` interface instead of `codec.LegacyAmino`.
* (server) [#20422](https://github.com/cosmos/cosmos-sdk/pull/20422) Deprecated `ServerContext`. To get `cmtcfg.Config` from cmd, use `client.GetCometConfigFromCmd(cmd)` instead of `server.GetServerContextFromCmd(cmd).Config`
* (x/genutil) [#20740](https://github.com/cosmos/cosmos-sdk/pull/20740) Update `genutilcli.Commands` and `genutilcli.CommandsWithCustomMigrationMap` to take the genesis module and abstract the module manager.
* (types/errors) [#20756](https://github.com/cosmos/cosmos-sdk/pull/20756) Remove `ResponseCheckTxWithEvents`, `ResponseExecTxResultWithEvents` & `QueryResult` from types/errors pkg. They have been moved to `baseapp/errors.go` and made private.
* (client) [#20976](https://github.com/cosmos/cosmos-sdk/pull/20976) Simplified command initialization by removing unnecessary parameters such as `txConfig` and `addressCodec`.
    * Remove parameter `txConfig` from `genutilcli.Commands`,`genutilcli.CommandsWithCustomMigrationMap`,`genutilcli.GenTxCmd`.
    * Remove parameter `addressCodec` from `genutilcli.GenTxCmd`,`genutilcli.AddGenesisAccountCmd`,`stakingcli.BuildCreateValidatorMsg`.
* (sims) [#21039](https://github.com/cosmos/cosmos-sdk/pull/21039): Remove Baseapp from sims by a new interface `simtypes.AppEntrypoint`.
* (x/genutil) [#21372](https://github.com/cosmos/cosmos-sdk/pull/21372) Remove `AddGenesisAccount` for `AddGenesisAccounts`.
* (baseapp) [#21413](https://github.com/cosmos/cosmos-sdk/pull/21413) Add `SelectBy` method to `Mempool` interface, which is thread-safe to use.
* (types/mempool) [#21744](https://github.com/cosmos/cosmos-sdk/pull/21744) Update types/mempool.Mempool interface to take decoded transactions. This avoid to decode the transaction twice.
* (x/auth/tx/config)  [#21822](https://github.com/cosmos/cosmos-sdk/pull/21822) Sign mode textual is no more automatically added to tx config when using runtime. Should be added manually on the server side.
* (x/auth/tx/config)  [#21822](https://github.com/cosmos/cosmos-sdk/pull/21822) This depinject module now only provide txconfig and tx config options. `x/validate` now handles the providing of ante/post handlers, alongside tx validators for v2. The corresponding app config options have been removed from the depinject module config.

### Client Breaking Changes

* (runtime) [#19040](https://github.com/cosmos/cosmos-sdk/pull/19040) Simplify app config implementation and deprecate `/cosmos/app/v1alpha1/config` query. 

### CLI Breaking Changes

* (server) [#18303](https://github.com/cosmos/cosmos-sdk/pull/18303) `appd export` has moved with other genesis commands, use `appd genesis export` instead.
* (perf)[#20490](https://github.com/cosmos/cosmos-sdk/pull/20490) Sims: Replace runsim command with Go stdlib testing. CLI: `Commit` default true, `Lean`, `SimulateEveryOperation`, `PrintAllInvariants`, `DBBackend` params removed
* (client/tx) [#20870](https://github.com/cosmos/cosmos-sdk/pull/20870) Removed `timeout-height` flag replace with `timeout-timestamp` flag for a time based timeout.

### Deprecated

* (simapp) [#19146](https://github.com/cosmos/cosmos-sdk/pull/19146) Replace `--v` CLI option with `--validator-count`/`-n`.
* (module) [#19370](https://github.com/cosmos/cosmos-sdk/pull/19370) Deprecate `module.Configurator`, use `appmodule.HasMigrations` and `appmodule.HasServices` instead from Core API.
* (types) [#21435](https://github.com/cosmos/cosmos-sdk/pull/21435) The `String()` method on `AccAddress`, `ValAddress` and `ConsAddress` have been deprecated. This is done because those are still using the deprecated global `sdk.Config`. Use an `address.Codec` instead.

## Previous Versions

[CHANGELOG of previous versions](https://github.com/cosmos/cosmos-sdk/blob/main/CHANGELOG.md#v0509---2024-08-07).
