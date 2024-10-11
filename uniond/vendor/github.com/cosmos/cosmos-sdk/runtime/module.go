package runtime

import (
	"fmt"
	"os"
	"slices"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoregistry"

	runtimev1alpha1 "cosmossdk.io/api/cosmos/app/runtime/v1alpha1"
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	reflectionv1 "cosmossdk.io/api/cosmos/reflection/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/comet"
	"cosmossdk.io/core/registry"
	"cosmossdk.io/core/store"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/std"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
	"github.com/cosmos/cosmos-sdk/x/genutil"
)

// appModule defines runtime as an AppModule
type appModule struct {
	app *App
}

func (m appModule) IsOnePerModuleType() {}
func (m appModule) IsAppModule()        {}

func (m appModule) RegisterServices(configurator module.Configurator) { //nolint:staticcheck // SA1019: Configurator is deprecated but still used in runtime v1.
	err := m.app.registerRuntimeServices(configurator)
	if err != nil {
		panic(err)
	}
}

func (m appModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			SubCommands: map[string]*autocliv1.ServiceCommandDescriptor{
				"autocli": {
					Service: autocliv1.Query_ServiceDesc.ServiceName,
					RpcCommandOptions: []*autocliv1.RpcCommandOptions{
						{
							RpcMethod: "AppOptions",
							Short:     "Query the custom autocli options",
						},
					},
				},
				"reflection": {
					Service: reflectionv1.ReflectionService_ServiceDesc.ServiceName,
					RpcCommandOptions: []*autocliv1.RpcCommandOptions{
						{
							RpcMethod: "FileDescriptors",
							Short:     "Query the app's protobuf file descriptors",
						},
					},
				},
			},
		},
	}
}

var (
	_ appmodule.AppModule = appModule{}
	_ module.HasServices  = appModule{}
)

// BaseAppOption is a depinject.AutoGroupType which can be used to pass
// BaseApp options into the depinject. It should be used carefully.
type BaseAppOption func(*baseapp.BaseApp)

// IsManyPerContainerType indicates that this is a depinject.ManyPerContainerType.
func (b BaseAppOption) IsManyPerContainerType() {}

func init() {
	appconfig.RegisterModule(&runtimev1alpha1.Module{},
		appconfig.Provide(
			ProvideApp,
			// to decouple runtime from sdk/codec ProvideInterfaceReistry can be registered from the app
			// i.e. in the call to depinject.Inject(...)
			codec.ProvideInterfaceRegistry,
			codec.ProvideLegacyAmino,
			codec.ProvideProtoCodec,
			codec.ProvideAddressCodec,
			ProvideKVStoreKey,
			ProvideTransientStoreKey,
			ProvideMemoryStoreKey,
			ProvideGenesisTxHandler,
			ProvideEnvironment,
			ProvideTransientStoreService,
			ProvideModuleManager,
			ProvideCometService,
		),
		appconfig.Invoke(SetupAppBuilder),
	)
}

func ProvideApp(
	interfaceRegistry codectypes.InterfaceRegistry,
	amino registry.AminoRegistrar,
	protoCodec *codec.ProtoCodec,
) (
	*AppBuilder,
	*baseapp.MsgServiceRouter,
	*baseapp.GRPCQueryRouter,
	appmodule.AppModule,
	protodesc.Resolver,
	protoregistry.MessageTypeResolver,
) {
	protoFiles := proto.HybridResolver
	protoTypes := protoregistry.GlobalTypes

	// At startup, check that all proto annotations are correct.
	if err := msgservice.ValidateProtoAnnotations(protoFiles); err != nil {
		// Once we switch to using protoreflect-based ante handlers, we might
		// want to panic here instead of logging a warning.
		_, _ = fmt.Fprintln(os.Stderr, err.Error())
	}

	std.RegisterInterfaces(interfaceRegistry)
	std.RegisterLegacyAminoCodec(amino)

	msgServiceRouter := baseapp.NewMsgServiceRouter()
	grpcQueryRouter := baseapp.NewGRPCQueryRouter()
	app := &App{
		storeKeys:         nil,
		interfaceRegistry: interfaceRegistry,
		cdc:               protoCodec,
		amino:             amino,
		msgServiceRouter:  msgServiceRouter,
		grpcQueryRouter:   grpcQueryRouter,
	}
	appBuilder := &AppBuilder{app: app}

	return appBuilder, msgServiceRouter, grpcQueryRouter, appModule{app}, protoFiles, protoTypes
}

type AppInputs struct {
	depinject.In

	Logger            log.Logger
	Config            *runtimev1alpha1.Module
	AppBuilder        *AppBuilder
	ModuleManager     *module.Manager
	BaseAppOptions    []BaseAppOption
	InterfaceRegistry codectypes.InterfaceRegistry
	LegacyAmino       registry.AminoRegistrar
	AppOptions        servertypes.AppOptions `optional:"true"` // can be nil in client wiring
}

func SetupAppBuilder(inputs AppInputs) {
	app := inputs.AppBuilder.app
	app.baseAppOptions = inputs.BaseAppOptions
	app.config = inputs.Config
	app.logger = inputs.Logger
	app.ModuleManager = inputs.ModuleManager
	app.ModuleManager.RegisterInterfaces(inputs.InterfaceRegistry)
	app.ModuleManager.RegisterLegacyAminoCodec(inputs.LegacyAmino)

	if inputs.AppOptions != nil {
		inputs.AppBuilder.appOptions = inputs.AppOptions
	}
}

func registerStoreKey(wrapper *AppBuilder, key storetypes.StoreKey) {
	wrapper.app.storeKeys = append(wrapper.app.storeKeys, key)
}

func storeKeyOverride(config *runtimev1alpha1.Module, moduleName string) *runtimev1alpha1.StoreKeyConfig {
	for _, cfg := range config.OverrideStoreKeys {
		if cfg.ModuleName == moduleName {
			return cfg
		}
	}
	return nil
}

func ProvideKVStoreKey(
	config *runtimev1alpha1.Module,
	key depinject.ModuleKey,
	app *AppBuilder,
) *storetypes.KVStoreKey {
	if slices.Contains(config.SkipStoreKeys, key.Name()) {
		return nil
	}

	override := storeKeyOverride(config, key.Name())

	var storeKeyName string
	if override != nil {
		storeKeyName = override.KvStoreKey
	} else {
		storeKeyName = key.Name()
	}

	storeKey := storetypes.NewKVStoreKey(storeKeyName)
	registerStoreKey(app, storeKey)
	return storeKey
}

func ProvideTransientStoreKey(
	config *runtimev1alpha1.Module,
	key depinject.ModuleKey,
	app *AppBuilder,
) *storetypes.TransientStoreKey {
	if slices.Contains(config.SkipStoreKeys, key.Name()) {
		return nil
	}

	storeKey := storetypes.NewTransientStoreKey(fmt.Sprintf("transient:%s", key.Name()))
	registerStoreKey(app, storeKey)
	return storeKey
}

func ProvideMemoryStoreKey(
	config *runtimev1alpha1.Module,
	key depinject.ModuleKey,
	app *AppBuilder,
) *storetypes.MemoryStoreKey {
	if slices.Contains(config.SkipStoreKeys, key.Name()) {
		return nil
	}

	storeKey := storetypes.NewMemoryStoreKey(fmt.Sprintf("memory:%s", key.Name()))
	registerStoreKey(app, storeKey)
	return storeKey
}

func ProvideModuleManager(modules map[string]appmodule.AppModule) *module.Manager {
	return module.NewManagerFromMap(modules)
}

func ProvideGenesisTxHandler(appBuilder *AppBuilder) genutil.TxHandler {
	return appBuilder.app
}

func ProvideEnvironment(
	logger log.Logger,
	config *runtimev1alpha1.Module,
	key depinject.ModuleKey,
	app *AppBuilder,
	msgServiceRouter *baseapp.MsgServiceRouter,
	queryServiceRouter *baseapp.GRPCQueryRouter,
) (store.KVStoreService, store.MemoryStoreService, appmodule.Environment) {
	var (
		kvService    store.KVStoreService     = failingStoreService{}
		memKvService store.MemoryStoreService = failingStoreService{}
	)

	// skips modules that have no store
	if !slices.Contains(config.SkipStoreKeys, key.Name()) {
		storeKey := ProvideKVStoreKey(config, key, app)
		kvService = kvStoreService{key: storeKey}

		memStoreKey := ProvideMemoryStoreKey(config, key, app)
		memKvService = memStoreService{key: memStoreKey}
	}

	return kvService, memKvService, NewEnvironment(
		kvService,
		logger.With(log.ModuleKey, fmt.Sprintf("x/%s", key.Name())),
		EnvWithMsgRouterService(msgServiceRouter),
		EnvWithQueryRouterService(queryServiceRouter),
		EnvWithMemStoreService(memKvService),
	)
}

func ProvideTransientStoreService(
	config *runtimev1alpha1.Module,
	key depinject.ModuleKey,
	app *AppBuilder,
) store.TransientStoreService {
	storeKey := ProvideTransientStoreKey(config, key, app)
	if storeKey == nil {
		return failingStoreService{}
	}

	return transientStoreService{key: storeKey}
}

func ProvideCometService() comet.Service {
	return NewContextAwareCometInfoService()
}
