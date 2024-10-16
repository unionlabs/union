package app

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	reflectionv1 "cosmossdk.io/api/cosmos/reflection/v1"
	"cosmossdk.io/client/v2/autocli"
	corestore "cosmossdk.io/core/store"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/accounts"
	"cosmossdk.io/x/authz"
	authzkeeper "cosmossdk.io/x/authz/keeper"
	authzmodule "cosmossdk.io/x/authz/module"
	"cosmossdk.io/x/bank"
	bankkeeper "cosmossdk.io/x/bank/keeper"
	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/consensus"
	consensusparamkeeper "cosmossdk.io/x/consensus/keeper"
	consensusparamtypes "cosmossdk.io/x/consensus/types"
	distr "cosmossdk.io/x/distribution"
	distrkeeper "cosmossdk.io/x/distribution/keeper"
	distrtypes "cosmossdk.io/x/distribution/types"
	"cosmossdk.io/x/evidence"
	evidencekeeper "cosmossdk.io/x/evidence/keeper"
	evidencetypes "cosmossdk.io/x/evidence/types"
	"cosmossdk.io/x/feegrant"
	feegrantkeeper "cosmossdk.io/x/feegrant/keeper"
	feegrantmodule "cosmossdk.io/x/feegrant/module"
	"cosmossdk.io/x/gov"
	govkeeper "cosmossdk.io/x/gov/keeper"
	govtypes "cosmossdk.io/x/gov/types"
	govv1beta1 "cosmossdk.io/x/gov/types/v1beta1"
	"cosmossdk.io/x/group"
	groupkeeper "cosmossdk.io/x/group/keeper"
	groupmodule "cosmossdk.io/x/group/module"
	"cosmossdk.io/x/mint"
	mintkeeper "cosmossdk.io/x/mint/keeper"
	minttypes "cosmossdk.io/x/mint/types"
	"cosmossdk.io/x/params"
	paramskeeper "cosmossdk.io/x/params/keeper"
	paramstypes "cosmossdk.io/x/params/types"
	paramproposal "cosmossdk.io/x/params/types/proposal"
	"cosmossdk.io/x/protocolpool"
	poolkeeper "cosmossdk.io/x/protocolpool/keeper"
	pooltypes "cosmossdk.io/x/protocolpool/types"
	"cosmossdk.io/x/slashing"
	slashingkeeper "cosmossdk.io/x/slashing/keeper"
	slashingtypes "cosmossdk.io/x/slashing/types"
	"cosmossdk.io/x/staking"
	stakingkeeper "cosmossdk.io/x/staking/keeper"
	stakingtypes "cosmossdk.io/x/staking/types"
	"cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/upgrade"
	upgradekeeper "cosmossdk.io/x/upgrade/keeper"
	upgradetypes "cosmossdk.io/x/upgrade/types"

	"github.com/CosmWasm/wasmd/x/wasm"
	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	wasmvm "github.com/CosmWasm/wasmvm/v2"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmtcrypto "github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/bn254"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/grpc/cmtservice"
	nodeservice "github.com/cosmos/cosmos-sdk/client/grpc/node"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	runtimeservices "github.com/cosmos/cosmos-sdk/runtime/services"
	"github.com/cosmos/cosmos-sdk/server"
	"github.com/cosmos/cosmos-sdk/server/api"
	"github.com/cosmos/cosmos-sdk/server/config"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/std"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	sigtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/cosmos-sdk/x/auth"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	authcodec "github.com/cosmos/cosmos-sdk/x/auth/codec"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	authsims "github.com/cosmos/cosmos-sdk/x/auth/simulation"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
	txmodule "github.com/cosmos/cosmos-sdk/x/auth/tx/config"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting"
	vestingtypes "github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"

	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/ibc-go/modules/capability"
	capabilitykeeper "github.com/cosmos/ibc-go/modules/capability/keeper"
	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"

	ibcwasm "github.com/cosmos/ibc-go/modules/light-clients/08-wasm"
	ibcwasmkeeper "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/keeper"
	ibcwasmtypes "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"

	ica "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts"
	icacontrollerkeeper "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/keeper"
	icacontrollertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	icahost "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host"
	icahostkeeper "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/keeper"
	icahosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	ibcfee "github.com/cosmos/ibc-go/v8/modules/apps/29-fee"
	ibcfeekeeper "github.com/cosmos/ibc-go/v8/modules/apps/29-fee/keeper"
	ibcfeetypes "github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/transfer"
	ibctransferkeeper "github.com/cosmos/ibc-go/v8/modules/apps/transfer/keeper"
	ibctransfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	ibc "github.com/cosmos/ibc-go/v8/modules/core"
	ibcclienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	ibcconnectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	ibcporttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibckeeper "github.com/cosmos/ibc-go/v8/modules/core/keeper"
	solomachine "github.com/cosmos/ibc-go/v8/modules/light-clients/06-solomachine"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"

	"github.com/spf13/cast"

	unioncustomquery "union/app/custom_query"
	ibcunion "union/app/ibc"
	ibccometblsclient "union/app/ibc/cometbls/02-client/keeper"
	"union/docs"
	unionstaking "union/x/staking"
	tfmodule "union/x/tokenfactory"
	tfbindings "union/x/tokenfactory/bindings"
	tfkeeper "union/x/tokenfactory/keeper"
	tftypes "union/x/tokenfactory/types"
)

const (
	AccountAddressPrefix = "union"
	Name                 = "union"
)

var (
	// DefaultNodeHome default home directories for the application daemon
	DefaultNodeHome string

	// If EnabledSpecificProposals is "", and this is "true", then enable all x/wasm proposals.
	// If EnabledSpecificProposals is "", and this is not "true", then disable all x/wasm proposals.
	ProposalsEnabled = "true"
	// If set to non-empty string it must be comma-separated list of values that are all a subset
	// of "EnableAllProposals" (takes precedence over ProposalsEnabled)
	// https://github.com/CosmWasm/wasmd/blob/02a54d33ff2c064f3539ae12d75d027d9c665f05/x/wasm/internal/types/proposal.go#L28-L34
	EnableSpecificProposals = ""

	// module account permissions
	maccPerms = map[string][]string{
		authtypes.FeeCollectorName:         nil,
		distrtypes.ModuleName:              nil,
		pooltypes.ModuleName:               nil,
		pooltypes.StreamAccount:            nil,
		pooltypes.ProtocolPoolDistrAccount: nil,
		icatypes.ModuleName:                nil,
		minttypes.ModuleName:               {authtypes.Minter},
		stakingtypes.BondedPoolName:        {authtypes.Burner, authtypes.Staking},
		stakingtypes.NotBondedPoolName:     {authtypes.Burner, authtypes.Staking},
		govtypes.ModuleName:                {authtypes.Burner},
		ibctransfertypes.ModuleName:        {authtypes.Minter, authtypes.Burner},
		ibcfeetypes.ModuleName:             nil,
		wasmtypes.ModuleName:               {authtypes.Burner}, // TODO(aeryz): is this necessary?
		tftypes.ModuleName:                 {authtypes.Minter, authtypes.Burner},
	}
)

