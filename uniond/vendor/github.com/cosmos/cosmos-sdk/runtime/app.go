package runtime

import (
	"encoding/json"
	"fmt"

	abci "github.com/cometbft/cometbft/abci/types"
	"golang.org/x/exp/slices"

	runtimev1alpha1 "cosmossdk.io/api/cosmos/app/runtime/v1alpha1"
	appv1alpha1 "cosmossdk.io/api/cosmos/app/v1alpha1"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	nodeservice "github.com/cosmos/cosmos-sdk/client/grpc/node"
	"github.com/cosmos/cosmos-sdk/client/grpc/tmservice"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/server/api"
	"github.com/cosmos/cosmos-sdk/server/config"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

// App is a wrapper around BaseApp and ModuleManager that can be used in hybrid
// app.go/app config scenarios or directly as a servertypes.Application instance.
// To get an instance of *App, *AppBuilder must be requested as a dependency
// in a container which declares the runtime module and the AppBuilder.Build()
// method must be called.
//
// App can be used to create a hybrid app.go setup where some configuration is
// done declaratively with an app config and the rest of it is done the old way.
// See simapp/app.go for an example of this setup.
type App struct {
	*baseapp.BaseApp

	ModuleManager     *module.Manager
	configurator      module.Configurator
	config            *runtimev1alpha1.Module
	storeKeys         []storetypes.StoreKey
	interfaceRegistry codectypes.InterfaceRegistry
	cdc               codec.Codec
	amino             *codec.LegacyAmino
	basicManager      module.BasicManager
	baseAppOptions    []BaseAppOption
	msgServiceRouter  *baseapp.MsgServiceRouter
	appConfig         *appv1alpha1.Config
	// initChainer is the init chainer function defined by the app config.
	// this is only required if the chain wants to add special InitChainer logic.
	initChainer sdk.InitChainer
}

// RegisterModules registers the provided modules with the module manager and
// the basic module manager. This is the primary hook for integrating with
// modules which are not registered using the app config.
func (a *App) RegisterModules(modules ...module.AppModule) error {
	for _, appModule := range modules {
		name := appModule.Name()
		if _, ok := a.ModuleManager.Modules[name]; ok {
			return fmt.Errorf("AppModule named %q already exists", name)
		}

		if _, ok := a.basicManager[name]; ok {
			return fmt.Errorf("AppModuleBasic named %q already exists", name)
		}

		a.ModuleManager.Modules[name] = appModule
		a.basicManager[name] = appModule
		appModule.RegisterInterfaces(a.interfaceRegistry)
		appModule.RegisterLegacyAminoCodec(a.amino)

		if module, ok := appModule.(module.HasServices); ok {
			module.RegisterServices(a.configurator)
		}
	}

	return nil
}

// RegisterStores registers the provided store keys.
// This method should only be used for registering extra stores
// wiich is necessary for modules that not registered using the app config.
// To be used in combination of RegisterModules.
func (a *App) RegisterStores(keys ...storetypes.StoreKey) error {
	a.storeKeys = append(a.storeKeys, keys...)
	a.MountStores(keys...)

	return nil
}

// Load finishes all initialization operations and loads the app.
func (a *App) Load(loadLatest bool) error {
	// register runtime module services
	if err := a.registerRuntimeServices(); err != nil {
		return err
	}

	if len(a.config.InitGenesis) != 0 {
		a.ModuleManager.SetOrderInitGenesis(a.config.InitGenesis...)
		if a.initChainer == nil {
			a.SetInitChainer(a.InitChainer)
		}
	}

	if len(a.config.ExportGenesis) != 0 {
		a.ModuleManager.SetOrderExportGenesis(a.config.ExportGenesis...)
	} else if len(a.config.InitGenesis) != 0 {
		a.ModuleManager.SetOrderExportGenesis(a.config.InitGenesis...)
	}

	if len(a.config.BeginBlockers) != 0 {
		a.ModuleManager.SetOrderBeginBlockers(a.config.BeginBlockers...)
		a.SetBeginBlocker(a.BeginBlocker)
	}

	if len(a.config.EndBlockers) != 0 {
		a.ModuleManager.SetOrderEndBlockers(a.config.EndBlockers...)
		a.SetEndBlocker(a.EndBlocker)
	}

	if len(a.config.OrderMigrations) != 0 {
		a.ModuleManager.SetOrderMigrations(a.config.OrderMigrations...)
	}

	if loadLatest {
		if err := a.LoadLatestVersion(); err != nil {
			return err
		}
	}

	return nil
}

// BeginBlocker application updates every begin block
func (a *App) BeginBlocker(ctx sdk.Context, req abci.RequestBeginBlock) abci.ResponseBeginBlock {
	return a.ModuleManager.BeginBlock(ctx, req)
}

// EndBlocker application updates every end block
func (a *App) EndBlocker(ctx sdk.Context, req abci.RequestEndBlock) abci.ResponseEndBlock {
	return a.ModuleManager.EndBlock(ctx, req)
}

// InitChainer initializes the chain.
func (a *App) InitChainer(ctx sdk.Context, req abci.RequestInitChain) abci.ResponseInitChain {
	var genesisState map[string]json.RawMessage
	if err := json.Unmarshal(req.AppStateBytes, &genesisState); err != nil {
		panic(err)
	}
	return a.ModuleManager.InitGenesis(ctx, a.cdc, genesisState)
}

// RegisterAPIRoutes registers all application module routes with the provided
// API server.
func (a *App) RegisterAPIRoutes(apiSvr *api.Server, _ config.APIConfig) {
	clientCtx := apiSvr.ClientCtx
	// Register new tx routes from grpc-gateway.
	authtx.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)

	// Register new tendermint queries routes from grpc-gateway.
	tmservice.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)

	// Register node gRPC service for grpc-gateway.
	nodeservice.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)

	// Register grpc-gateway routes for all modules.
	a.basicManager.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
}

