package app

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	reflectionv1 "cosmossdk.io/api/cosmos/reflection/v1"
	"cosmossdk.io/client/v2/autocli"
	corestore "cosmossdk.io/core/store"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/accounts"
	"cosmossdk.io/x/accounts/accountstd"
	baseaccount "cosmossdk.io/x/accounts/defaults/base"
	lockup "cosmossdk.io/x/accounts/defaults/lockup"
	multisig "cosmossdk.io/x/accounts/defaults/multisig"
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
	"github.com/cosmos/cosmos-sdk/types/mempool"
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

	ibccometblsclient "union/app/ibc/cometbls/02-client/keeper"
	"union/docs"
	unionstaking "union/x/staking"
)

const (
	AccountAddressPrefix = "union"
	Name                 = "union"
)

var (
	// DefaultNodeHome is the default home directory for the application daemon.
	DefaultNodeHome string

	// ProposalsEnabled enables or disables all x/wasm proposals.
	ProposalsEnabled = "true"
	// EnableSpecificProposals (if non-empty) must be a comma-separated list of values
	// that are a subset of "EnableAllProposals". This takes precedence over ProposalsEnabled.
	EnableSpecificProposals = ""

	// Module account permissions.
	maccPerms = map[string][]string{
		authtypes.FeeCollectorName:         nil,
		distrtypes.ModuleName:              nil,
		pooltypes.ModuleName:               nil,
		pooltypes.StreamAccount:            nil,
		pooltypes.ProtocolPoolDistrAccount: nil,
		minttypes.ModuleName:               {authtypes.Minter},
		stakingtypes.BondedPoolName:        {authtypes.Burner, authtypes.Staking},
		stakingtypes.NotBondedPoolName:     {authtypes.Burner, authtypes.Staking},
		govtypes.ModuleName:                {authtypes.Burner},
		ibctransfertypes.ModuleName:        {authtypes.Minter, authtypes.Burner},
		wasmtypes.ModuleName:               {authtypes.Burner},
	}
)

// contractMemoryLimit defines the memory limit (in MiB) for each contract execution.
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

// UnionApp extends an ABCI application with most of its parameters exported.
type UnionApp struct {
	*baseapp.BaseApp

	logger            log.Logger
	legacyAmino       *codec.LegacyAmino
	appCodec          codec.Codec
	interfaceRegistry types.InterfaceRegistry
	txConfig          client.TxConfig

	// Keys to access the substores.
	keys    map[string]*storetypes.KVStoreKey
	tkeys   map[string]*storetypes.TransientStoreKey
	memKeys map[string]*storetypes.MemoryStoreKey

	// Keepers.
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
	IBCKeeper             *ibckeeper.Keeper // IBC Keeper must be a pointer so that we can set the router correctly.
	EvidenceKeeper        evidencekeeper.Keeper
	TransferKeeper        ibctransferkeeper.Keeper
	FeeGrantKeeper        feegrantkeeper.Keeper
	GroupKeeper           groupkeeper.Keeper
	ConsensusParamsKeeper consensusparamkeeper.Keeper
	WasmKeeper            wasmkeeper.Keeper
	PoolKeeper            poolkeeper.Keeper

	// Public scoped keepers (for testing purposes).
	ScopedIBCKeeper      capabilitykeeper.ScopedKeeper
	ScopedTransferKeeper capabilitykeeper.ScopedKeeper
	ScopedICAHostKeeper  capabilitykeeper.ScopedKeeper
	ScopedIBCFeeKeeper   capabilitykeeper.ScopedKeeper
	ScopedWasmKeeper     capabilitykeeper.ScopedKeeper

	ModuleManager     *module.Manager
	simulationManager *module.SimulationManager
	configurator      module.Configurator // Deprecated but still used in runtime v1.
}

