package staking

import (
	"context"
	"encoding/json"
	"fmt"
	"sort"

	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	"github.com/cosmos/cosmos-sdk/x/staking/exported"

	abci "github.com/cometbft/cometbft/abci/types"
	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"
	"golang.org/x/exp/maps"

	modulev1 "cosmossdk.io/api/cosmos/staking/module/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	store "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/staking/client/cli"
	"github.com/cosmos/cosmos-sdk/x/staking/keeper"
	"github.com/cosmos/cosmos-sdk/x/staking/simulation"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

const (
	consensusVersion uint64 = 4
)

var (
	_ module.BeginBlockAppModule = AppModule{}
	_ module.EndBlockAppModule   = AppModule{}
	_ module.AppModuleBasic      = AppModuleBasic{}
	_ module.AppModuleSimulation = AppModule{}
)

// AppModuleBasic defines the basic application module used by the staking module.
type AppModuleBasic struct {
	cdc codec.Codec
}

var _ module.AppModuleBasic = AppModuleBasic{}

// Name returns the staking module's name.
func (AppModuleBasic) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec registers the staking module's types on the given LegacyAmino codec.
func (AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	types.RegisterLegacyAminoCodec(cdc)
}

// RegisterInterfaces registers the module's interface types
func (b AppModuleBasic) RegisterInterfaces(registry cdctypes.InterfaceRegistry) {
	types.RegisterInterfaces(registry)
}

// DefaultGenesis returns default genesis state as raw bytes for the staking
// module.
func (AppModuleBasic) DefaultGenesis(cdc codec.JSONCodec) json.RawMessage {
	return cdc.MustMarshalJSON(types.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the staking module.
func (AppModuleBasic) ValidateGenesis(cdc codec.JSONCodec, config client.TxEncodingConfig, bz json.RawMessage) error {
	var data types.GenesisState
	if err := cdc.UnmarshalJSON(bz, &data); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return ValidateGenesis(&data)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the staking module.
func (AppModuleBasic) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *gwruntime.ServeMux) {
	if err := types.RegisterQueryHandlerClient(context.Background(), mux, types.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// GetTxCmd returns the root tx command for the staking module.
func (AppModuleBasic) GetTxCmd() *cobra.Command {
	return cli.NewTxCmd()
}

// GetQueryCmd returns no root query command for the staking module.
func (AppModuleBasic) GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}

// AppModule implements an application module for the staking module.
type AppModule struct {
	AppModuleBasic

	keeper        *keeper.Keeper
	accountKeeper types.AccountKeeper
	bankKeeper    types.BankKeeper

	// legacySubspace is used solely for migration of x/params managed parameters
	legacySubspace exported.Subspace
}

// NewAppModule creates a new AppModule object
func NewAppModule(
	cdc codec.Codec,
	keeper *keeper.Keeper,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	ls exported.Subspace,
) AppModule {
	return AppModule{
		AppModuleBasic: AppModuleBasic{cdc: cdc},
		keeper:         keeper,
		accountKeeper:  ak,
		bankKeeper:     bk,
		legacySubspace: ls,
	}
}

var _ appmodule.AppModule = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// Name returns the staking module's name.
func (AppModule) Name() string {
	return types.ModuleName
}

// RegisterInvariants registers the staking module invariants.
func (am AppModule) RegisterInvariants(ir sdk.InvariantRegistry) {
	keeper.RegisterInvariants(ir, am.keeper)
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(cfg module.Configurator) {
	types.RegisterMsgServer(cfg.MsgServer(), keeper.NewMsgServerImpl(am.keeper))
	querier := keeper.Querier{Keeper: am.keeper}
	types.RegisterQueryServer(cfg.QueryServer(), querier)

	m := keeper.NewMigrator(am.keeper, am.legacySubspace)
	if err := cfg.RegisterMigration(types.ModuleName, 1, m.Migrate1to2); err != nil {
		panic(fmt.Sprintf("failed to migrate x/%s from version 1 to 2: %v", types.ModuleName, err))
	}
	if err := cfg.RegisterMigration(types.ModuleName, 2, m.Migrate2to3); err != nil {
		panic(fmt.Sprintf("failed to migrate x/%s from version 2 to 3: %v", types.ModuleName, err))
	}
	if err := cfg.RegisterMigration(types.ModuleName, 3, m.Migrate3to4); err != nil {
		panic(fmt.Sprintf("failed to migrate x/%s from version 3 to 4: %v", types.ModuleName, err))
	}
}

// InitGenesis performs genesis initialization for the staking module.
func (am AppModule) InitGenesis(ctx sdk.Context, cdc codec.JSONCodec, data json.RawMessage) []abci.ValidatorUpdate {
	var genesisState types.GenesisState

	cdc.MustUnmarshalJSON(data, &genesisState)

	return am.keeper.InitGenesis(ctx, &genesisState)
}

// ExportGenesis returns the exported genesis state as raw bytes for the staking
// module.
func (am AppModule) ExportGenesis(ctx sdk.Context, cdc codec.JSONCodec) json.RawMessage {
	return cdc.MustMarshalJSON(am.keeper.ExportGenesis(ctx))
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return consensusVersion }

// BeginBlock returns the begin blocker for the staking module.
func (am AppModule) BeginBlock(ctx sdk.Context, _ abci.RequestBeginBlock) {
	BeginBlocker(ctx, am.keeper)
}

// EndBlock returns the end blocker for the staking module. It returns no validator
// updates.
func (am AppModule) EndBlock(ctx sdk.Context, _ abci.RequestEndBlock) []abci.ValidatorUpdate {
	return EndBlocker(ctx, am.keeper)
}

func init() {
	appmodule.Register(
		&modulev1.Module{},
		appmodule.Provide(ProvideModule),
		appmodule.Invoke(InvokeSetStakingHooks),
	)
}

type StakingInputs struct {
	depinject.In

	Config        *modulev1.Module
	AccountKeeper types.AccountKeeper
	BankKeeper    types.BankKeeper
	Cdc           codec.Codec
	Key           *store.KVStoreKey

	// LegacySubspace is used solely for migration of x/params managed parameters
	LegacySubspace exported.Subspace `optional:"true"`
}

// Dependency Injection Outputs
type StakingOutputs struct {
	depinject.Out

	StakingKeeper *keeper.Keeper
	Module        appmodule.AppModule
}

func ProvideModule(in StakingInputs) StakingOutputs {
	// default to governance authority if not provided
	authority := authtypes.NewModuleAddress(govtypes.ModuleName)
	if in.Config.Authority != "" {
		authority = authtypes.NewModuleAddressOrBech32Address(in.Config.Authority)
	}

	k := keeper.NewKeeper(
		in.Cdc,
		in.Key,
		in.AccountKeeper,
		in.BankKeeper,
		authority.String(),
	)
	m := NewAppModule(in.Cdc, k, in.AccountKeeper, in.BankKeeper, in.LegacySubspace)
	return StakingOutputs{StakingKeeper: k, Module: m}
}

func InvokeSetStakingHooks(
	config *modulev1.Module,
	keeper *keeper.Keeper,
	stakingHooks map[string]types.StakingHooksWrapper,
) error {
	// all arguments to invokers are optional
	if keeper == nil || config == nil {
		return nil
	}

	modNames := maps.Keys(stakingHooks)
	order := config.HooksOrder
	if len(order) == 0 {
		order = modNames
		sort.Strings(order)
	}

	if len(order) != len(modNames) {
		return fmt.Errorf("len(hooks_order: %v) != len(hooks modules: %v)", order, modNames)
	}

	if len(modNames) == 0 {
		return nil
	}

	var multiHooks types.MultiStakingHooks
	for _, modName := range order {
		hook, ok := stakingHooks[modName]
		if !ok {
			return fmt.Errorf("can't find staking hooks for module %s", modName)
		}

		multiHooks = append(multiHooks, hook)
	}

	keeper.SetHooks(multiHooks)
	return nil
}

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the staking module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// ProposalMsgs returns msgs used for governance proposals for simulations.
func (AppModule) ProposalMsgs(simState module.SimulationState) []simtypes.WeightedProposalMsg {
	return simulation.ProposalMsgs()
}

// RegisterStoreDecoder registers a decoder for staking module's types
func (am AppModule) RegisterStoreDecoder(sdr sdk.StoreDecoderRegistry) {
	sdr[types.StoreKey] = simulation.NewDecodeStore(am.cdc)
}

// WeightedOperations returns the all the staking module operations with their respective weights.
func (am AppModule) WeightedOperations(simState module.SimulationState) []simtypes.WeightedOperation {
	return simulation.WeightedOperations(
		simState.AppParams, simState.Cdc, am.accountKeeper, am.bankKeeper, am.keeper,
	)
}