// contractMemoryLimit is the memory limit of each contract execution (in MiB)
// constant value so all nodes run with the same limit.
const contractMemoryLimit = 32

var (
	_ servertypes.Application = (*UnionApp)(nil)
)

func init() {
	userHomeDir, err := os.UserHomeDir()
	if err != nil {
		panic(err)
	}

	DefaultNodeHome = filepath.Join(userHomeDir, "."+Name)
}

// UnionApp extends an ABCI application, but with most of its parameters exported.
// They are exported for convenience in creating helper functions, as object
// capabilities aren't needed for testing.
type UnionApp struct {
	*baseapp.BaseApp

	logger            log.Logger
	legacyAmino       *codec.LegacyAmino
	appCodec          codec.Codec
	interfaceRegistry types.InterfaceRegistry
	txConfig          client.TxConfig

	// keys to access the substores
	keys    map[string]*storetypes.KVStoreKey
	tkeys   map[string]*storetypes.TransientStoreKey
	memKeys map[string]*storetypes.MemoryStoreKey

	// keepers
	AccountKeeper         accounts.Keeper
	AuthKeeper            authkeeper.AccountKeeper
	BankKeeper            bankkeeper.BaseKeeper
	CapabilityKeeper      *capabilitykeeper.Keeper
	StakingKeeper         *stakingkeeper.Keeper
	SlashingKeeper        slashingkeeper.Keeper
	MintKeeper            mintkeeper.Keeper
	DistrKeeper           distrkeeper.Keeper
	GovKeeper             govkeeper.Keeper
	UpgradeKeeper         *upgradekeeper.Keeper
	AuthzKeeper           authzkeeper.Keeper
	ParamsKeeper          paramskeeper.Keeper
	IBCKeeper             *ibckeeper.Keeper // IBC Keeper must be a pointer in the app, so we can SetRouter on it correctly
	IBCFeeKeeper          ibcfeekeeper.Keeper
	EvidenceKeeper        evidencekeeper.Keeper
	TransferKeeper        ibctransferkeeper.Keeper
	WasmClientKeeper      ibcwasmkeeper.Keeper
	ICAHostKeeper         icahostkeeper.Keeper
	FeeGrantKeeper        feegrantkeeper.Keeper
	GroupKeeper           groupkeeper.Keeper
	ConsensusParamsKeeper consensusparamkeeper.Keeper
	WasmKeeper            wasmkeeper.Keeper
	TfKeeper              tfkeeper.Keeper
	PoolKeeper            poolkeeper.Keeper

	// make scoped keepers public for test purposes
	ScopedIBCKeeper      capabilitykeeper.ScopedKeeper
	ScopedTransferKeeper capabilitykeeper.ScopedKeeper
	ScopedICAHostKeeper  capabilitykeeper.ScopedKeeper
	ScopedIBCFeeKeeper   capabilitykeeper.ScopedKeeper
	ScopedWasmKeeper     capabilitykeeper.ScopedKeeper

	ModuleManager *module.Manager

	simulationManager *module.SimulationManager
	configurator      module.Configurator //nolint:staticcheck // SA1019: Configurator is deprecated but still used in runtime v1.
}

