package ica

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	"cosmossdk.io/core/appmodule"
	coreregistry "cosmossdk.io/core/registry"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/client/cli"
	controllerkeeper "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/keeper"
	controllertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	genesistypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/genesis/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host"
	hostkeeper "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/keeper"
	hosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/simulation"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
)

var (
	_ module.AppModule           = (*AppModule)(nil)
	_ module.AppModuleBasic      = (*AppModule)(nil)
	_ module.AppModuleSimulation = (*AppModule)(nil)
	_ module.HasGenesis          = (*AppModule)(nil)
	_ module.HasServices         = (*AppModule)(nil)
	_ appmodule.AppModule        = (*AppModule)(nil)

	_ porttypes.IBCModule = (*host.IBCModule)(nil)
)

// Name implements AppModule interface
func (AppModule) Name() string {
	return types.ModuleName
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// RegisterLegacyAminoCodec implements AppModule.
func (AppModule) RegisterLegacyAminoCodec(cdc coreregistry.AminoRegistrar) {}

// RegisterInterfaces registers module concrete types into protobuf Any
func (AppModule) RegisterInterfaces(registry coreregistry.InterfaceRegistrar) {
	controllertypes.RegisterInterfaces(registry)
	hosttypes.RegisterInterfaces(registry)
	types.RegisterInterfaces(registry)
}

// DefaultGenesis returns default genesis state as raw bytes for the IBC
// interchain accounts module
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(genesistypes.DefaultGenesis())
}

// ValidateGenesis performs genesis state validation for the IBC interchain acounts module
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var gs genesistypes.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &gs); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return gs.Validate()
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the interchain accounts module.
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *runtime.ServeMux) {
	err := controllertypes.RegisterQueryHandlerClient(context.Background(), mux, controllertypes.NewQueryClient(clientCtx))
	if err != nil {
		panic(err)
	}

	err = hosttypes.RegisterQueryHandlerClient(context.Background(), mux, hosttypes.NewQueryClient(clientCtx))
	if err != nil {
		panic(err)
	}
}

// GetTxCmd implements AppModule interface
func (AppModule) GetTxCmd() *cobra.Command {
	return cli.NewTxCmd()
}

// GetQueryCmd implements AppModule interface
func (AppModule) GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}

// AppModule is the application module for the IBC interchain accounts module
type AppModule struct {
	cdc              codec.Codec
	controllerKeeper *controllerkeeper.Keeper
	hostKeeper       *hostkeeper.Keeper
}

// NewAppModule creates a new IBC interchain accounts module
func NewAppModule(cdc codec.Codec, controllerKeeper *controllerkeeper.Keeper, hostKeeper *hostkeeper.Keeper) AppModule {
	return AppModule{
		cdc:              cdc,
		controllerKeeper: controllerKeeper,
		hostKeeper:       hostKeeper,
	}
}

// InitModule will initialize the interchain accounts module. It should only be
// called once and as an alternative to InitGenesis.
func (am AppModule) InitModule(ctx context.Context, controllerParams controllertypes.Params, hostParams hosttypes.Params) {
	if am.controllerKeeper != nil {
		controllerkeeper.InitGenesis(ctx, *am.controllerKeeper, genesistypes.ControllerGenesisState{
			Params: controllerParams,
		})
	}

	if am.hostKeeper != nil {
		if err := hostParams.Validate(); err != nil {
			panic(fmt.Errorf("could not set ica host params at initialization: %v", err))
		}

		hostkeeper.InitGenesis(ctx, *am.hostKeeper, genesistypes.HostGenesisState{
			Params: hostParams,
			Port:   types.HostPortID,
		})
	}
}

// RegisterServices registers module services
func (am AppModule) RegisterServices(cfg module.Configurator) {
	if am.controllerKeeper != nil {
		controllertypes.RegisterMsgServer(cfg.MsgServer(), controllerkeeper.NewMsgServerImpl(am.controllerKeeper))
		controllertypes.RegisterQueryServer(cfg.QueryServer(), am.controllerKeeper)
	}

	if am.hostKeeper != nil {
		hosttypes.RegisterMsgServer(cfg.MsgServer(), hostkeeper.NewMsgServerImpl(am.hostKeeper))
		hosttypes.RegisterQueryServer(cfg.QueryServer(), am.hostKeeper)
	}

	controllerMigrator := controllerkeeper.NewMigrator(am.controllerKeeper)
	if err := cfg.RegisterMigration(types.ModuleName, 1, controllerMigrator.AssertChannelCapabilityMigrations); err != nil {
		panic(fmt.Errorf("failed to migrate interchainaccounts app from version 1 to 2 (channel capabilities owned by controller submodule check): %v", err))
	}

	hostMigrator := hostkeeper.NewMigrator(am.hostKeeper)
	if err := cfg.RegisterMigration(types.ModuleName, 2, func(ctx sdk.Context) error {
		if err := hostMigrator.MigrateParams(ctx); err != nil {
			return err
		}
		return controllerMigrator.MigrateParams(ctx)
	}); err != nil {
		panic(fmt.Errorf("failed to migrate interchainaccounts app from version 2 to 3 (self-managed params migration): %v", err))
	}
}

// InitGenesis performs genesis initialization for the interchain accounts module.
// It returns no validator updates.
func (am AppModule) InitGenesis(ctx context.Context, data json.RawMessage) error {
	var genesisState genesistypes.GenesisState
	if err := am.cdc.UnmarshalJSON(data, &genesisState); err != nil {
		return err
	}

	if am.controllerKeeper != nil {
		controllerkeeper.InitGenesis(ctx, *am.controllerKeeper, genesisState.ControllerGenesisState)
	}

	if am.hostKeeper != nil {
		hostkeeper.InitGenesis(ctx, *am.hostKeeper, genesisState.HostGenesisState)
	}
	return nil
}

// ExportGenesis returns the exported genesis state as raw bytes for the interchain accounts module
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	var (
		controllerGenesisState = genesistypes.DefaultControllerGenesis()
		hostGenesisState       = genesistypes.DefaultHostGenesis()
	)

	if am.controllerKeeper != nil {
		controllerGenesisState = controllerkeeper.ExportGenesis(ctx, *am.controllerKeeper)
	}

	if am.hostKeeper != nil {
		hostGenesisState = hostkeeper.ExportGenesis(ctx, *am.hostKeeper)
	}

	gs := genesistypes.NewGenesisState(controllerGenesisState, hostGenesisState)

	return am.cdc.MarshalJSON(gs)
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return 3 }

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the ics27 module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// ProposalMsgs returns msgs used for governance proposals for simulations.
func (am AppModule) ProposalMsgs(simState module.SimulationState) []simtypes.WeightedProposalMsg {
	return simulation.ProposalMsgs(am.controllerKeeper, am.hostKeeper)
}

// WeightedOperations is unimplemented.
func (AppModule) WeightedOperations(simState module.SimulationState) []simtypes.WeightedOperation {
	return nil
}

// RegisterStoreDecoder registers a decoder for interchain accounts module's types
func (AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[controllertypes.StoreKey] = simulation.NewDecodeStore()
	sdr[hosttypes.StoreKey] = simulation.NewDecodeStore()
}
