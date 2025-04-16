package app

import (
	"fmt"
	"path/filepath"

	storetypes "cosmossdk.io/store/types"
	"github.com/CosmWasm/wasmd/x/wasm"
	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	wasmvm "github.com/CosmWasm/wasmvm/v2"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/runtime"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/cosmos/cosmos-sdk/x/auth/posthandler"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	distrkeeper "github.com/cosmos/cosmos-sdk/x/distribution/keeper"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	"github.com/cosmos/gogoproto/proto"
	ibcfee "github.com/cosmos/ibc-go/v8/modules/apps/29-fee"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
)

// ContractMemoryLimit is the memory limit of each contract execution (in MiB)
// constant value so all nodes run with the same limit.
const ContractMemoryLimit = 32

// registerWasmModules register CosmWasm keepers and non dependency inject modules.
func (app *App) registerWasmModules(
	appOpts servertypes.AppOptions,
	wasmOpts ...wasmkeeper.Option,
) (porttypes.IBCModule, error) {
	// set up non depinject support modules store keys
	if err := app.RegisterStores(
		storetypes.NewKVStoreKey(wasmtypes.StoreKey),
	); err != nil {
		panic(err)
	}

	scopedWasmKeeper := app.CapabilityKeeper.ScopeToModule(wasmtypes.ModuleName)

	nodeConfig, err := wasm.ReadNodeConfig(appOpts)
	if err != nil {
		return nil, fmt.Errorf("error while reading wasm config: %s", err)
	}

	// create a new wasmvm with our own settings
	wasmDir := filepath.Join(DefaultNodeHome, "wasm")
	wasmer, err := wasmvm.NewVM(
		wasmDir,
		wasmkeeper.BuiltInCapabilities(),
		ContractMemoryLimit,
		nodeConfig.ContractDebugMode,
		nodeConfig.MemoryCacheSize,
	)
	if err != nil {
		panic(err)
	}
	wasmOpts = append(wasmOpts, wasmkeeper.WithWasmEngine(wasmer))

	// The last arguments can contain custom message handlers, and custom query handlers,
	// if we want to allow any custom callbacks
	app.WasmKeeper = wasmkeeper.NewKeeper(
		app.AppCodec(),
		runtime.NewKVStoreService(app.GetKey(wasmtypes.StoreKey)),
		app.AccountKeeper,
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
		nodeConfig,
		wasmtypes.VMConfig{},
		wasmkeeper.BuiltInCapabilities(),
		authtypes.NewModuleAddress(govtypes.ModuleName).String(),
		wasmOpts...,
	)

	// register IBC modules
	if err := app.RegisterModules(
		wasm.NewAppModule(
			app.AppCodec(),
			&app.WasmKeeper,
			app.StakingKeeper,
			app.AccountKeeper,
			app.BankKeeper,
			app.MsgServiceRouter(),
			app.GetSubspace(wasmtypes.ModuleName),
		)); err != nil {
		return nil, err
	}

	if err := app.setAnteHandler(app.txConfig, nodeConfig, app.GetKey(wasmtypes.StoreKey)); err != nil {
		return nil, err
	}

	if manager := app.SnapshotManager(); manager != nil {
		err := manager.RegisterExtensions(
			wasmkeeper.NewWasmSnapshotter(app.CommitMultiStore(), &app.WasmKeeper),
		)
		if err != nil {
			return nil, fmt.Errorf("failed to register snapshot extension: %s", err)
		}
	}
	app.ScopedWasmKeeper = scopedWasmKeeper

	if err := app.setPostHandler(); err != nil {
		return nil, err
	}

	// At startup, after all modules have been registered, check that all proto
	// annotations are correct.
	protoFiles, err := proto.MergedRegistry()
	if err != nil {
		return nil, err
	}
	err = msgservice.ValidateProtoAnnotations(protoFiles)
	if err != nil {
		return nil, err
	}

	// Create fee enabled wasm ibc Stack
	var wasmStack porttypes.IBCModule
	wasmStack = wasm.NewIBCHandler(app.WasmKeeper, app.IBCKeeper.ChannelKeeper, app.IBCFeeKeeper)
	wasmStack = ibcfee.NewIBCMiddleware(wasmStack, app.IBCFeeKeeper)

	return wasmStack, nil
}

func (app *App) setPostHandler() error {
	postHandler, err := posthandler.NewPostHandler(
		posthandler.HandlerOptions{},
	)
	if err != nil {
		return err
	}
	app.SetPostHandler(postHandler)
	return nil
}

func (app *App) setAnteHandler(txConfig client.TxConfig, nodeConfig wasmtypes.NodeConfig, txCounterStoreKey *storetypes.KVStoreKey) error {
	anteHandler, err := NewAnteHandler(
		HandlerOptions{
			HandlerOptions: ante.HandlerOptions{
				AccountKeeper:   app.AccountKeeper,
				BankKeeper:      app.BankKeeper,
				SignModeHandler: txConfig.SignModeHandler(),
				FeegrantKeeper:  app.FeeGrantKeeper,
				SigGasConsumer:  ante.DefaultSigVerificationGasConsumer,
			},
			IBCKeeper:              app.IBCKeeper,
			NodeConfig:             &nodeConfig,
			WasmKeeper:             &app.WasmKeeper,
			TXCounterStoreService:  runtime.NewKVStoreService(txCounterStoreKey),
			CircuitKeeper:          &app.CircuitBreakerKeeper,
			FeeMarketKeeper:        app.FeeMarketKeeper,
			FeeMarketBankKeeper:    app.BankKeeper,
			FeeMarketAccountKeeper: app.AccountKeeper,
		},
	)
	if err != nil {
		return fmt.Errorf("failed to create AnteHandler: %s", err)
	}

	// Set the AnteHandler for the app
	app.SetAnteHandler(anteHandler)
	return nil
}
