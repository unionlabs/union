package runtime

import (
	"encoding/json"
	"fmt"
	"slices"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtcrypto "github.com/cometbft/cometbft/crypto"
	cmted25519 "github.com/cometbft/cometbft/crypto/ed25519"
	"google.golang.org/grpc"

	runtimev1alpha1 "cosmossdk.io/api/cosmos/app/runtime/v1alpha1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/grpc/cmtservice"
	nodeservice "github.com/cosmos/cosmos-sdk/client/grpc/node"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/server"
	"github.com/cosmos/cosmos-sdk/server/api"
	"github.com/cosmos/cosmos-sdk/server/config"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/x/auth/ante/unorderedtx"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

// KeyGenF is a function that generates a private key for use by comet.
type KeyGenF = func() (cmtcrypto.PrivKey, error)

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

	ModuleManager      *module.Manager
	UnorderedTxManager *unorderedtx.Manager

	configurator      module.Configurator //nolint:staticcheck // SA1019: Configurator is deprecated but still used in runtime v1.
	config            *runtimev1alpha1.Module
	storeKeys         []storetypes.StoreKey
	interfaceRegistry codectypes.InterfaceRegistry
	cdc               codec.Codec
	amino             registry.AminoRegistrar
	baseAppOptions    []BaseAppOption
	msgServiceRouter  *baseapp.MsgServiceRouter
	grpcQueryRouter   *baseapp.GRPCQueryRouter
	logger            log.Logger
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

		a.ModuleManager.Modules[name] = appModule
		if mod, ok := appModule.(appmodule.HasRegisterInterfaces); ok {
			mod.RegisterInterfaces(a.interfaceRegistry)
		}

		if mod, ok := appModule.(module.HasAminoCodec); ok {
			mod.RegisterLegacyAminoCodec(a.amino)
		}

		if mod, ok := appModule.(module.HasServices); ok {
			mod.RegisterServices(a.configurator)
		} else if module, ok := appModule.(hasServicesV1); ok {
			if err := module.RegisterServices(a.configurator); err != nil {
				return err
			}
		}
	}

	return nil
}

// RegisterStores registers the provided store keys.
// This method should only be used for registering extra stores
// which is necessary for modules that not registered using the app config.
// To be used in combination of RegisterModules.
func (a *App) RegisterStores(keys ...storetypes.StoreKey) error {
	a.storeKeys = append(a.storeKeys, keys...)
	a.MountStores(keys...)

	return nil
}

// Load finishes all initialization operations and loads the app.
func (a *App) Load(loadLatest bool) error {
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

	if len(a.config.PreBlockers) != 0 {
		a.ModuleManager.SetOrderPreBlockers(a.config.PreBlockers...)
		if a.BaseApp.PreBlocker() == nil {
			a.SetPreBlocker(a.PreBlocker)
		}
	}

	if len(a.config.BeginBlockers) != 0 {
		a.ModuleManager.SetOrderBeginBlockers(a.config.BeginBlockers...)
		a.SetBeginBlocker(a.BeginBlocker)
	}

	if len(a.config.EndBlockers) != 0 {
		a.ModuleManager.SetOrderEndBlockers(a.config.EndBlockers...)
		a.SetEndBlocker(a.EndBlocker)
	}

	if len(a.config.Precommiters) != 0 {
		a.ModuleManager.SetOrderPrecommiters(a.config.Precommiters...)
		a.SetPrecommiter(a.Precommiter)
	}

	if len(a.config.PrepareCheckStaters) != 0 {
		a.ModuleManager.SetOrderPrepareCheckStaters(a.config.PrepareCheckStaters...)
		a.SetPrepareCheckStater(a.PrepareCheckStater)
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

// Close closes all necessary application resources.
// It implements servertypes.Application.
func (a *App) Close() error {
	// the unordered tx manager could be nil (unlikely but possible)
	// if the app has no app options supplied.
	if a.UnorderedTxManager != nil {
		if err := a.UnorderedTxManager.Close(); err != nil {
			return err
		}
	}

	return a.BaseApp.Close()
}

// PreBlocker application updates every pre block
func (a *App) PreBlocker(ctx sdk.Context, _ *abci.FinalizeBlockRequest) error {
	if a.UnorderedTxManager != nil {
		a.UnorderedTxManager.OnNewBlock(ctx.BlockTime())
	}
	return a.ModuleManager.PreBlock(ctx)
}

// BeginBlocker application updates every begin block
func (a *App) BeginBlocker(ctx sdk.Context) (sdk.BeginBlock, error) {
	return a.ModuleManager.BeginBlock(ctx)
}

// EndBlocker application updates every end block
func (a *App) EndBlocker(ctx sdk.Context) (sdk.EndBlock, error) {
	return a.ModuleManager.EndBlock(ctx)
}

// Precommiter application updates every commit
func (a *App) Precommiter(ctx sdk.Context) {
	if err := a.ModuleManager.Precommit(ctx); err != nil {
		panic(err)
	}
}

// PrepareCheckStater application updates every commit
func (a *App) PrepareCheckStater(ctx sdk.Context) {
	if err := a.ModuleManager.PrepareCheckState(ctx); err != nil {
		panic(err)
	}
}

// InitChainer initializes the chain.
func (a *App) InitChainer(ctx sdk.Context, req *abci.InitChainRequest) (*abci.InitChainResponse, error) {
	var genesisState map[string]json.RawMessage
	if err := json.Unmarshal(req.AppStateBytes, &genesisState); err != nil {
		return nil, err
	}
	return a.ModuleManager.InitGenesis(ctx, genesisState)
}

// RegisterAPIRoutes registers all application module routes with the provided
// API server.
func (a *App) RegisterAPIRoutes(apiSvr *api.Server, _ config.APIConfig) {
	clientCtx := apiSvr.ClientCtx
	// Register new tx routes from grpc-gateway.
	authtx.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)

	// Register new CometBFT queries routes from grpc-gateway.
	cmtservice.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)

	// Register node gRPC service for grpc-gateway.
	nodeservice.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)

	// Register grpc-gateway routes for all modules.
	a.ModuleManager.RegisterGRPCGatewayRoutes(clientCtx, apiSvr.GRPCGatewayRouter)
}

