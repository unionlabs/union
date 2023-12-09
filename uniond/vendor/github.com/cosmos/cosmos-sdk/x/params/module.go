package params

import (
	"context"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"

	modulev1 "cosmossdk.io/api/cosmos/params/module/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"
	store "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	govv1beta1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	"github.com/cosmos/cosmos-sdk/x/params/keeper"
	"github.com/cosmos/cosmos-sdk/x/params/types"
	"github.com/cosmos/cosmos-sdk/x/params/types/proposal"
)

var (
	_ module.AppModuleBasic      = AppModule{}
	_ module.AppModuleSimulation = AppModule{}
	_ module.HasServices         = AppModule{}

	_ appmodule.AppModule = AppModule{}
)

// ConsensusVersion defines the current x/params module consensus version.
const ConsensusVersion = 1

// AppModuleBasic defines the basic application module used by the params module.
type AppModuleBasic struct{}

// Name returns the params module's name.
func (AppModuleBasic) Name() string {
	return proposal.ModuleName
}

// RegisterLegacyAminoCodec registers the params module's types on the given LegacyAmino codec.
func (AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	proposal.RegisterLegacyAminoCodec(cdc)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the params module.
func (AppModuleBasic) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *gwruntime.ServeMux) {
	if err := proposal.RegisterQueryHandlerClient(context.Background(), mux, proposal.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

func (am AppModuleBasic) RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	proposal.RegisterInterfaces(registry)
}

// AppModule implements an application module for the distribution module.
type AppModule struct {
	AppModuleBasic

	keeper keeper.Keeper
}

// NewAppModule creates a new AppModule object
func NewAppModule(k keeper.Keeper) AppModule {
	return AppModule{
		AppModuleBasic: AppModuleBasic{},
		keeper:         k,
	}
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// GenerateGenesisState performs a no-op.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {}

// RegisterServices registers a gRPC query service to respond to the
// module-specific gRPC queries.
func (am AppModule) RegisterServices(cfg module.Configurator) {
	proposal.RegisterQueryServer(cfg.QueryServer(), am.keeper)
}

// RegisterStoreDecoder doesn't register any type.
func (AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {}

// WeightedOperations returns the all the gov module operations with their respective weights.
func (am AppModule) WeightedOperations(_ module.SimulationState) []simtypes.WeightedOperation {
	return nil
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }

//
// App Wiring Setup
//

func init() {
	appmodule.Register(&modulev1.Module{},
		appmodule.Provide(
			ProvideModule,
			ProvideSubspace,
		))
}

type ModuleInputs struct {
	depinject.In

	KvStoreKey        *store.KVStoreKey
	TransientStoreKey *store.TransientStoreKey
	Cdc               codec.Codec
	LegacyAmino       *codec.LegacyAmino
}

type ModuleOutputs struct {
	depinject.Out

	ParamsKeeper keeper.Keeper
	Module       appmodule.AppModule
	GovHandler   govv1beta1.HandlerRoute
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	k := keeper.NewKeeper(in.Cdc, in.LegacyAmino, in.KvStoreKey, in.TransientStoreKey)

	m := NewAppModule(k)
	govHandler := govv1beta1.HandlerRoute{RouteKey: proposal.RouterKey, Handler: NewParamChangeProposalHandler(k)}

	return ModuleOutputs{ParamsKeeper: k, Module: m, GovHandler: govHandler}
}

type SubspaceInputs struct {
	depinject.In

	Key       depinject.ModuleKey
	Keeper    keeper.Keeper
	KeyTables map[string]types.KeyTable
}

func ProvideSubspace(in SubspaceInputs) types.Subspace {
	moduleName := in.Key.Name()
	kt, exists := in.KeyTables[moduleName]
	if !exists {
		return in.Keeper.Subspace(moduleName)
	}
	return in.Keeper.Subspace(moduleName).WithKeyTable(kt)
}
