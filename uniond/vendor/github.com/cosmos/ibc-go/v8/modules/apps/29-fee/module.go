package fee

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/client/cli"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/keeper"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
)

var (
	_ module.AppModule              = (*AppModule)(nil)
	_ module.AppModuleSimulation    = (*AppModule)(nil)
	_ module.HasGenesis             = (*AppModule)(nil)
	_ appmodule.HasConsensusVersion = (*AppModule)(nil)
	_ module.HasServices            = (*AppModule)(nil)
	_ appmodule.AppModule           = (*AppModule)(nil)
)

// AppModuleBasic is the 29-fee AppModuleBasic
type AppModuleBasic struct{}

// Name implements AppModuleBasic interface
func (AppModuleBasic) Name() string {
	return types.ModuleName
}

func (AppModule) Name() string {
	return types.ModuleName
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// RegisterLegacyAminoCodec implements AppModuleBasic interface
func (AppModuleBasic) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	types.RegisterLegacyAminoCodec(registrar)
}

// RegisterInterfaces registers module concrete types into protobuf Any.
func (AppModuleBasic) RegisterInterfaces(registry registry.InterfaceRegistrar) {
	types.RegisterInterfaces(registry)
}

// DefaultGenesis returns default genesis state as raw bytes for the ibc
// 29-fee module.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(types.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the 29-fee module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var gs types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &gs); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return gs.Validate()
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for ics29 fee module.
func (AppModuleBasic) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *runtime.ServeMux) {
	err := types.RegisterQueryHandlerClient(context.Background(), mux, types.NewQueryClient(clientCtx))
	if err != nil {
		panic(err)
	}
}

// GetTxCmd implements AppModuleBasic interface
func (AppModuleBasic) GetTxCmd() *cobra.Command {
	return cli.NewTxCmd()
}

// GetQueryCmd implements AppModuleBasic interface
func (AppModuleBasic) GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}

// AppModule represents the AppModule for this module
type AppModule struct {
	cdc    codec.Codec
	keeper keeper.Keeper
}

// NewAppModule creates a new 29-fee module
func NewAppModule(cdc codec.Codec, k keeper.Keeper) AppModule {
	return AppModule{
		cdc:    cdc,
		keeper: k,
	}
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(cfg module.Configurator) {
	types.RegisterMsgServer(cfg.MsgServer(), am.keeper)
	types.RegisterQueryServer(cfg.QueryServer(), am.keeper)

	m := keeper.NewMigrator(am.keeper)
	if err := cfg.RegisterMigration(types.ModuleName, 1, m.Migrate1to2); err != nil {
		panic(fmt.Errorf("failed to migrate ibc-fee module from version 1 to 2 (refund leftover fees): %v", err))
	}
}

// InitGenesis performs genesis initialization for the ibc-29-fee module. It returns
// no validator updates.
func (am AppModule) InitGenesis(ctx context.Context, data json.RawMessage) error {
	var genesisState types.GenesisState
	am.cdc.MustUnmarshalJSON(data, &genesisState)
	am.keeper.InitGenesis(ctx, genesisState)
	return nil
}

// ExportGenesis returns the exported genesis state as raw bytes for the ibc-29-fee
// module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs := am.keeper.ExportGenesis(ctx)
	return am.cdc.MarshalJSON(gs)
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return 2 }

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the 29-fee module.
func (AppModule) GenerateGenesisState(_ *module.SimulationState) {
}

// RegisterStoreDecoder registers a decoder for 29-fee module's types
func (AppModule) RegisterStoreDecoder(_ simtypes.StoreDecoderRegistry) {
}

// WeightedOperations returns the all the 29-fee module operations with their respective weights.
func (AppModule) WeightedOperations(_ module.SimulationState) []simtypes.WeightedOperation {
	return nil
}