// NewUnionApp returns a reference to an initialized blockchain app.
func NewUnionApp(
	logger log.Logger,
	db corestore.KVStoreWithBatch,
	traceStore io.Writer,
	loadLatest bool,
	appOpts servertypes.AppOptions,
	wasmOpts []wasmkeeper.Option,
	baseAppOptions ...func(*baseapp.BaseApp),
) *UnionApp {
	homePath := cast.ToString(appOpts.Get(flags.FlagHome))
	if homePath == "" {
		panic("home path not provided in app options")
	}

	// 1. Initialize BaseApp.
	bApp, txConfig, interfaceRegistry, appCodec, legacyAmino, signingCtx := initBaseApp(logger, db, traceStore, baseAppOptions)

	// 2. Initialize mempool.
	initMempool(bApp, signingCtx)

	// 3. Create store keys.
	keys := storetypes.NewKVStoreKeys(
		authtypes.StoreKey, authz.ModuleName, banktypes.StoreKey, stakingtypes.StoreKey,
		minttypes.StoreKey, distrtypes.StoreKey, slashingtypes.StoreKey,
		govtypes.StoreKey, paramstypes.StoreKey, ibcexported.StoreKey, upgradetypes.StoreKey,
		feegrant.StoreKey, evidencetypes.StoreKey, ibctransfertypes.StoreKey,
		capabilitytypes.StoreKey, group.StoreKey,
		consensusparamtypes.StoreKey, wasmtypes.StoreKey,
		pooltypes.StoreKey, accounts.StoreKey,
	)
	tkeys := storetypes.NewTransientStoreKeys(paramstypes.TStoreKey)
	memKeys := storetypes.NewMemoryStoreKeys(capabilitytypes.MemStoreKey)

	// 4. Create app instance.
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
	govModuleAddr := getGovModuleAddr(signingCtx)
	app.ParamsKeeper = initParamsKeeper(appCodec, legacyAmino, keys[paramstypes.StoreKey], tkeys[paramstypes.TStoreKey])
	app.ConsensusParamsKeeper = consensusparamkeeper.NewKeeper(
		appCodec,
		runtime.NewEnvironment(runtime.NewKVStoreService(keys[consensusparamtypes.StoreKey]), logger.With(log.ModuleKey, "x/consensus")),
		govModuleAddr,
	)
	bApp.SetParamStore(app.ConsensusParamsKeeper.ParamsStore)
	bApp.SetVersionModifier(consensus.ProvideAppVersionModifier(app.ConsensusParamsKeeper))

	// 5. Initialize capability keeper.
	app.CapabilityKeeper = capabilitykeeper.NewKeeper(
		appCodec,
		runtime.NewKVStoreService(keys[capabilitytypes.StoreKey]),
		memKeys[capabilitytypes.MemStoreKey],
	)
	scopedIBCKeeper := app.CapabilityKeeper.ScopeToModule(ibcexported.ModuleName)
	scopedTransferKeeper := app.CapabilityKeeper.ScopeToModule(ibctransfertypes.ModuleName)
	scopedWasmKeeper := app.CapabilityKeeper.ScopeToModule(wasmtypes.ModuleName)

	// 6. Initialize remaining keepers.
	initKeepers(app, signingCtx, govModuleAddr, cometService, scopedIBCKeeper, scopedTransferKeeper, scopedWasmKeeper, appOpts, wasmOpts)

	// 7. Initialize Module Manager and set module orders.
	initModuleManager(app, txConfig)

	// 8. Setup upgrade handlers, ante handler, and snapshot extensions.
	setupUpgradeAndAnte(app, appOpts, wasmOpts)

	// 9. Mount stores and load latest version if required.
	app.MountKVStores(keys)
	app.MountTransientStores(tkeys)
	app.MountMemoryStores(memKeys)
	app.SetInitChainer(app.InitChainer)
	app.SetPreBlocker(app.PreBlocker)
	app.SetBeginBlocker(app.BeginBlocker)
	app.SetEndBlocker(app.EndBlocker)

	if loadLatest {
		if err := app.LoadLatestVersion(); err != nil {
			panic(fmt.Errorf("error loading latest version: %w", err))
		}
		ctx := app.BaseApp.NewUncachedContext(true, tmproto.Header{})
		if err := app.WasmKeeper.InitializePinnedCodes(ctx); err != nil {
			panic(fmt.Errorf("failed to initialize pinned codes: %w", err))
		}
	}

	app.ScopedIBCKeeper = scopedIBCKeeper
	app.ScopedTransferKeeper = scopedTransferKeeper
	app.ScopedWasmKeeper = scopedWasmKeeper

	return app
}

