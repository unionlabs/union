package ibc

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

	ibcclient "github.com/cosmos/ibc-go/v8/modules/core/02-client"
	clientkeeper "github.com/cosmos/ibc-go/v8/modules/core/02-client/keeper"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connection "github.com/cosmos/ibc-go/v8/modules/core/03-connection"
	connectionkeeper "github.com/cosmos/ibc-go/v8/modules/core/03-connection/keeper"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channel "github.com/cosmos/ibc-go/v8/modules/core/04-channel"
	channelkeeper "github.com/cosmos/ibc-go/v8/modules/core/04-channel/keeper"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	"github.com/cosmos/ibc-go/v8/modules/core/client/cli"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	"github.com/cosmos/ibc-go/v8/modules/core/keeper"
	"github.com/cosmos/ibc-go/v8/modules/core/simulation"
	"github.com/cosmos/ibc-go/v8/modules/core/types"
)

var (
	_ module.AppModule           = (*AppModule)(nil)
	_ module.AppModuleSimulation = (*AppModule)(nil)
	_ module.HasGenesis          = (*AppModule)(nil)

	_ appmodule.HasConsensusVersion = (*AppModule)(nil)
	_ appmodule.AppModule           = (*AppModule)(nil)
	_ appmodule.HasBeginBlocker     = (*AppModule)(nil)

	_ appmodule.HasMigrations         = AppModule{}
	_ appmodule.HasGenesis            = AppModule{}
	_ appmodule.HasRegisterInterfaces = AppModule{}
)

// AppModuleBasic defines the basic application module used by the ibc module.
type AppModuleBasic struct {
	cdc codec.Codec
}

// Name returns the ibc module's name.
func (AppModuleBasic) Name() string {
	return exported.ModuleName
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// RegisterLegacyAminoCodec does nothing. IBC does not support amino.
func (AppModuleBasic) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {}

// DefaultGenesis returns default genesis state as raw bytes for the ibc
// module.
func (am AppModuleBasic) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(types.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the ibc module.
func (am AppModuleBasic) ValidateGenesis(bz json.RawMessage) error {
	var gs types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &gs); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", exported.ModuleName, err)
	}

	return gs.Validate()
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the ibc module.
func (AppModuleBasic) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *runtime.ServeMux) {
	err := clienttypes.RegisterQueryHandlerClient(context.Background(), mux, clienttypes.NewQueryClient(clientCtx))
	if err != nil {
		panic(err)
	}
	err = connectiontypes.RegisterQueryHandlerClient(context.Background(), mux, connectiontypes.NewQueryClient(clientCtx))
	if err != nil {
		panic(err)
	}
	err = channeltypes.RegisterQueryHandlerClient(context.Background(), mux, channeltypes.NewQueryClient(clientCtx))
	if err != nil {
		panic(err)
	}
}

// GetTxCmd returns the root tx command for the ibc module.
func (AppModuleBasic) GetTxCmd() *cobra.Command {
	return cli.GetTxCmd()
}

// GetQueryCmd returns no root query command for the ibc module.
func (AppModuleBasic) GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}

// RegisterInterfaces registers module concrete types into protobuf Any.
func (AppModuleBasic) RegisterInterfaces(registry registry.InterfaceRegistrar) {
	types.RegisterInterfaces(registry)
}

// RegisterInterfaces registers module concrete types into protobuf Any.
func (AppModule) RegisterInterfaces(registry registry.InterfaceRegistrar) {
	types.RegisterInterfaces(registry)
}

// AppModule implements an application module for the ibc module.
type AppModule struct {
	AppModuleBasic
	keeper *keeper.Keeper
}

// NewAppModule creates a new AppModule object
func NewAppModule(cdc codec.Codec, k *keeper.Keeper) AppModule {
	return AppModule{
		keeper: k,
		AppModuleBasic: AppModuleBasic{
			cdc: cdc,
		},
	}
}

// Name returns the ibc module's name.
func (AppModule) Name() string {
	return exported.ModuleName
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	clienttypes.RegisterMsgServer(registrar, am.keeper)
	connectiontypes.RegisterMsgServer(registrar, am.keeper)
	channeltypes.RegisterMsgServer(registrar, am.keeper)
	ibcclient.RegisterQueryService(registrar, am.keeper)
	connection.RegisterQueryService(registrar, am.keeper)
	channel.RegisterQueryService(registrar, am.keeper)

	return nil
}

func (am AppModule) RegisterMigrations(mr appmodule.MigrationRegistrar) error {
	clientMigrator := clientkeeper.NewMigrator(am.keeper.ClientKeeper)
	if err := mr.Register(exported.ModuleName, 2, clientMigrator.Migrate2to3); err != nil {
		panic(err)
	}

	connectionMigrator := connectionkeeper.NewMigrator(am.keeper.ConnectionKeeper)
	if err := mr.Register(exported.ModuleName, 3, connectionMigrator.Migrate3to4); err != nil {
		panic(err)
	}

	if err := mr.Register(exported.ModuleName, 4, func(bareCtx context.Context) error {
		ctx := sdk.UnwrapSDKContext(bareCtx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
		if err := clientMigrator.MigrateParams(ctx); err != nil {
			return err
		}
		return connectionMigrator.MigrateParams(ctx)
	}); err != nil {
		panic(err)
	}

	channelMigrator := channelkeeper.NewMigrator(am.keeper.ChannelKeeper)
	err := mr.Register(exported.ModuleName, 5, channelMigrator.MigrateParams)
	if err != nil {
		panic(err)
	}
	return nil
}

// InitGenesis performs genesis initialization for the ibc module. It returns
// no validator updates.
func (am AppModule) InitGenesis(ctx context.Context, bz json.RawMessage) error {
	var gs types.GenesisState
	err := am.cdc.UnmarshalJSON(bz, &gs)
	if err != nil {
		panic(fmt.Errorf("failed to unmarshal %s genesis state: %s", exported.ModuleName, err))
	}
	return InitGenesis(ctx, *am.keeper, &gs)
}

// ExportGenesis returns the exported genesis state as raw bytes for the ibc
// module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs, err := ExportGenesis(ctx, *am.keeper)
	if err != nil {
		return nil, err
	}
	return am.cdc.MarshalJSON(gs)
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return 6 }

// BeginBlock returns the begin blocker for the ibc module.
func (am AppModule) BeginBlock(ctx context.Context) error {
	ibcclient.BeginBlocker(sdk.UnwrapSDKContext(ctx), am.keeper.ClientKeeper)
	return nil
}

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the ibc module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// ProposalMsgs returns msgs used for governance proposals for simulations.
func (AppModule) ProposalMsgs(simState module.SimulationState) []simtypes.WeightedProposalMsg {
	return simulation.ProposalMsgs()
}

// RegisterStoreDecoder registers a decoder for ibc module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[exported.StoreKey] = simulation.NewDecodeStore(*am.keeper)
}

// WeightedOperations returns the all the ibc module operations with their respective weights.
func (AppModule) WeightedOperations(_ module.SimulationState) []simtypes.WeightedOperation {
	return nil
}