// New returns a reference to an initialized blockchain app
func NewUnionApp(
	logger log.Logger,
	db corestore.KVStoreWithBatch,
	traceStore io.Writer,
	loadLatest bool,
	appOpts servertypes.AppOptions,
	wasmOpts []wasmkeeper.Option,
	baseAppOptions ...func(*baseapp.BaseApp),
) *UnionApp {
	interfaceRegistry, _ := types.NewInterfaceRegistryWithOptions(types.InterfaceRegistryOptions{
		ProtoFiles: proto.HybridResolver,
		SigningOptions: signing.Options{
			AddressCodec: address.Bech32Codec{
				Bech32Prefix: sdk.GetConfig().GetBech32AccountAddrPrefix(),
			},
			ValidatorAddressCodec: address.Bech32Codec{
				Bech32Prefix: sdk.GetConfig().GetBech32ValidatorAddrPrefix(),
			},
		},
	})
	appCodec := codec.NewProtoCodec(interfaceRegistry)
	legacyAmino := codec.NewLegacyAmino()
	signingCtx := interfaceRegistry.SigningContext()
	txConfig := authtx.NewTxConfig(appCodec, signingCtx.AddressCodec(), signingCtx.ValidatorAddressCodec(), authtx.DefaultSignModes)

	govModuleAddr, err := signingCtx.AddressCodec().BytesToString(authtypes.NewModuleAddress(govtypes.ModuleName))
	if err != nil {
		panic(err)
	}

	if err := signingCtx.Validate(); err != nil {
		panic(err)
	}

	std.RegisterLegacyAminoCodec(legacyAmino)
	std.RegisterInterfaces(interfaceRegistry)

	bApp := baseapp.NewBaseApp(
		Name,
		logger,
		db,
		txConfig.TxDecoder(),
		baseAppOptions...,
	)
	bApp.SetCommitMultiStoreTracer(traceStore)
	bApp.SetVersion(version.Version)
	bApp.SetInterfaceRegistry(interfaceRegistry)
	bApp.SetTxEncoder(txConfig.TxEncoder())

	keys := storetypes.NewKVStoreKeys(
		authtypes.StoreKey, authz.ModuleName, banktypes.StoreKey, stakingtypes.StoreKey,
		minttypes.StoreKey, distrtypes.StoreKey, slashingtypes.StoreKey,
		govtypes.StoreKey, paramstypes.StoreKey, ibcexported.StoreKey, upgradetypes.StoreKey,
		feegrant.StoreKey, evidencetypes.StoreKey, ibctransfertypes.StoreKey, ibcwasmtypes.StoreKey,
		icahosttypes.StoreKey, capabilitytypes.StoreKey, group.StoreKey, icacontrollertypes.StoreKey,
		consensusparamtypes.StoreKey, ibcfeetypes.StoreKey, wasmtypes.StoreKey, tftypes.StoreKey,
		pooltypes.StoreKey, accounts.StoreKey, ibcunion.StoreKey,
	)

	// register streaming services
	if err := bApp.RegisterStreamingServices(appOpts, keys); err != nil {
		panic(err)
	}

	tkeys := storetypes.NewTransientStoreKeys(paramstypes.TStoreKey)
	memKeys := storetypes.NewMemoryStoreKeys(capabilitytypes.MemStoreKey, tftypes.MemStoreKey)

	app := &UnionApp{
		BaseApp:           bApp,
		logger:            logger,
		legacyAmino:       legacyAmino,
		appCodec:          appCodec,
		interfaceRegistry: interfaceRegistry,
		txConfig:          txConfig,
		keys:              keys,
		tkeys:             tkeys,
		memKeys:           memKeys,
	}
	cometService := runtime.NewContextAwareCometInfoService()

	app.ParamsKeeper = initParamsKeeper(
		appCodec,
		legacyAmino,
		keys[paramstypes.StoreKey],
		tkeys[paramstypes.TStoreKey],
	)

	// set the BaseApp's parameter store
	app.ConsensusParamsKeeper = consensusparamkeeper.NewKeeper(
		appCodec,
		runtime.NewEnvironment(runtime.NewKVStoreService(keys[consensusparamtypes.StoreKey]), logger.With(log.ModuleKey, "x/consensus")),
		govModuleAddr,
	)
	bApp.SetParamStore(app.ConsensusParamsKeeper.ParamsStore)

	// add capability keeper and ScopeToModule for ibc module
	app.CapabilityKeeper = capabilitykeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(keys[capabilitytypes.StoreKey]),
		memKeys[capabilitytypes.MemStoreKey],
	)

	// grant capabilities for the ibc and ibc-transfer modules
	scopedIBCKeeper := app.CapabilityKeeper.ScopeToModule(ibcexported.ModuleName)
	scopedICAControllerKeeper := app.CapabilityKeeper.ScopeToModule(icacontrollertypes.SubModuleName)
	scopedTransferKeeper := app.CapabilityKeeper.ScopeToModule(ibctransfertypes.ModuleName)
	scopedICAHostKeeper := app.CapabilityKeeper.ScopeToModule(icahosttypes.SubModuleName)
	scopedWasmKeeper := app.CapabilityKeeper.ScopeToModule(wasmtypes.ModuleName)

	// add keepers
	accountsKeeper, err := accounts.NewKeeper(
		appCodec,
		runtime.NewEnvironment(runtime.NewKVStoreService(keys[accounts.StoreKey]), logger.With(log.ModuleKey, "x/accounts"), runtime.EnvWithMsgRouterService(app.MsgServiceRouter()), runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())),
		signingCtx.AddressCodec(),
		appCodec.InterfaceRegistry(),
	)
	if err != nil {
		panic(err)
	}
	app.AccountKeeper = accountsKeeper

	app.AuthKeeper = authkeeper.NewAccountKeeper(runtime.NewEnvironment(runtime.NewKVStoreService(keys[authtypes.StoreKey]), logger.With(log.ModuleKey, "x/auth")), appCodec, authtypes.ProtoBaseAccount, accountsKeeper, maccPerms, signingCtx.AddressCodec(), sdk.Bech32MainPrefix, govModuleAddr)

	app.AuthzKeeper = authzkeeper.NewKeeper(runtime.NewEnvironment(runtime.NewKVStoreService(keys[authzkeeper.StoreKey]), logger.With(log.ModuleKey, "x/authz"), runtime.EnvWithMsgRouterService(app.MsgServiceRouter()), runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())), appCodec, app.AuthKeeper)

	blockedAddrs := app.BlockedModuleAccountAddrs()

	app.BankKeeper = bankkeeper.NewBaseKeeper(
		runtime.NewEnvironment(runtime.NewKVStoreService(keys[banktypes.StoreKey]), logger.With(log.ModuleKey, "x/bank")),
		appCodec,
		app.AuthKeeper,
		blockedAddrs,
		govModuleAddr,
	)

	// enable sign mode textual by overwriting the default tx config (after setting the bank keeper)
	enabledSignModes := append(authtx.DefaultSignModes, sigtypes.SignMode_SIGN_MODE_TEXTUAL)
	txConfigOpts := authtx.ConfigOptions{
		EnabledSignModes:           enabledSignModes,
		TextualCoinMetadataQueryFn: txmodule.NewBankKeeperCoinMetadataQueryFn(app.BankKeeper),
		SigningOptions: &signing.Options{
			AddressCodec:          signingCtx.AddressCodec(),
			ValidatorAddressCodec: signingCtx.ValidatorAddressCodec(),
		},
	}
	txConfig, err = authtx.NewTxConfigWithOptions(
		appCodec,
		txConfigOpts,
	)
	if err != nil {
		panic(err)
	}
	app.txConfig = txConfig

	app.StakingKeeper = stakingkeeper.NewKeeper(
		appCodec,
		runtime.NewEnvironment(
			runtime.NewKVStoreService(keys[stakingtypes.StoreKey]),
			logger.With(log.ModuleKey, "x/staking"),
			runtime.EnvWithMsgRouterService(app.MsgServiceRouter()),
			runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())),
		app.AuthKeeper,
		app.BankKeeper,
		app.ConsensusParamsKeeper,
		govModuleAddr,
		signingCtx.ValidatorAddressCodec(),
		authcodec.NewBech32Codec(sdk.Bech32PrefixConsAddr),
		cometService,
	)

	app.FeeGrantKeeper = feegrantkeeper.NewKeeper(runtime.NewEnvironment(runtime.NewKVStoreService(keys[feegrant.StoreKey]), logger.With(log.ModuleKey, "x/feegrant")), appCodec, app.AuthKeeper)

	app.MintKeeper = mintkeeper.NewKeeper(appCodec, runtime.NewEnvironment(runtime.NewKVStoreService(keys[minttypes.StoreKey]), logger.With(log.ModuleKey, "x/mint")), app.StakingKeeper, app.AuthKeeper, app.BankKeeper, authtypes.FeeCollectorName, govModuleAddr)

	app.PoolKeeper = poolkeeper.NewKeeper(appCodec, runtime.NewEnvironment(runtime.NewKVStoreService(keys[pooltypes.StoreKey]), logger.With(log.ModuleKey, "x/protocolpool")), app.AuthKeeper, app.BankKeeper, app.StakingKeeper, govModuleAddr)

	app.DistrKeeper = distrkeeper.NewKeeper(appCodec, runtime.NewEnvironment(runtime.NewKVStoreService(keys[distrtypes.StoreKey]), logger.With(log.ModuleKey, "x/distribution")), app.AuthKeeper, app.BankKeeper, app.StakingKeeper, cometService, authtypes.FeeCollectorName, govModuleAddr)

	app.SlashingKeeper = slashingkeeper.NewKeeper(runtime.NewEnvironment(runtime.NewKVStoreService(keys[slashingtypes.StoreKey]), logger.With(log.ModuleKey, "x/slashing")),
		appCodec, legacyAmino, app.StakingKeeper, govModuleAddr,
	)

	groupConfig := group.DefaultConfig()
	app.GroupKeeper = groupkeeper.NewKeeper(runtime.NewEnvironment(runtime.NewKVStoreService(keys[group.StoreKey]), logger.With(log.ModuleKey, "x/group"), runtime.EnvWithMsgRouterService(app.MsgServiceRouter()), runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())), appCodec, app.AuthKeeper, groupConfig)

	skipUpgradeHeights := map[int64]bool{}
	for _, h := range cast.ToIntSlice(appOpts.Get(server.FlagUnsafeSkipUpgrades)) {
		skipUpgradeHeights[int64(h)] = true
	}
	homePath := cast.ToString(appOpts.Get(flags.FlagHome))

	app.UpgradeKeeper = upgradekeeper.NewKeeper(
		runtime.NewEnvironment(runtime.NewKVStoreService(keys[upgradetypes.StoreKey]), logger.With(log.ModuleKey, "x/upgrade"), runtime.EnvWithMsgRouterService(app.MsgServiceRouter()), runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())),
		skipUpgradeHeights, appCodec, homePath, app.BaseApp, govModuleAddr, app.ConsensusParamsKeeper)

	doubleCommitStore := ibcunion.NewDoubleCommitStoreService(
		appCodec,
		keys[ibcunion.StoreKey],
		keys[ibcexported.StoreKey],
	)

	// Create IBC Keeper
	ibcKeeper := ibckeeper.NewKeeper(
		appCodec,
		doubleCommitStore,
		app.GetSubspace(ibcexported.ModuleName),
		app.StakingKeeper,
		app.UpgradeKeeper,
		scopedIBCKeeper,
		authtypes.NewModuleAddress(govtypes.ModuleName).String(),
	)

	/*
	 We create an intermediate client keeper called cometbls client because
	 the connection handshake does a `ValidateSelfClient` which downcast the
	 client state to tendermint client state by default. This was blocking
	 the ConnectionOpenTry to succeed.
	*/
	ibcCometblsClient := ibccometblsclient.NewConsensusHost(appCodec, app.StakingKeeper)
	ibcKeeper.SetConsensusHost(ibcCometblsClient)
	app.IBCKeeper = ibcKeeper

	// IBC Fee Module keeper
	app.IBCFeeKeeper = ibcfeekeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(keys[ibcfeetypes.StoreKey]),
		app.IBCKeeper.ChannelKeeper, // may be replaced with IBC middleware
		app.IBCKeeper.ChannelKeeper,
		app.IBCKeeper.PortKeeper,
		app.AuthKeeper,
		app.BankKeeper,
	)

	// Create Transfer Keepers
	app.TransferKeeper = ibctransferkeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(keys[ibctransfertypes.StoreKey]),
		app.GetSubspace(ibctransfertypes.ModuleName),
		app.IBCFeeKeeper, // ISC4 Wrapper: fee IBC middleware
		app.IBCKeeper.ChannelKeeper,
		app.IBCKeeper.PortKeeper,
		app.AuthKeeper,
		app.BankKeeper,
		scopedTransferKeeper,
		authtypes.NewModuleAddress(govtypes.ModuleName).String(),
	)

	transferModule := transfer.NewAppModule(appCodec, app.TransferKeeper)
	var transferIBCModule ibcporttypes.IBCModule
	transferIBCModule = transfer.NewIBCModule(app.TransferKeeper)
	transferIBCModule = ibcunion.NewDoubleCommitMiddleware(
		app.IBCKeeper,
		transferIBCModule,
		app.IBCFeeKeeper,
		doubleCommitStore,
	)

	app.ICAHostKeeper = icahostkeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(keys[icahosttypes.StoreKey]),
		app.GetSubspace(icahosttypes.SubModuleName),
		app.IBCFeeKeeper, // use ics29 fee as ics4Wrapper in middleware stack
		app.IBCKeeper.ChannelKeeper,
		app.IBCKeeper.PortKeeper,
		app.AuthKeeper,
		scopedICAHostKeeper,
		app.MsgServiceRouter(),
		authtypes.NewModuleAddress(govtypes.ModuleName).String(),
	)
	app.ICAHostKeeper.WithQueryRouter(app.GRPCQueryRouter())

	icaControllerKeeper := icacontrollerkeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(keys[icacontrollertypes.StoreKey]),
		app.GetSubspace(icacontrollertypes.SubModuleName),
		app.IBCFeeKeeper, // use ics29 fee as ics4Wrapper in middleware stack
		app.IBCKeeper.ChannelKeeper,
		app.IBCKeeper.PortKeeper,
		scopedICAControllerKeeper,
		app.MsgServiceRouter(),
		authtypes.NewModuleAddress(govtypes.ModuleName).String(),
	)

	icaModule := ica.NewAppModule(appCodec, &icaControllerKeeper, &app.ICAHostKeeper)
	var icaHostIBCModule ibcporttypes.IBCModule
	icaHostIBCModule = icahost.NewIBCModule(app.ICAHostKeeper)
	icaHostIBCModule = ibcunion.NewDoubleCommitMiddleware(
		app.IBCKeeper,
		icaHostIBCModule,
		app.IBCFeeKeeper,
		doubleCommitStore,
	)

	// Create evidence Keeper for to register the IBC light client misbehaviour evidence route
	evidenceKeeper := evidencekeeper.NewKeeper(
		appCodec,
		runtime.NewEnvironment(runtime.NewKVStoreService(keys[evidencetypes.StoreKey]), logger.With(log.ModuleKey, "x/evidence"), runtime.EnvWithMsgRouterService(app.MsgServiceRouter()), runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())),
		app.StakingKeeper,
		app.SlashingKeeper,
		app.ConsensusParamsKeeper,
		app.AuthKeeper.AddressCodec(),
	)
	// If evidence needs to be handled for the app, set routes in router here and seal
	app.EvidenceKeeper = *evidenceKeeper

	// TODO(aeryz): seems like you dont add specific handlers anymore but lets make sure
	govRouter := govv1beta1.NewRouter()
	govRouter.
		AddRoute(govtypes.RouterKey, govv1beta1.ProposalHandler).
		AddRoute(paramproposal.RouterKey, params.NewParamChangeProposalHandler(app.ParamsKeeper))

	app.TfKeeper = tfkeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(keys[tftypes.StoreKey]),
		app.GetSubspace(tftypes.ModuleName),
		app.AuthKeeper,
		app.BankKeeper,
		app.PoolKeeper,
	)
	tfModule := tfmodule.NewAppModule(
		appCodec,
		app.TfKeeper,
		app.AuthKeeper,
		app.BankKeeper,
	)

	wasmOpts = append(wasmOpts, tfbindings.RegisterCustomPlugins(&app.BankKeeper, &app.TfKeeper)...)

	wasmDir := filepath.Join(homePath, "wasm")
	wasmConfig, err := wasm.ReadWasmConfig(appOpts)
	if err != nil {
		panic(fmt.Sprintf("error while reading wasm config: %s", err))
	}

	wasmer, err := wasmvm.NewVM(
		wasmDir,
		AllCapabilities(),
		contractMemoryLimit,
		wasmConfig.ContractDebugMode,
		wasmConfig.MemoryCacheSize,
	)
	if err != nil {
		panic(err)
	}
	wasmOpts = append(wasmOpts, wasmkeeper.WithWasmEngine(wasmer))

	app.WasmKeeper = wasmkeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(keys[wasmtypes.StoreKey]),
		app.AuthKeeper,
		app.BankKeeper,
		app.StakingKeeper,
		distrkeeper.NewQuerier(app.DistrKeeper),
		app.IBCFeeKeeper, // ISC4 Wrapper: fee IBC middleware
		app.IBCKeeper.ChannelKeeper,
		app.IBCKeeper.PortKeeper,
		scopedWasmKeeper,
		app.TransferKeeper,
		app.MsgServiceRouter(),
		app.GRPCQueryRouter(),
		wasmDir,
		wasmConfig,
		AllCapabilities(),
		authtypes.NewModuleAddress(govtypes.ModuleName).String(),
		wasmOpts...,
	)

	querierOption := ibcwasmkeeper.WithQueryPlugins(&ibcwasmtypes.QueryPlugins{
		Custom: unioncustomquery.CustomQuerier(),
		Stargate: ibcwasmtypes.AcceptListStargateQuerier([]string{
			"/cosmos.base.tendermint.v1beta1.Service/ABCIQuery",
		}),
	})
	app.WasmClientKeeper = ibcwasmkeeper.NewKeeperWithVM(
		appCodec,
		runtime.NewKVStoreService(keys[ibcwasmtypes.StoreKey]),
		ibcKeeper.ClientKeeper,
		authtypes.NewModuleAddress(govtypes.ModuleName).String(),
		wasmer,
		app.GRPCQueryRouter(),
		querierOption,
	)

	govConfig := govkeeper.DefaultConfig()
	govKeeper := govkeeper.NewKeeper(appCodec, runtime.NewEnvironment(runtime.NewKVStoreService(keys[govtypes.StoreKey]), logger.With(log.ModuleKey, "x/gov"), runtime.EnvWithMsgRouterService(app.MsgServiceRouter()), runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())), app.AuthKeeper, app.BankKeeper, app.StakingKeeper, app.PoolKeeper, govConfig, govModuleAddr)

	// Set legacy router for backwards compatibility with gov v1beta1
	govKeeper.SetLegacyRouter(govRouter)
	app.GovKeeper = *govKeeper.SetHooks(
		govtypes.NewMultiGovHooks(
		// register the governance hooks
		),
	)

	/**** IBC Routing ****/

	// Sealing prevents other modules from creating scoped sub-keepers
	app.CapabilityKeeper.Seal()

	// Create static IBC router, add transfer route, then set and seal it
	var wasmStack ibcporttypes.IBCModule
	wasmStack = wasm.NewIBCHandler(app.WasmKeeper, app.IBCKeeper.ChannelKeeper, app.IBCFeeKeeper)
	wasmStack = ibcfee.NewIBCMiddleware(wasmStack, app.IBCFeeKeeper)
	wasmStack = ibcunion.NewDoubleCommitMiddleware(
		app.IBCKeeper,
		wasmStack,
		app.IBCFeeKeeper,
		doubleCommitStore,
	)

	ibcRouter := ibcporttypes.NewRouter()
	ibcRouter.AddRoute(icahosttypes.SubModuleName, icaHostIBCModule).
		AddRoute(wasmtypes.ModuleName, wasmStack).
		AddRoute(ibctransfertypes.ModuleName, transferIBCModule)
	// this line is used by starport scaffolding # ibc/app/router
	app.IBCKeeper.SetRouter(ibcRouter)

	app.setupUpgradeStoreLoaders()

	/**** Module Options ****/

	/**** Module Hooks ****/

	// register hooks after all modules have been initialized
	unionStaking := unionstaking.NewMsgServerImpl(app.BaseApp, stakingkeeper.NewMsgServerImpl(app.StakingKeeper))

	unionstaking.RegisterInterfaces(interfaceRegistry)
	unionstaking.RegisterMsgServer(app.MsgServiceRouter(), unionStaking)

	app.StakingKeeper.SetHooks(
		stakingtypes.NewMultiStakingHooks(
			// insert staking hooks receivers here
			app.DistrKeeper.Hooks(),
			app.SlashingKeeper.Hooks(),
			unionStaking.StakingHooks,
		),
	)

	// NOTE: Any module instantiated in the module manager that is later modified
	// must be passed by reference here.

	app.ModuleManager = module.NewManager(
		genutil.NewAppModule(appCodec, app.AuthKeeper, app.StakingKeeper, app, txConfig, genutiltypes.DefaultMessageValidator),
		accounts.NewAppModule(appCodec, app.AccountKeeper),
		auth.NewAppModule(appCodec, app.AuthKeeper, app.AccountKeeper, authsims.RandomGenesisAccounts, nil),
		authzmodule.NewAppModule(appCodec, app.AuthzKeeper, app.interfaceRegistry),
		vesting.NewAppModule(app.AuthKeeper, app.BankKeeper),
		bank.NewAppModule(appCodec, app.BankKeeper, app.AuthKeeper),
		capability.NewAppModule(appCodec, *app.CapabilityKeeper, false),
		feegrantmodule.NewAppModule(appCodec, app.FeeGrantKeeper, app.interfaceRegistry),
		groupmodule.NewAppModule(appCodec, app.GroupKeeper, app.AuthKeeper, app.BankKeeper, app.interfaceRegistry),
		gov.NewAppModule(appCodec, &app.GovKeeper, app.AuthKeeper, app.BankKeeper, app.PoolKeeper),
		mint.NewAppModule(appCodec, app.MintKeeper, app.AuthKeeper, nil),
		slashing.NewAppModule(appCodec, app.SlashingKeeper, app.AuthKeeper, app.BankKeeper, app.StakingKeeper, app.interfaceRegistry, cometService),
		distr.NewAppModule(appCodec, app.DistrKeeper, app.StakingKeeper),
		staking.NewAppModule(appCodec, app.StakingKeeper),
		upgrade.NewAppModule(app.UpgradeKeeper),
		evidence.NewAppModule(appCodec, app.EvidenceKeeper, cometService),
		consensus.NewAppModule(appCodec, app.ConsensusParamsKeeper),
		protocolpool.NewAppModule(appCodec, app.PoolKeeper, app.AuthKeeper, app.BankKeeper),
		ibc.NewAppModule(appCodec, app.IBCKeeper),
		params.NewAppModule(app.ParamsKeeper),
		transferModule,
		ibcwasm.NewAppModule(appCodec, app.WasmClientKeeper),
		wasm.NewAppModule(appCodec, &app.WasmKeeper, app.StakingKeeper, app.AuthKeeper, app.BankKeeper, app.MsgServiceRouter(), app.GetSubspace(wasmtypes.ModuleName)),
		icaModule,
		tfModule,
		ibctm.NewAppModule(),
		solomachine.NewAppModule(),
	)

	ibccometblsclient.RegisterInterfaces(interfaceRegistry)
	app.ModuleManager.RegisterLegacyAminoCodec(legacyAmino)
	app.ModuleManager.RegisterInterfaces(interfaceRegistry)

	// NOTE: upgrade module is required to be prioritized
	app.ModuleManager.SetOrderPreBlockers(
		upgradetypes.ModuleName,
	)

	// During begin block slashing happens after distr.BeginBlocker so that
	// there is nothing left over in the validator fee pool, so as to keep the
	// CanWithdrawInvariant invariant.
	// NOTE: staking module is required if HistoricalEntries param > 0
	app.ModuleManager.SetOrderBeginBlockers(
		capabilitytypes.ModuleName,
		minttypes.ModuleName,
		distrtypes.ModuleName,
		pooltypes.ModuleName,
		slashingtypes.ModuleName,
		evidencetypes.ModuleName,
		stakingtypes.ModuleName,
		authtypes.ModuleName,
		banktypes.ModuleName,
		govtypes.ModuleName,
		ibctransfertypes.ModuleName,
		ibcexported.ModuleName,
		icatypes.ModuleName,
		genutiltypes.ModuleName,
		authz.ModuleName,
		feegrant.ModuleName,
		ibcwasmtypes.ModuleName,
		group.ModuleName,
		paramstypes.ModuleName,
		vestingtypes.ModuleName,
		consensusparamtypes.ModuleName,
		wasmtypes.ModuleName,
		tftypes.ModuleName,
	)

	app.ModuleManager.SetOrderEndBlockers(
		govtypes.ModuleName,
		stakingtypes.ModuleName,
		ibctransfertypes.ModuleName,
		ibcexported.ModuleName,
		icatypes.ModuleName,
		capabilitytypes.ModuleName,
		authtypes.ModuleName,
		banktypes.ModuleName,
		distrtypes.ModuleName,
		slashingtypes.ModuleName,
		minttypes.ModuleName,
		genutiltypes.ModuleName,
		evidencetypes.ModuleName,
		authz.ModuleName,
		feegrant.ModuleName,
		ibcwasmtypes.ModuleName,
		group.ModuleName,
		paramstypes.ModuleName,
		upgradetypes.ModuleName,
		vestingtypes.ModuleName,
		consensusparamtypes.ModuleName,
		wasmtypes.ModuleName,
		tftypes.ModuleName,
		pooltypes.ModuleName,
	)

	// NOTE: The genutils module must occur after staking so that pools are
	// properly initialized with tokens from genesis accounts.
	// NOTE: Capability module must occur first so that it can initialize any capabilities
	// so that other modules that want to create or claim capabilities afterwards in InitChain
	// can do so safely.
	genesisModuleOrder := []string{
		capabilitytypes.ModuleName,
		accounts.ModuleName,
		authtypes.ModuleName,
		banktypes.ModuleName,
		distrtypes.ModuleName,
		stakingtypes.ModuleName,
		slashingtypes.ModuleName,
		govtypes.ModuleName,
		minttypes.ModuleName,
		genutiltypes.ModuleName,
		ibcwasmtypes.ModuleName,
		ibctransfertypes.ModuleName,
		ibcexported.ModuleName,
		icatypes.ModuleName,
		evidencetypes.ModuleName,
		authz.ModuleName,
		feegrant.ModuleName,
		group.ModuleName,
		paramstypes.ModuleName,
		upgradetypes.ModuleName,
		vestingtypes.ModuleName,
		consensusparamtypes.ModuleName,
		pooltypes.ModuleName,
		wasmtypes.ModuleName,
		tftypes.ModuleName,
	}
	app.ModuleManager.SetOrderInitGenesis(genesisModuleOrder...)
	app.ModuleManager.SetOrderExportGenesis(genesisModuleOrder...)

	app.configurator = module.NewConfigurator(app.appCodec, app.MsgServiceRouter(), app.GRPCQueryRouter())
	err = app.ModuleManager.RegisterServices(app.configurator)
	if err != nil {
		panic(err)
	}

	autocliv1.RegisterQueryServer(app.GRPCQueryRouter(), runtimeservices.NewAutoCLIQueryService(app.ModuleManager.Modules))
	reflectionSvc, err := runtimeservices.NewReflectionService()
	if err != nil {
		panic(err)
	}
	reflectionv1.RegisterReflectionServiceServer(app.GRPCQueryRouter(), reflectionSvc)

	// create the simulation manager and define the order of the modules for deterministic simulations
	overrideModules := map[string]module.AppModuleSimulation{
		authtypes.ModuleName: auth.NewAppModule(app.appCodec, app.AuthKeeper, app.AccountKeeper, authsims.RandomGenesisAccounts, nil),
	}
	app.simulationManager = module.NewSimulationManagerFromAppModules(app.ModuleManager.Modules, overrideModules)
	app.simulationManager.RegisterStoreDecoders()

	app.setupUpgradeHandlers()

	// initialize stores
	app.MountKVStores(keys)
	app.MountTransientStores(tkeys)
	app.MountMemoryStores(memKeys)

	// initialize BaseApp
	app.SetInitChainer(app.InitChainer)
	app.SetPreBlocker(app.PreBlocker)
	app.SetBeginBlocker(app.BeginBlocker)

	anteHandler, err := NewAnteHandler(
		HandlerOptions{
			HandlerOptions: ante.HandlerOptions{
				Environment:              runtime.NewEnvironment(nil, app.logger, runtime.EnvWithMsgRouterService(app.MsgServiceRouter()), runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())), // nil is set as the kvstoreservice to avoid module access
				AccountAbstractionKeeper: app.AccountKeeper,
				AccountKeeper:            app.AuthKeeper,
				BankKeeper:               app.BankKeeper,
				ConsensusKeeper:          app.ConsensusParamsKeeper,
				SignModeHandler:          txConfig.SignModeHandler(),
				FeegrantKeeper:           app.FeeGrantKeeper,
				SigGasConsumer:           ante.DefaultSigVerificationGasConsumer,
			},
			IBCKeeper:             app.IBCKeeper,
			WasmConfig:            &wasmConfig,
			TXCounterStoreService: runtime.NewKVStoreService(keys[wasmtypes.StoreKey]),
		},
	)
	if err != nil {
		panic(fmt.Errorf("failed to create AnteHandler: %w", err))
	}

	app.SetAnteHandler(anteHandler)
	app.SetEndBlocker(app.EndBlocker)

	// must be before Loading version
	// requires the snapshot store to be created and registered as a BaseAppOption
	if manager := app.SnapshotManager(); manager != nil {
		err := manager.RegisterExtensions(
			wasmkeeper.NewWasmSnapshotter(app.CommitMultiStore(), &app.WasmKeeper),
		)
		if err != nil {
			panic(fmt.Errorf("failed to register wasm snapshot extension: %s", err))
		}

		err = manager.RegisterExtensions(
			ibcwasmkeeper.NewWasmSnapshotter(app.CommitMultiStore(), &app.WasmClientKeeper),
		)
		if err != nil {
			panic(fmt.Errorf("failed to register ibc-wasm snapshot extension: %s", err))
		}
	}

	if loadLatest {
		if err := app.LoadLatestVersion(); err != nil {
			panic(fmt.Errorf("error loading last version: %w", err))
		}

		ctx := app.BaseApp.NewUncachedContext(true, tmproto.Header{})

		// Initialize pinned codes in wasmvm as they are not persisted there
		if err := app.WasmKeeper.InitializePinnedCodes(ctx); err != nil {
			panic(fmt.Errorf("failed initialize pinned codes %s", err))
		}
	}

	app.ScopedIBCKeeper = scopedIBCKeeper
	app.ScopedTransferKeeper = scopedTransferKeeper
	app.ScopedWasmKeeper = scopedWasmKeeper
	// this line is used by starport scaffolding # stargate/app/beforeInitReturn

	return app
}