// RegisterTxService implements the Application.RegisterTxService method.
func (a *App) RegisterTxService(clientCtx client.Context) {
	authtx.RegisterTxService(a.GRPCQueryRouter(), clientCtx, a.Simulate, a.interfaceRegistry)
}

// RegisterTendermintService implements the Application.RegisterTendermintService method.
func (a *App) RegisterTendermintService(clientCtx client.Context) {
	cmtApp := server.NewCometABCIWrapper(a)
	cmtservice.RegisterTendermintService(
		clientCtx,
		a.GRPCQueryRouter(),
		a.interfaceRegistry,
		cmtApp.Query,
	)
}

// RegisterNodeService registers the node gRPC service on the app gRPC router.
func (a *App) RegisterNodeService(clientCtx client.Context, cfg config.Config) {
	nodeservice.RegisterNodeService(clientCtx, a.GRPCQueryRouter(), cfg)
}

// Configurator returns the app's configurator.
func (a *App) Configurator() module.Configurator { //nolint:staticcheck // SA1019: Configurator is deprecated but still used in runtime v1.
	return a.configurator
}

// LoadHeight loads a particular height
func (a *App) LoadHeight(height int64) error {
	return a.LoadVersion(height)
}

// DefaultGenesis returns a default genesis from the registered AppModule's.
func (a *App) DefaultGenesis() map[string]json.RawMessage {
	return a.ModuleManager.DefaultGenesis()
}

// SetInitChainer sets the init chainer function
// It wraps `BaseApp.SetInitChainer` to allow setting a custom init chainer from an app.
func (a *App) SetInitChainer(initChainer sdk.InitChainer) {
	a.initChainer = initChainer
	a.BaseApp.SetInitChainer(initChainer)
}

// GetStoreKeys returns all the stored store keys.
func (a *App) GetStoreKeys() []storetypes.StoreKey {
	return a.storeKeys
}

// GetKey returns the KVStoreKey for the provided store key.
//
// NOTE: This should only be used in testing.
func (a *App) GetKey(storeKey string) *storetypes.KVStoreKey {
	sk := a.UnsafeFindStoreKey(storeKey)
	kvStoreKey, ok := sk.(*storetypes.KVStoreKey)
	if !ok {
		return nil
	}
	return kvStoreKey
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

// hasServicesV1 is the interface for registering service in baseapp Cosmos SDK.
// This API is part of core/appmodule but commented out for dependencies.
type hasServicesV1 interface {
	RegisterServices(grpc.ServiceRegistrar) error
}

// ValidatorKeyProvider returns a function that generates a private key for use by comet.
func (a *App) ValidatorKeyProvider() KeyGenF {
	return func() (cmtcrypto.PrivKey, error) {
		return cmted25519.GenPrivKey(), nil
	}
}
