package module

import (
	"context"
	"encoding/json"
	"fmt"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	modulev1 "cosmossdk.io/api/cosmos/group/module/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"
	store "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	sdkclient "github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/group"
	"github.com/cosmos/cosmos-sdk/x/group/client/cli"
	"github.com/cosmos/cosmos-sdk/x/group/keeper"
	"github.com/cosmos/cosmos-sdk/x/group/simulation"
)

// ConsensusVersion defines the current x/group module consensus version.
const ConsensusVersion = 2

var (
	_ module.AppModuleBasic      = AppModule{}
	_ module.AppModuleSimulation = AppModule{}
	_ module.HasGenesis          = AppModule{}
	_ module.HasServices         = AppModule{}
	_ module.HasInvariants       = AppModule{}

	_ appmodule.AppModule     = AppModule{}
	_ appmodule.HasEndBlocker = AppModule{}
)

type AppModule struct {
	AppModuleBasic
	keeper     keeper.Keeper
	bankKeeper group.BankKeeper
	accKeeper  group.AccountKeeper
	registry   cdctypes.InterfaceRegistry
}

// NewAppModule creates a new AppModule object
func NewAppModule(cdc codec.Codec, keeper keeper.Keeper, ak group.AccountKeeper, bk group.BankKeeper, registry cdctypes.InterfaceRegistry) AppModule {
	return AppModule{
		AppModuleBasic: AppModuleBasic{cdc: cdc, ac: ak.AddressCodec()},
		keeper:         keeper,
		bankKeeper:     bk,
		accKeeper:      ak,
		registry:       registry,
	}
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

type AppModuleBasic struct {
	cdc codec.Codec
	ac  address.Codec
}

// Name returns the group module's name.
func (AppModuleBasic) Name() string {
	return group.ModuleName
}

// DefaultGenesis returns default genesis state as raw bytes for the group
// module.
func (AppModuleBasic) DefaultGenesis(cdc codec.JSONCodec) json.RawMessage {
	return cdc.MustMarshalJSON(group.NewGenesisState())
}

// ValidateGenesis performs genesis state validation for the group module.
func (AppModuleBasic) ValidateGenesis(cdc codec.JSONCodec, config sdkclient.TxEncodingConfig, bz json.RawMessage) error {
	var data group.GenesisState
	if err := cdc.UnmarshalJSON(bz, &data); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", group.ModuleName, err)
	}
	return data.Validate()
}

// GetTxCmd returns the transaction commands for the group module
func (a AppModuleBasic) GetTxCmd() *cobra.Command {
	return cli.TxCmd(a.Name(), a.ac)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the group module.
func (a AppModuleBasic) RegisterGRPCGatewayRoutes(clientCtx sdkclient.Context, mux *gwruntime.ServeMux) {
	if err := group.RegisterQueryHandlerClient(context.Background(), mux, group.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// RegisterInterfaces registers the group module's interface types
func (AppModuleBasic) RegisterInterfaces(registry cdctypes.InterfaceRegistry) {
	group.RegisterInterfaces(registry)
}

// RegisterLegacyAminoCodec registers the group module's types for the given codec.
func (AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	group.RegisterLegacyAminoCodec(cdc)
}

// RegisterInvariants does nothing, there are no invariants to enforce
func (am AppModule) RegisterInvariants(ir sdk.InvariantRegistry) {
	keeper.RegisterInvariants(ir, am.keeper)
}

// InitGenesis performs genesis initialization for the group module. It returns
// no validator updates.
func (am AppModule) InitGenesis(ctx sdk.Context, cdc codec.JSONCodec, data json.RawMessage) {
	am.keeper.InitGenesis(ctx, cdc, data)
}

// ExportGenesis returns the exported genesis state as raw bytes for the group
// module.
func (am AppModule) ExportGenesis(ctx sdk.Context, cdc codec.JSONCodec) json.RawMessage {
	gs := am.keeper.ExportGenesis(ctx, cdc)
	return cdc.MustMarshalJSON(gs)
}

// RegisterServices registers a gRPC query service to respond to the
// module-specific gRPC queries.
func (am AppModule) RegisterServices(cfg module.Configurator) {
	group.RegisterMsgServer(cfg.MsgServer(), am.keeper)
	group.RegisterQueryServer(cfg.QueryServer(), am.keeper)

	m := keeper.NewMigrator(am.keeper)
	if err := cfg.RegisterMigration(group.ModuleName, 1, m.Migrate1to2); err != nil {
		panic(fmt.Sprintf("failed to migrate x/%s from version 1 to 2: %v", group.ModuleName, err))
	}
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }

// EndBlock implements the group module's EndBlock.
func (am AppModule) EndBlock(ctx context.Context) error {
	c := sdk.UnwrapSDKContext(ctx)
	return EndBlocker(c, am.keeper)
}

// ____________________________________________________________________________

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the group module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// RegisterStoreDecoder registers a decoder for group module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[group.StoreKey] = simulation.NewDecodeStore(am.cdc)
}

// WeightedOperations returns the all the gov module operations with their respective weights.
func (am AppModule) WeightedOperations(simState module.SimulationState) []simtypes.WeightedOperation {
	return simulation.WeightedOperations(
		am.registry,
		simState.AppParams, simState.Cdc, simState.TxConfig,
		am.accKeeper, am.bankKeeper, am.keeper, am.cdc,
	)
}

//
// App Wiring Setup
//

func init() {
	appmodule.Register(
		&modulev1.Module{},
		appmodule.Provide(ProvideModule),
	)
}

type GroupInputs struct {
	depinject.In

	Config           *modulev1.Module
	Key              *store.KVStoreKey
	Cdc              codec.Codec
	AccountKeeper    group.AccountKeeper
	BankKeeper       group.BankKeeper
	Registry         cdctypes.InterfaceRegistry
	MsgServiceRouter baseapp.MessageRouter
}

type GroupOutputs struct {
	depinject.Out

	GroupKeeper keeper.Keeper
	Module      appmodule.AppModule
}

func ProvideModule(in GroupInputs) GroupOutputs {
	/*
		Example of setting group params:
		in.Config.MaxMetadataLen = 1000
		in.Config.MaxExecutionPeriod = "1209600s"
	*/

	k := keeper.NewKeeper(in.Key, in.Cdc, in.MsgServiceRouter, in.AccountKeeper, group.Config{MaxExecutionPeriod: in.Config.MaxExecutionPeriod.AsDuration(), MaxMetadataLen: in.Config.MaxMetadataLen})
	m := NewAppModule(in.Cdc, k, in.AccountKeeper, in.BankKeeper, in.Registry)
	return GroupOutputs{GroupKeeper: k, Module: m}
}