// Name returns the name of the App
func (app *UnionApp) Name() string { return app.BaseApp.Name() }

// PreBlocker application updates every pre block
func (app *UnionApp) PreBlocker(ctx sdk.Context, _ *abci.FinalizeBlockRequest) error {
	return app.ModuleManager.PreBlock(ctx)
}

// BeginBlocker application updates every begin block
func (app *UnionApp) BeginBlocker(ctx sdk.Context) (sdk.BeginBlock, error) {
	return app.ModuleManager.BeginBlock(ctx)
}

// EndBlocker application updates every end block
func (app *UnionApp) EndBlocker(ctx sdk.Context) (sdk.EndBlock, error) {
	return app.ModuleManager.EndBlock(ctx)
}

// InitChainer application update at chain initialization
func (app *UnionApp) InitChainer(ctx sdk.Context, req *abci.InitChainRequest) (*abci.InitChainResponse, error) {
	var genesisState GenesisState
	if err := json.Unmarshal(req.AppStateBytes, &genesisState); err != nil {
		panic(err)
	}
	if err := app.UpgradeKeeper.SetModuleVersionMap(ctx, app.ModuleManager.GetVersionMap()); err != nil {
		panic(err)
	}
	return app.ModuleManager.InitGenesis(ctx, genesisState)
}