// initBaseApp creates and configures the BaseApp.
func initBaseApp(logger log.Logger, db corestore.KVStoreWithBatch, traceStore io.Writer, baseAppOptions []func(*baseapp.BaseApp)) (*baseapp.BaseApp, client.TxConfig, types.InterfaceRegistry, codec.Codec, *codec.LegacyAmino, signing.Context) {
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

	bApp := baseapp.NewBaseApp(Name, logger, db, txConfig.TxDecoder(), baseAppOptions...)
	bApp.SetCommitMultiStoreTracer(traceStore)
	bApp.SetVersion(version.Version)
	bApp.SetInterfaceRegistry(interfaceRegistry)
	bApp.SetTxEncoder(txConfig.TxEncoder())

	return bApp, txConfig, interfaceRegistry, appCodec, legacyAmino, signingCtx
}

// initMempool configures the mempool with priority addresses.
func initMempool(bApp *baseapp.BaseApp, signingCtx signing.Context) {
	parentTxPriority := mempool.NewDefaultTxPriority()
	priorityAddresses := strings.Split(os.Getenv("MEMPOOL_PRIORITY"), ",")
	bApp.SetMempool(mempool.NewPriorityMempool(mempool.PriorityNonceMempoolConfig[int64]{
		TxPriority: mempool.TxPriority[int64]{
			GetTxPriority: func(goCtx context.Context, tx sdk.Tx) int64 {
				senders, err := tx.GetSenders()
				if err == nil {
					for _, sender := range senders {
						unionSender, err := signingCtx.AddressCodec().BytesToString(sender)
						if err != nil {
							panic("impossible")
						}
						for _, prioritySender := range priorityAddresses {
							if unionSender == prioritySender {
								return 100
							}
						}
					}
				}
				return parentTxPriority.GetTxPriority(goCtx, tx)
			},
			Compare:  parentTxPriority.Compare,
			MinValue: parentTxPriority.MinValue,
		},
		SignerExtractor: mempool.NewDefaultSignerExtractionAdapter(),
	}))
}

// getGovModuleAddr returns the governance module address as a string.
func getGovModuleAddr(signingCtx signing.Context) string {
	addr, err := signingCtx.AddressCodec().BytesToString(authtypes.NewModuleAddress(govtypes.ModuleName))
	if err != nil {
		panic(fmt.Errorf("failed to get gov module address: %w", err))
	}
	return addr
}

// initKeepers initializes the remaining keepers.
func initKeepers(app *UnionApp, signingCtx signing.Context, govModuleAddr string, cometService *runtime.CometInfoService,
	scopedIBCKeeper, scopedTransferKeeper, scopedWasmKeeper capabilitykeeper.ScopedKeeper,
	appOpts servertypes.AppOptions, wasmOpts []wasmkeeper.Option) {
	// Initialize Accounts Keeper.
	accountsKeeper, err := accounts.NewKeeper(
		app.appCodec,
		runtime.NewEnvironment(runtime.NewKVStoreService(app.keys[accounts.StoreKey]), app.logger.With(log.ModuleKey, "x/accounts"),
			runtime.EnvWithMsgRouterService(app.MsgServiceRouter()),
			runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())),
		signingCtx.AddressCodec(),
		app.appCodec.InterfaceRegistry(),
		accountstd.AddAccount(lockup.CONTINUOUS_LOCKING_ACCOUNT, lockup.NewContinuousLockingAccount),
		accountstd.AddAccount(lockup.PERIODIC_LOCKING_ACCOUNT, lockup.NewPeriodicLockingAccount),
		accountstd.AddAccount(lockup.DELAYED_LOCKING_ACCOUNT, lockup.NewDelayedLockingAccount),
		accountstd.AddAccount(lockup.PERMANENT_LOCKING_ACCOUNT, lockup.NewPermanentLockingAccount),
		accountstd.AddAccount("multisig", multisig.NewAccount),
		baseaccount.NewAccount("base", app.txConfig.SignModeHandler(), baseaccount.WithSecp256K1PubKey()),
	)
	if err != nil {
		panic(fmt.Errorf("failed to initialize accounts keeper: %w", err))
	}
	app.AccountKeeper = accountsKeeper

	// Initialize AuthKeeper.
	app.AuthKeeper = authkeeper.NewAccountKeeper(
		runtime.NewEnvironment(runtime.NewKVStoreService(app.keys[authtypes.StoreKey]), app.logger.With(log.ModuleKey, "x/auth")),
		app.appCodec,
		authtypes.ProtoBaseAccount,
		accountsKeeper,
		maccPerms,
		signingCtx.AddressCodec(),
		AccountAddressPrefix,
		govModuleAddr,
	)

	// Initialize AuthzKeeper.
	app.AuthzKeeper = authzkeeper.NewKeeper(
		runtime.NewEnvironment(runtime.NewKVStoreService(app.keys[authzkeeper.StoreKey]), app.logger.With(log.ModuleKey, "x/authz"),
			runtime.EnvWithMsgRouterService(app.MsgServiceRouter()),
			runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())),
		app.appCodec,
		app.AuthKeeper,
	)

	// Initialize BankKeeper.
	blockedAddrs := app.BlockedModuleAccountAddrs()
	app.BankKeeper = bankkeeper.NewBaseKeeper(
		runtime.NewEnvironment(runtime.NewKVStoreService(app.keys[banktypes.StoreKey]), app.logger.With(log.ModuleKey, "x/bank")),
		app.appCodec,
		app.AuthKeeper,
		blockedAddrs,
		govModuleAddr,
	)

	// Initialize StakingKeeper.
	app.StakingKeeper = stakingkeeper.NewKeeper(
		app.appCodec,
		runtime.NewEnvironment(runtime.NewKVStoreService(app.keys[stakingtypes.StoreKey]), app.logger.With(log.ModuleKey, "x/staking"),
			runtime.EnvWithMsgRouterService(app.MsgServiceRouter()),
			runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())),
		app.AuthKeeper,
		app.BankKeeper,
		app.ConsensusParamsKeeper,
		govModuleAddr,
		signingCtx.ValidatorAddressCodec(),
		authcodec.NewBech32Codec(sdk.GetBech32PrefixConsAddr(AccountAddressPrefix)),
		cometService,
	)

	// Additional keepers (FeeGrant, Mint, Pool, Distr, Slashing, Group, etc.) can be initialized similarly.
	// ...
}

