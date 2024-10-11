package params

import (
	"context"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"google.golang.org/grpc"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"
	"cosmossdk.io/x/params/keeper"
	"cosmossdk.io/x/params/types/proposal"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

var (
	_ module.HasAminoCodec       = AppModule{}
	_ module.HasGRPCGateway      = AppModule{}
	_ module.AppModuleSimulation = AppModule{}

	_ appmodule.AppModule             = AppModule{}
	_ appmodule.HasRegisterInterfaces = AppModule{}
)

// ConsensusVersion defines the current x/params module consensus version.
const ConsensusVersion = 1

// AppModule implements an application module for the distribution module.
type AppModule struct {
	keeper keeper.Keeper
}

// NewAppModule creates a new AppModule object
func NewAppModule(k keeper.Keeper) AppModule {
	return AppModule{
		keeper: k,
	}
}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// Name returns the params module's name.
// Deprecated: kept for legacy reasons.
func (AppModule) Name() string {
	return proposal.ModuleName
}

// RegisterLegacyAminoCodec registers the params module's types on the given LegacyAmino codec.
func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	proposal.RegisterLegacyAminoCodec(registrar)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the params module.
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *gwruntime.ServeMux) {
	if err := proposal.RegisterQueryHandlerClient(context.Background(), mux, proposal.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// RegisterInterfaces registers the module's interface types
func (AppModule) RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	proposal.RegisterInterfaces(registrar)
}

// GenerateGenesisState performs a no-op.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	proposal.RegisterQueryServer(registrar, am.keeper)

	return nil
}

// RegisterStoreDecoder doesn't register any type.
func (AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {}

// ConsensusVersion implements HasConsensusVersion
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }
