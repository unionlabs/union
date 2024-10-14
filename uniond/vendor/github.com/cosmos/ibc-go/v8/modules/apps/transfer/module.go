package transfer

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	"google.golang.org/grpc"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/client/cli"
	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/keeper"
	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/simulation"
	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
)

var (
	_ module.AppModule                = (*AppModule)(nil)
	_ module.AppModuleBasic           = (*AppModule)(nil)
	_ module.AppModuleSimulation      = (*AppModule)(nil)
	_ module.HasGenesis               = (*AppModule)(nil)
	_ appmodule.HasConsensusVersion   = (*AppModule)(nil)
	_ module.HasInvariants            = (*AppModule)(nil)
	_ appmodule.AppModule             = (*AppModule)(nil)
	_ appmodule.HasMigrations         = AppModule{}
	_ appmodule.HasRegisterInterfaces = AppModule{}

	_ porttypes.IBCModule = (*IBCModule)(nil)
)

// Name implements AppModuleBasic interface
func (AppModule) Name() string {
	return types.ModuleName
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// RegisterLegacyAminoCodec implements AppModuleBasic interface
func (AppModule) RegisterLegacyAminoCodec(cdc registry.AminoRegistrar) {
	types.RegisterLegacyAminoCodec(cdc)
}

// RegisterInterfaces registers module concrete types into protobuf Any.
func (AppModule) RegisterInterfaces(registry registry.InterfaceRegistrar) {
	types.RegisterInterfaces(registry)
}

// DefaultGenesis returns default genesis state as raw bytes for the ibc
// transfer module.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(types.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the ibc transfer module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var gs types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &gs); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return gs.Validate()
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the ibc-transfer module.
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *runtime.ServeMux) {
	err := types.RegisterQueryHandlerClient(context.Background(), mux, types.NewQueryClient(clientCtx))
	if err != nil {
		panic(err)
	}
}

// GetTxCmd implements AppModuleBasic interface
func (AppModule) GetTxCmd() *cobra.Command {
	return cli.NewTxCmd()
}

// GetQueryCmd implements AppModuleBasic interface
func (AppModule) GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}

// AppModule represents the AppModule for this module
type AppModule struct {
	cdc    codec.Codec
	keeper keeper.Keeper
}

// NewAppModule creates a new 20-transfer module
func NewAppModule(cdc codec.Codec, k keeper.Keeper) AppModule {
	return AppModule{
		cdc:    cdc,
		keeper: k,
	}
}

// RegisterInvariants implements the AppModule interface
func (am AppModule) RegisterInvariants(ir sdk.InvariantRegistry) {
	keeper.RegisterInvariants(ir, &am.keeper)
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	types.RegisterMsgServer(registrar, am.keeper)
	types.RegisterQueryServer(registrar, am.keeper)
	return nil
}

func (am AppModule) RegisterMigrations(mr appmodule.MigrationRegistrar) error {
	m := keeper.NewMigrator(am.keeper)
	if err := mr.Register(types.ModuleName, 1, m.MigrateTraces); err != nil {
		panic(fmt.Errorf("failed to migrate transfer app from version 1 to 2 (denom trace format migration): %v", err))
	}

	if err := mr.Register(types.ModuleName, 2, m.MigrateTotalEscrowForDenom); err != nil {
		panic(fmt.Errorf("failed to migrate transfer app from version 2 to 3 (total escrow entry migration): %v", err))
	}

	if err := mr.Register(types.ModuleName, 3, m.MigrateParams); err != nil {
		panic(fmt.Errorf("failed to migrate transfer app version 3 to 4 (self-managed params migration): %v", err))
	}

	if err := mr.Register(types.ModuleName, 4, m.MigrateDenomMetadata); err != nil {
		panic(fmt.Errorf("failed to migrate transfer app from version 4 to 5 (set denom metadata migration): %v", err))
	}
	return nil
}

// InitGenesis performs genesis initialization for the ibc-transfer module. It returns
// no validator updates.
func (am AppModule) InitGenesis(ctx context.Context, data json.RawMessage) error {
	var genesisState types.GenesisState
	am.cdc.MustUnmarshalJSON(data, &genesisState)
	am.keeper.InitGenesis(ctx, genesisState)
	return nil
}

// ExportGenesis returns the exported genesis state as raw bytes for the ibc-transfer
// module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs := am.keeper.ExportGenesis(ctx)
	return am.cdc.MarshalJSON(gs)
}

// ConsensusVersion implements AppModule/ConsensusVersion defining the current version of transfer.
func (AppModule) ConsensusVersion() uint64 { return 5 }

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the transfer module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// ProposalMsgs returns msgs used for governance proposals for simulations.
func (AppModule) ProposalMsgs(simState module.SimulationState) []simtypes.WeightedProposalMsg {
	return simulation.ProposalMsgs()
}

// RegisterStoreDecoder registers a decoder for transfer module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[types.StoreKey] = simulation.NewDecodeStore(am.keeper)
}

// WeightedOperations returns the all the transfer module operations with their respective weights.
func (AppModule) WeightedOperations(_ module.SimulationState) []simtypes.WeightedOperation {
	return nil
}