// initModuleManager initializes the Module Manager and sets module ordering.
func initModuleManager(app *UnionApp, txConfig client.TxConfig) {
	app.ModuleManager = module.NewManager(
		genutil.NewAppModule(app.appCodec, app.AuthKeeper, app.StakingKeeper, app, txConfig, genutiltypes.DefaultMessageValidator),
		accounts.NewAppModule(app.appCodec, app.AccountKeeper),
		auth.NewAppModule(app.appCodec, app.AuthKeeper, app.AccountKeeper, authsims.RandomGenesisAccounts, nil),
		authzmodule.NewAppModule(app.appCodec, app.AuthzKeeper, app.interfaceRegistry),
		vesting.NewAppModule(app.AuthKeeper, app.BankKeeper),
		bank.NewAppModule(app.appCodec, app.BankKeeper, app.AuthKeeper),
		capability.NewAppModule(app.appCodec, *app.CapabilityKeeper, false),
		feegrantmodule.NewAppModule(app.appCodec, app.FeeGrantKeeper, app.interfaceRegistry),
		groupmodule.NewAppModule(app.appCodec, app.GroupKeeper, app.AuthKeeper, app.BankKeeper, app.interfaceRegistry),
		gov.NewAppModule(app.appCodec, &app.GovKeeper, app.AuthKeeper, app.BankKeeper, app.PoolKeeper),
		mint.NewAppModule(app.appCodec, app.MintKeeper, app.AuthKeeper, nil),
		slashing.NewAppModule(app.appCodec, app.SlashingKeeper, app.AuthKeeper, app.BankKeeper, app.StakingKeeper, app.interfaceRegistry, nil),
		distr.NewAppModule(app.appCodec, app.DistrKeeper, app.StakingKeeper),
		staking.NewAppModule(app.appCodec, app.StakingKeeper),
		upgrade.NewAppModule(app.UpgradeKeeper),
		evidence.NewAppModule(app.appCodec, app.EvidenceKeeper, nil),
		consensus.NewAppModule(app.appCodec, app.ConsensusParamsKeeper),
		protocolpool.NewAppModule(app.appCodec, app.PoolKeeper, app.AuthKeeper, app.BankKeeper),
		ibc.NewAppModule(app.appCodec, app.IBCKeeper),
		params.NewAppModule(app.ParamsKeeper),
		transfer.NewAppModule(app.appCodec, app.TransferKeeper),
		wasm.NewAppModule(app.appCodec, &app.WasmKeeper, app.StakingKeeper, app.AuthKeeper, app.BankKeeper, app.MsgServiceRouter(), app.GetSubspace(wasmtypes.ModuleName)),
		ibctm.NewAppModule(),
		solomachine.NewAppModule(),
	)
	// Set module orders for begin block, end block, genesis, etc.
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
		genutiltypes.ModuleName,
		authz.ModuleName,
		feegrant.ModuleName,
		group.ModuleName,
		paramstypes.ModuleName,
		vestingtypes.ModuleName,
		consensusparamtypes.ModuleName,
		wasmtypes.ModuleName,
	)
	// Set end block and genesis ordering similarly...
}

