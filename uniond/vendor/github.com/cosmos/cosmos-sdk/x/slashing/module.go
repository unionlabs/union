package slashing

import (
	"context"
	"encoding/json"
	"fmt"

	abci "github.com/cometbft/cometbft/abci/types"
	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	modulev1 "cosmossdk.io/api/cosmos/slashing/module/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	store "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	"github.com/cosmos/cosmos-sdk/x/slashing/client/cli"
	"github.com/cosmos/cosmos-sdk/x/slashing/exported"
	"github.com/cosmos/cosmos-sdk/x/slashing/keeper"
	"github.com/cosmos/cosmos-sdk/x/slashing/simulation"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
	staking "github.com/cosmos/cosmos-sdk/x/staking/types"
)

// ConsensusVersion defines the current x/slashing module consensus version.
const ConsensusVersion = 3

var (
	_ module.BeginBlockAppModule = AppModule{}
	_ module.AppModuleBasic      = AppModuleBasic{}
	_ module.AppModuleSimulation = AppModule{}
)

// AppModuleBasic defines the basic application module used by the slashing module.
type AppModuleBasic struct {
	cdc codec.Codec
}

var _ module.AppModuleBasic = AppModuleBasic{}

// Name returns the slashing module's name.
func (AppModuleBasic) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec registers the slashing module's types for the given codec.
func (AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	types.RegisterLegacyAminoCodec(cdc)
}

// RegisterInterfaces registers the module's interface types
func (b AppModuleBasic) RegisterInterfaces(registry cdctypes.InterfaceRegistry) {
	types.RegisterInterfaces(registry)
}