// RegisterTxService implements the Application.RegisterTxService method.
func (a *App) RegisterTxService(clientCtx client.Context) {
	authtx.RegisterTxService(a.GRPCQueryRouter(), clientCtx, a.Simulate, a.interfaceRegistry)
}

// RegisterTendermintService implements the Application.RegisterTendermintService method.
func (a *App) RegisterTendermintService(clientCtx client.Context) {
	tmservice.RegisterTendermintService(
		clientCtx,
		a.GRPCQueryRouter(),
		a.interfaceRegistry,
		a.Query,
	)
}

// RegisterNodeService registers the node gRPC service on the app gRPC router.
func (a *App) RegisterNodeService(clientCtx client.Context) {
	nodeservice.RegisterNodeService(clientCtx, a.GRPCQueryRouter())
}

// Configurator returns the app's configurator.
func (a *App) Configurator() module.Configurator {
	return a.configurator
}

// LoadHeight loads a particular height
func (a *App) LoadHeight(height int64) error {
	return a.LoadVersion(height)
}

// DefaultGenesis returns a default genesis from the registered AppModuleBasic's.
func (a *App) DefaultGenesis() map[string]json.RawMessage {
	return a.basicManager.DefaultGenesis(a.cdc)
}

// GetStoreKeys returns all the stored store keys.
func (a *App) GetStoreKeys() []storetypes.StoreKey {
	return a.storeKeys
}

// SetInitChainer sets the init chainer function
// It wraps `BaseApp.SetInitChainer` to allow setting a custom init chainer from an app.
func (a *App) SetInitChainer(initChainer sdk.InitChainer) {
	a.initChainer = initChainer
	a.BaseApp.SetInitChainer(initChainer)
}

// UnsafeFindStoreKey fetches a registered StoreKey from the App in linear time.
//
// NOTE: This should only be used in testing.
func (a *App) UnsafeFindStoreKey(storeKey string) storetypes.StoreKey {
	i := slices.IndexFunc(a.storeKeys, func(s storetypes.StoreKey) bool { return s.Name() == storeKey })
	if i == -1 {
		return nil
	}

	return a.storeKeys[i]
}

var _ servertypes.Application = &App{}