// setupUpgradeAndAnte sets up upgrade handlers, the ante handler, and snapshot extensions.
func setupUpgradeAndAnte(app *UnionApp, appOpts servertypes.AppOptions, wasmOpts []wasmkeeper.Option) {
	app.setupUpgradeStoreLoaders()

	wasmDir := filepath.Join(cast.ToString(appOpts.Get(flags.FlagHome)), "wasm")
	wasmConfig, err := wasm.ReadWasmConfig(appOpts)
	if err != nil {
		panic(fmt.Errorf("error while reading wasm config: %s", err))
	}
	anteHandler, err := NewAnteHandler(
		HandlerOptions{
			HandlerOptions: ante.HandlerOptions{
				Environment:              runtime.NewEnvironment(nil, app.logger, runtime.EnvWithMsgRouterService(app.MsgServiceRouter()), runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())),
				AccountAbstractionKeeper: app.AccountKeeper,
				AccountKeeper:            app.AuthKeeper,
				BankKeeper:               app.BankKeeper,
				ConsensusKeeper:          app.ConsensusParamsKeeper,
				SignModeHandler:          app.txConfig.SignModeHandler(),
				FeegrantKeeper:           app.FeeGrantKeeper,
				SigGasConsumer:           ante.DefaultSigVerificationGasConsumer,
			},
			IBCKeeper:             app.IBCKeeper,
			WasmConfig:            &wasmConfig,
			TXCounterStoreService: runtime.NewKVStoreService(app.keys[wasmtypes.StoreKey]),
		},
	)
	if err != nil {
		panic(fmt.Errorf("failed to create AnteHandler: %w", err))
	}
	app.SetAnteHandler(anteHandler)
	app.SetEndBlocker(app.EndBlocker)

	if manager := app.SnapshotManager(); manager != nil {
		if err := manager.RegisterExtensions(
			wasmkeeper.NewWasmSnapshotter(app.CommitMultiStore(), &app.WasmKeeper),
		); err != nil {
			panic(fmt.Errorf("failed to register wasm snapshot extension: %s", err))
		}
	}
}
  
// Close closes all necessary application resources.
func (app *UnionApp) Close() error {
	return app.BaseApp.Close()
}

// Name returns the name of the App.
func (app *UnionApp) Name() string { return app.BaseApp.Name() }

// PreBlocker executes updates at the beginning of every pre-block.
func (app *UnionApp) PreBlocker(ctx sdk.Context, _ *abci.FinalizeBlockRequest) error {
	return app.ModuleManager.PreBlock(ctx)
}

// BeginBlocker executes updates at the beginning of every block.
func (app *UnionApp) BeginBlocker(ctx sdk.Context) (sdk.BeginBlock, error) {
	return app.ModuleManager.BeginBlock(ctx)
}

// EndBlocker executes updates at the end of every block.
func (app *UnionApp) EndBlocker(ctx sdk.Context) (sdk.EndBlock, error) {
	return app.ModuleManager.EndBlock(ctx)
}

// InitChainer performs the application update at chain initialization.
func (app *UnionApp) InitChainer(ctx sdk.Context, req *abci.InitChainRequest) (*abci.InitChainResponse, error) {
	var genesisState GenesisState
	if err := json.Unmarshal(req.AppStateBytes, &genesisState); err != nil {
		return nil, err
	}
	if err := app.UpgradeKeeper.SetModuleVersionMap(ctx, app.ModuleManager.GetVersionMap()); err != nil {
		return nil, err
	}
	return app.ModuleManager.InitGenesis(ctx, genesisState)
}

// Configurator returns the app configurator.
func (app *UnionApp) Configurator() module.Configurator {
	return app.configurator
}

// LoadHeight loads a particular height.
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

// BlockedModuleAccountAddrs returns the blocked module account addresses.
func (app *UnionApp) BlockedModuleAccountAddrs() map[string]bool {
	modAccAddrs := app.ModuleAccountAddrs()
	delete(modAccAddrs, authtypes.NewModuleAddress(govtypes.ModuleName).String())
	return modAccAddrs
}