// Configurator get app configurator
func (app *UnionApp) Configurator() module.Configurator { //nolint:staticcheck // SA1019: Configurator is deprecated but still used in runtime v1.
	return app.configurator
}

// LoadHeight loads a particular height
func (app *UnionApp) LoadHeight(height int64) error {
	return app.LoadVersion(height)
}

// ModuleAccountAddrs returns all the app's module account addresses.
func (app *UnionApp) ModuleAccountAddrs() map[string]bool {
	modAccAddrs := make(map[string]bool)
	for acc := range maccPerms {
		modAccAddrs[authtypes.NewModuleAddress(acc).String()] = true
	}

	return modAccAddrs
}

// BlockedModuleAccountAddrs returns all the app's blocked module account
// addresses.
func (app *UnionApp) BlockedModuleAccountAddrs() map[string]bool {
	modAccAddrs := app.ModuleAccountAddrs()
	delete(modAccAddrs, authtypes.NewModuleAddress(govtypes.ModuleName).String())

	return modAccAddrs
}

// LegacyAmino returns SimApp's amino codec.
//
// NOTE: This is solely to be used for testing purposes as it may be desirable
// for modules to register their own custom testing types.
func (app *UnionApp) LegacyAmino() *codec.LegacyAmino {
	return app.legacyAmino
}

func (app *UnionApp) AutoCliOpts() autocli.AppOptions {
	return autocli.AppOptions{
		Modules:       app.ModuleManager.Modules,
		ModuleOptions: runtimeservices.ExtractAutoCLIOptions(app.ModuleManager.Modules),
	}
}

// AppCodec returns an app codec.
//
// NOTE: This is solely to be used for testing purposes as it may be desirable
// for modules to register their own custom testing types.
func (app *UnionApp) AppCodec() codec.Codec {
	return app.appCodec
}

// InterfaceRegistry returns an InterfaceRegistry
func (app *UnionApp) InterfaceRegistry() types.InterfaceRegistry {
	return app.interfaceRegistry
}

// TxConfig returns SimApp's TxConfig
func (app *UnionApp) TxConfig() client.TxConfig {
	return app.txConfig
}

// DefaultGenesis returns a default genesis from the registered AppModuleBasic's.
func (app *UnionApp) DefaultGenesis() map[string]json.RawMessage {
	return app.ModuleManager.DefaultGenesis()
}

// GetKey returns the KVStoreKey for the provided store key.
//
// NOTE: This is solely to be used for testing purposes.
func (app *UnionApp) GetKey(storeKey string) *storetypes.KVStoreKey {
	return app.keys[storeKey]
}

// GetTKey returns the TransientStoreKey for the provided store key.
//
// NOTE: This is solely to be used for testing purposes.
func (app *UnionApp) GetTKey(storeKey string) *storetypes.TransientStoreKey {
	return app.tkeys[storeKey]
}

// GetMemKey returns the MemStoreKey for the provided mem key.
//
// NOTE: This is solely used for testing purposes.
func (app *UnionApp) GetMemKey(storeKey string) *storetypes.MemoryStoreKey {
	return app.memKeys[storeKey]
}

// GetSubspace returns a param subspace for a given module name.
//
// NOTE: This is solely to be used for testing purposes.
func (app *UnionApp) GetSubspace(moduleName string) paramstypes.Subspace {
	subspace, _ := app.ParamsKeeper.GetSubspace(moduleName)
	return subspace
}

// RegisterAPIRoutes registers all application module routes with the provided
// API server.
func (app *UnionApp) RegisterAPIRoutes(apiSvr *api.Server, apiConfig config.APIConfig) {
	clientCtx := apiSvr.ClientCtx
	// Register new tx routes from grpc-gateway.
	authtx.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
	// Register new tendermint queries routes from grpc-gateway.
	cmtservice.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
	// Register node gRPC service for grpc-gateway.
	nodeservice.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)

	// Register grpc-gateway routes for all modules.
	app.ModuleManager.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)

	// register app's OpenAPI routes.
	docs.RegisterOpenAPIService(Name, apiSvr.Router)
}

// RegisterTxService implements the Application.RegisterTxService method.
func (app *UnionApp) RegisterTxService(clientCtx client.Context) {
	authtx.RegisterTxService(app.BaseApp.GRPCQueryRouter(), clientCtx, app.BaseApp.Simulate, app.interfaceRegistry)
}

// RegisterTendermintService implements the Application.RegisterTendermintService method.
func (app *UnionApp) RegisterTendermintService(clientCtx client.Context) {
	cmtservice.RegisterTendermintService(
		clientCtx,
		app.BaseApp.GRPCQueryRouter(),
		app.interfaceRegistry,
		app.Query,
	)
}

// RegisterNodeService implements the Application.RegisterNodeService method.
func (app *UnionApp) RegisterNodeService(clientCtx client.Context, cfg config.Config) {
	nodeservice.RegisterNodeService(clientCtx, app.GRPCQueryRouter(), cfg)
}

// ValidatorKeyProvider returns a function that generates a validator key
// Supported key types are those supported by Comet: ed25519, secp256k1, bls12-381
func (app *UnionApp) ValidatorKeyProvider() runtime.KeyGenF {
	return func() (cmtcrypto.PrivKey, error) {
		return bn254.GenPrivKey(), nil
	}
}

// initParamsKeeper init params keeper and its subspaces
func initParamsKeeper(appCodec codec.BinaryCodec, legacyAmino *codec.LegacyAmino, key, tkey storetypes.StoreKey) paramskeeper.Keeper {
	paramsKeeper := paramskeeper.NewKeeper(appCodec, legacyAmino, key, tkey)

	paramsKeeper.Subspace(authtypes.ModuleName)
	paramsKeeper.Subspace(banktypes.ModuleName)
	paramsKeeper.Subspace(stakingtypes.ModuleName)
	paramsKeeper.Subspace(minttypes.ModuleName)
	paramsKeeper.Subspace(distrtypes.ModuleName)
	paramsKeeper.Subspace(slashingtypes.ModuleName)
	paramsKeeper.Subspace(govtypes.ModuleName)
	paramsKeeper.Subspace(wasmtypes.ModuleName)
	paramsKeeper.Subspace(tftypes.ModuleName)

	keyTable := ibcclienttypes.ParamKeyTable()
	keyTable.RegisterParamSet(&ibcconnectiontypes.Params{})
	paramsKeeper.Subspace(ibcexported.ModuleName).WithKeyTable(keyTable)
	paramsKeeper.Subspace(ibctransfertypes.ModuleName).WithKeyTable(ibctransfertypes.ParamKeyTable())
	paramsKeeper.Subspace(icacontrollertypes.SubModuleName).WithKeyTable(icacontrollertypes.ParamKeyTable())
	paramsKeeper.Subspace(icahosttypes.SubModuleName).WithKeyTable(icahosttypes.ParamKeyTable())

	return paramsKeeper
}

// SimulationManager implements the SimulationApp interface
func (app *UnionApp) SimulationManager() *module.SimulationManager {
	return app.simulationManager
}

func AllCapabilities() []string {
	return []string{
		"iterator",
		"staking",
		"stargate",
		"cosmwasm_1_1",
		"cosmwasm_1_2",
		"cosmwasm_1_3",
		"cosmwasm_1_4",
		"cosmwasm_2_0",
	}
}
