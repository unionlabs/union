package wasm

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"cosmossdk.io/core/appmodule"
	coreregistry "cosmossdk.io/core/registry"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/client/cli"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/keeper"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/simulation"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

var (
	_ module.AppModule              = (*AppModule)(nil)
	_ module.AppModuleBasic         = (*AppModule)(nil)
	_ module.HasGenesis             = (*AppModule)(nil)
	_ appmodule.HasConsensusVersion = (*AppModule)(nil)
	_ module.HasServices            = (*AppModule)(nil)
	_ appmodule.AppModule           = (*AppModule)(nil)
)

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// AppModuleBasic defines the basic application module used by the Wasm light client.
// Only the RegisterInterfaces function needs to be implemented. All other function perform
// a no-op.
type AppModuleBasic struct {
	cdc codec.Codec
}

func NewAppModuleBasic(cdc codec.Codec) AppModuleBasic {
	return AppModuleBasic{
		cdc: cdc,
	}
}

// Name returns the tendermint module name.
func (AppModuleBasic) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec performs a no-op. The Wasm client does not support amino.
func (AppModule) RegisterLegacyAminoCodec(cdc coreregistry.AminoRegistrar) {}

// RegisterInterfaces registers module concrete types into protobuf Any. This allows core IBC
// to unmarshal Wasm light client types.
func (AppModuleBasic) RegisterInterfaces(registry coreregistry.InterfaceRegistrar) {
	types.RegisterInterfaces(registry)
}

// DefaultGenesis returns an empty state, i.e. no contracts
func (am AppModuleBasic) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(&types.GenesisState{
		Contracts: []types.Contract{},
	})
}

// ValidateGenesis performs a no-op.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var gs types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &gs); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return gs.Validate()
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for Wasm client module.
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
	AppModuleBasic
	keeper keeper.Keeper
}

// NewAppModule creates a new 08-wasm module
func NewAppModule(cdc codec.Codec, k keeper.Keeper) AppModule {
	return AppModule{
		AppModuleBasic: NewAppModuleBasic(cdc),
		keeper:         k,
	}
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(cfg module.Configurator) {
	types.RegisterMsgServer(cfg.MsgServer(), am.keeper)
	types.RegisterQueryServer(cfg.QueryServer(), am.keeper)

	wasmMigrator := keeper.NewMigrator(am.keeper)
	if err := cfg.RegisterMigration(types.ModuleName, 1, wasmMigrator.MigrateChecksums); err != nil {
		panic(fmt.Errorf("failed to migrate 08-wasm module from version 1 to 2 (checksums migration to collections): %v", err))
	}
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return 2 }

// ProposalMsgs returns msgs used for governance proposals for simulations.
func (AppModule) ProposalMsgs(simState module.SimulationState) []simtypes.WeightedProposalMsg {
	return simulation.ProposalMsgs()
}

func (am AppModule) InitGenesis(ctx context.Context, data json.RawMessage) error {
	var genesisState types.GenesisState
	am.cdc.MustUnmarshalJSON(data, &genesisState)
	am.keeper.InitGenesis(ctx, genesisState)
	return nil
}

func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs := am.keeper.ExportGenesis(ctx)
	return am.cdc.MarshalJSON(gs)
}