// DefaultGenesis returns default genesis state as raw bytes for the slashing
// module.
func (AppModuleBasic) DefaultGenesis(cdc codec.JSONCodec) json.RawMessage {
	return cdc.MustMarshalJSON(types.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the slashing module.
func (AppModuleBasic) ValidateGenesis(cdc codec.JSONCodec, config client.TxEncodingConfig, bz json.RawMessage) error {
	var data types.GenesisState
	if err := cdc.UnmarshalJSON(bz, &data); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return types.ValidateGenesis(data)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the slashig module.
func (AppModuleBasic) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *gwruntime.ServeMux) {
	if err := types.RegisterQueryHandlerClient(context.Background(), mux, types.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// GetTxCmd returns the root tx command for the slashing module.
func (AppModuleBasic) GetTxCmd() *cobra.Command {
	return cli.NewTxCmd()
}

// GetQueryCmd returns no root query command for the slashing module.
func (AppModuleBasic) GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}

// AppModule implements an application module for the slashing module.
type AppModule struct {
	AppModuleBasic

	keeper        keeper.Keeper
	accountKeeper types.AccountKeeper
	bankKeeper    types.BankKeeper
	stakingKeeper types.StakingKeeper

	// legacySubspace is used solely for migration of x/slashing managed parameters
	legacySubspace exported.Subspace
}

// NewAppModule creates a new AppModule object
func NewAppModule(cdc codec.Codec, keeper keeper.Keeper, ak types.AccountKeeper, bk types.BankKeeper, sk types.StakingKeeper, ss exported.Subspace) AppModule {
	return AppModule{
		AppModuleBasic: AppModuleBasic{cdc: cdc},
		keeper:         keeper,
		accountKeeper:  ak,
		bankKeeper:     bk,
		stakingKeeper:  sk,
		legacySubspace: ss,
	}
}

var _ appmodule.AppModule = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// Name returns the slashing module's name.
func (AppModule) Name() string {
	return types.ModuleName
}

// RegisterInvariants registers the slashing module invariants.
func (am AppModule) RegisterInvariants(_ sdk.InvariantRegistry) {}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(cfg module.Configurator) {
	types.RegisterMsgServer(cfg.MsgServer(), keeper.NewMsgServerImpl(am.keeper))
	types.RegisterQueryServer(cfg.QueryServer(), am.keeper)

	m := keeper.NewMigrator(am.keeper, am.legacySubspace)
	if err := cfg.RegisterMigration(types.ModuleName, 1, m.Migrate1to2); err != nil {
		panic(fmt.Sprintf("failed to migrate x/%s from version 1 to 2: %v", types.ModuleName, err))
	}

	if err := cfg.RegisterMigration(types.ModuleName, 2, m.Migrate2to3); err != nil {
		panic(fmt.Sprintf("failed to migrate x/%s from version 2 to 3: %v", types.ModuleName, err))
	}
}

// InitGenesis performs genesis initialization for the slashing module. It returns
// no validator updates.
func (am AppModule) InitGenesis(ctx sdk.Context, cdc codec.JSONCodec, data json.RawMessage) []abci.ValidatorUpdate {
	var genesisState types.GenesisState
	cdc.MustUnmarshalJSON(data, &genesisState)
	am.keeper.InitGenesis(ctx, am.stakingKeeper, &genesisState)
	return []abci.ValidatorUpdate{}
}

// ExportGenesis returns the exported genesis state as raw bytes for the slashing
// module.
func (am AppModule) ExportGenesis(ctx sdk.Context, cdc codec.JSONCodec) json.RawMessage {
	gs := am.keeper.ExportGenesis(ctx)
	return cdc.MustMarshalJSON(gs)
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }

// BeginBlock returns the begin blocker for the slashing module.
func (am AppModule) BeginBlock(ctx sdk.Context, req abci.RequestBeginBlock) {
	BeginBlocker(ctx, req, am.keeper)
}

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the slashing module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// ProposalMsgs returns msgs used for governance proposals for simulations.
func (AppModule) ProposalMsgs(simState module.SimulationState) []simtypes.WeightedProposalMsg {
	return simulation.ProposalMsgs()
}

// RegisterStoreDecoder registers a decoder for slashing module's types
func (am AppModule) RegisterStoreDecoder(sdr sdk.StoreDecoderRegistry) {
	sdr[types.StoreKey] = simulation.NewDecodeStore(am.cdc)
}

// WeightedOperations returns the all the slashing module operations with their respective weights.
func (am AppModule) WeightedOperations(simState module.SimulationState) []simtypes.WeightedOperation {
	return simulation.WeightedOperations(
		simState.AppParams, simState.Cdc,
		am.accountKeeper, am.bankKeeper, am.keeper, am.stakingKeeper,
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

type SlashingInputs struct {
	depinject.In

	Config      *modulev1.Module
	Key         *store.KVStoreKey
	Cdc         codec.Codec
	LegacyAmino *codec.LegacyAmino

	AccountKeeper types.AccountKeeper
	BankKeeper    types.BankKeeper
	StakingKeeper types.StakingKeeper

	// LegacySubspace is used solely for migration of x/params managed parameters
	LegacySubspace exported.Subspace `optional:"true"`
}

type SlashingOutputs struct {
	depinject.Out

	Keeper keeper.Keeper
	Module appmodule.AppModule
	Hooks  staking.StakingHooksWrapper
}

func ProvideModule(in SlashingInputs) SlashingOutputs {
	// default to governance authority if not provided
	authority := authtypes.NewModuleAddress(govtypes.ModuleName)
	if in.Config.Authority != "" {
		authority = authtypes.NewModuleAddressOrBech32Address(in.Config.Authority)
	}

	k := keeper.NewKeeper(in.Cdc, in.LegacyAmino, in.Key, in.StakingKeeper, authority.String())
	m := NewAppModule(in.Cdc, k, in.AccountKeeper, in.BankKeeper, in.StakingKeeper, in.LegacySubspace)
	return SlashingOutputs{
		Keeper: k,
		Module: m,
		Hooks:  staking.StakingHooksWrapper{StakingHooks: k.Hooks()},
	}
}
