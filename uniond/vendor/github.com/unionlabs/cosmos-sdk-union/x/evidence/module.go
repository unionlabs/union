package evidence

import (
	"context"
	"encoding/json"
	"fmt"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/comet"
	"cosmossdk.io/core/registry"
	eviclient "cosmossdk.io/x/evidence/client"
	"cosmossdk.io/x/evidence/client/cli"
	"cosmossdk.io/x/evidence/keeper"
	"cosmossdk.io/x/evidence/simulation"
	"cosmossdk.io/x/evidence/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

var (
	_ module.HasAminoCodec       = AppModule{}
	_ module.HasGRPCGateway      = AppModule{}
	_ module.AppModuleSimulation = AppModule{}

	_ appmodule.AppModule             = AppModule{}
	_ appmodule.HasBeginBlocker       = AppModule{}
	_ appmodule.HasRegisterInterfaces = AppModule{}
	_ appmodule.HasGenesis            = AppModule{}
)

const ConsensusVersion = 1

// AppModule implements the AppModule interface for the evidence module.
type AppModule struct {
	cdc              codec.Codec
	evidenceHandlers []eviclient.EvidenceHandler
	keeper           keeper.Keeper
	cometService     comet.Service
}

// NewAppModule creates a new AppModule object.
func NewAppModule(cdc codec.Codec, keeper keeper.Keeper, cometService comet.Service, evidenceHandlers ...eviclient.EvidenceHandler) AppModule {
	return AppModule{
		keeper:           keeper,
		evidenceHandlers: evidenceHandlers,
		cdc:              cdc,
		cometService:     cometService,
	}
}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// Name returns the evidence module's name.
// Deprecated: kept for legacy reasons.
func (AppModule) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec registers the evidence module's types to the LegacyAmino codec.
func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	types.RegisterLegacyAminoCodec(registrar)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the evidence module.
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *gwruntime.ServeMux) {
	if err := types.RegisterQueryHandlerClient(context.Background(), mux, types.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// GetTxCmd returns the evidence module's root tx command.
func (am AppModule) GetTxCmd() *cobra.Command {
	evidenceCLIHandlers := make([]*cobra.Command, len(am.evidenceHandlers))
	for i, evidenceHandler := range am.evidenceHandlers {
		evidenceCLIHandlers[i] = evidenceHandler.CLIHandler()
	}

	return cli.GetTxCmd(evidenceCLIHandlers)
}

// RegisterInterfaces registers the evidence module's interface types
func (AppModule) RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	types.RegisterInterfaces(registrar)
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	types.RegisterMsgServer(registrar, keeper.NewMsgServerImpl(am.keeper))
	types.RegisterQueryServer(registrar, keeper.NewQuerier(&am.keeper))
	return nil
}

// DefaultGenesis returns the evidence module's default genesis state.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(types.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the evidence module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var gs types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &gs); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return gs.Validate()
}

// InitGenesis performs the evidence module's genesis initialization
func (am AppModule) InitGenesis(ctx context.Context, bz json.RawMessage) error {
	var gs types.GenesisState
	err := am.cdc.UnmarshalJSON(bz, &gs)
	if err != nil {
		return err
	}

	return InitGenesis(ctx, am.keeper, &gs)
}

// ExportGenesis returns the evidence module's exported genesis state as raw JSON bytes.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs, err := ExportGenesis(ctx, am.keeper)
	if err != nil {
		return nil, err
	}
	return am.cdc.MarshalJSON(gs)
}

// ConsensusVersion implements HasConsensusVersion
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }

// BeginBlock executes all ABCI BeginBlock logic respective to the evidence module.
func (am AppModule) BeginBlock(ctx context.Context) error {
	return am.keeper.BeginBlocker(ctx, am.cometService)
}

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the evidence module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// RegisterStoreDecoder registers a decoder for evidence module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[types.StoreKey] = simtypes.NewStoreDecoderFuncFromCollectionsSchema(am.keeper.Schema)
}