// LegacyAmino returns the app's legacy amino codec (for testing purposes).
func (app *UnionApp) LegacyAmino() *codec.LegacyAmino {
	return app.legacyAmino
}

// AutoCliOpts returns the AutoCLI options.
func (app *UnionApp) AutoCliOpts() autocli.AppOptions {
	return autocli.AppOptions{
		Modules:       app.ModuleManager.Modules,
		ModuleOptions: runtimeservices.ExtractAutoCLIOptions(app.ModuleManager.Modules),
	}
}

// AppCodec returns the app codec.
func (app *UnionApp) AppCodec() codec.Codec {
	return app.appCodec
}

// InterfaceRegistry returns the interface registry.
func (app *UnionApp) InterfaceRegistry() types.InterfaceRegistry {
	return app.interfaceRegistry
}

// TxConfig returns the TxConfig.
func (app *UnionApp) TxConfig() client.TxConfig {
	return app.txConfig
}

// DefaultGenesis returns the default genesis state.
func (app *UnionApp) DefaultGenesis() map[string]json.RawMessage {
	return app.ModuleManager.DefaultGenesis()
}

// GetKey returns the KVStoreKey for the provided store key.
func (app *UnionApp) GetKey(storeKey string) *storetypes.KVStoreKey {
	return app.keys[storeKey]
}

// GetTKey returns the TransientStoreKey for the provided store key.
func (app *UnionApp) GetTKey(storeKey string) *storetypes.TransientStoreKey {
	return app.tkeys[storeKey]
}

// GetMemKey returns the MemoryStoreKey for the provided store key.
func (app *UnionApp) GetMemKey(storeKey string) *storetypes.MemoryStoreKey {
	return app.memKeys[storeKey]
}

// GetSubspace returns a parameter subspace for the given module name.
func (app *UnionApp) GetSubspace(moduleName string) paramstypes.Subspace {
	subspace, _ := app.ParamsKeeper.GetSubspace(moduleName)
	return subspace
}

// RegisterAPIRoutes registers all API routes.
func (app *UnionApp) RegisterAPIRoutes(apiSvr *api.Server, apiConfig config.APIConfig) {
	clientCtx := apiSvr.ClientCtx
	authtx.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
	cmtservice.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
	nodeservice.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
	app.ModuleManager.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
	docs.RegisterOpenAPIService(Name, apiSvr.Router)
}

// RegisterTxService registers the Tx service.
func (app *UnionApp) RegisterTxService(clientCtx client.Context) {
	authtx.RegisterTxService(app.BaseApp.GRPCQueryRouter(), clientCtx, app.BaseApp.Simulate, app.interfaceRegistry)
}

// RegisterTendermintService registers the Tendermint service.
func (app *UnionApp) RegisterTendermintService(clientCtx client.Context) {
	cmtservice.RegisterTendermintService(
		clientCtx,
		app.BaseApp.GRPCQueryRouter(),
		app.interfaceRegistry,
		app.Query,
	)
}

// RegisterNodeService registers the Node service.
func (app *UnionApp) RegisterNodeService(clientCtx client.Context, cfg config.Config) {
	nodeservice.RegisterNodeService(clientCtx, app.GRPCQueryRouter(), cfg)
}

// ValidatorKeyProvider returns a function that generates a validator key.
func (app *UnionApp) ValidatorKeyProvider() runtime.KeyGenF {
	return func() (cmtcrypto.PrivKey, error) {
		return bn254.GenPrivKey(), nil
	}
}

// initParamsKeeper initializes the params keeper and its subspaces.
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
	keyTable := ibcclienttypes.ParamKeyTable()
	keyTable.RegisterParamSet(&ibcconnectiontypes.Params{})
	paramsKeeper.Subspace(ibcexported.ModuleName).WithKeyTable(keyTable)
	paramsKeeper.Subspace(ibctransfertypes.ModuleName).WithKeyTable(ibctransfertypes.ParamKeyTable())
	return paramsKeeper
}

// SimulationManager returns the simulation manager.
func (app *UnionApp) SimulationManager() *module.SimulationManager {
	return app.simulationManager
}

// AllCapabilities returns all available capabilities.
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
		"cosmwasm_2_1",
	}
}
