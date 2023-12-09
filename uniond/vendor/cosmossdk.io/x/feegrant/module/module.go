package module

import (
	"context"
	"encoding/json"
	"fmt"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	modulev1 "cosmossdk.io/api/cosmos/feegrant/module/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/store"
	"cosmossdk.io/depinject"
	"cosmossdk.io/errors"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/client/cli"
	"cosmossdk.io/x/feegrant/keeper"
	"cosmossdk.io/x/feegrant/simulation"

	sdkclient "github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

var (
	_ module.AppModuleBasic      = AppModule{}
	_ module.AppModuleSimulation = AppModule{}
	_ module.HasServices         = AppModule{}
	_ module.HasGenesis          = AppModule{}

	_ appmodule.AppModule     = AppModule{}
	_ appmodule.HasEndBlocker = AppModule{}
)

// ----------------------------------------------------------------------------
// AppModuleBasic
// ----------------------------------------------------------------------------

// AppModuleBasic defines the basic application module used by the feegrant module.
type AppModuleBasic struct {
	cdc codec.Codec
	ac  address.Codec
}

// Name returns the feegrant module's name.
func (ab AppModuleBasic) Name() string {
	return feegrant.ModuleName
}

// RegisterServices registers a gRPC query service to respond to the
// module-specific gRPC queries.
func (am AppModule) RegisterServices(cfg module.Configurator) {
	feegrant.RegisterMsgServer(cfg.MsgServer(), keeper.NewMsgServerImpl(am.keeper))
	feegrant.RegisterQueryServer(cfg.QueryServer(), am.keeper)
	m := keeper.NewMigrator(am.keeper)
	err := cfg.RegisterMigration(feegrant.ModuleName, 1, m.Migrate1to2)
	if err != nil {
		panic(fmt.Sprintf("failed to migrate x/feegrant from version 1 to 2: %v", err))
	}
}

// RegisterLegacyAminoCodec registers the feegrant module's types for the given codec.
func (ab AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	feegrant.RegisterLegacyAminoCodec(cdc)
}

// RegisterInterfaces registers the feegrant module's interface types
func (ab AppModuleBasic) RegisterInterfaces(registry cdctypes.InterfaceRegistry) {
	feegrant.RegisterInterfaces(registry)
}

// DefaultGenesis returns default genesis state as raw bytes for the feegrant
// module.
func (ab AppModuleBasic) DefaultGenesis(cdc codec.JSONCodec) json.RawMessage {
	return cdc.MustMarshalJSON(feegrant.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the feegrant module.
func (ab AppModuleBasic) ValidateGenesis(cdc codec.JSONCodec, config sdkclient.TxEncodingConfig, bz json.RawMessage) error {
	var data feegrant.GenesisState
	if err := cdc.UnmarshalJSON(bz, &data); err != nil {
		return errors.Wrapf(err, "failed to unmarshal %s genesis state", feegrant.ModuleName)
	}

	return feegrant.ValidateGenesis(data)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the feegrant module.
func (ab AppModuleBasic) RegisterGRPCGatewayRoutes(clientCtx sdkclient.Context, mux *gwruntime.ServeMux) {
	if err := feegrant.RegisterQueryHandlerClient(context.Background(), mux, feegrant.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// GetTxCmd returns the root tx command for the feegrant module.
func (ab AppModuleBasic) GetTxCmd() *cobra.Command {
	return cli.GetTxCmd(ab.ac)
}

// ----------------------------------------------------------------------------
// AppModule
// ----------------------------------------------------------------------------

// AppModule implements an application module for the feegrant module.
type AppModule struct {
	AppModuleBasic

	keeper        keeper.Keeper
	accountKeeper feegrant.AccountKeeper
	bankKeeper    feegrant.BankKeeper
	registry      cdctypes.InterfaceRegistry
}

// NewAppModule creates a new AppModule object
func NewAppModule(cdc codec.Codec, ak feegrant.AccountKeeper, bk feegrant.BankKeeper, keeper keeper.Keeper, registry cdctypes.InterfaceRegistry) AppModule {
	return AppModule{
		AppModuleBasic: AppModuleBasic{cdc: cdc, ac: ak.AddressCodec()},
		keeper:         keeper,
		accountKeeper:  ak,
		bankKeeper:     bk,
		registry:       registry,
	}
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// InitGenesis performs genesis initialization for the feegrant module. It returns
// no validator updates.
func (am AppModule) InitGenesis(ctx sdk.Context, cdc codec.JSONCodec, bz json.RawMessage) {
	var gs feegrant.GenesisState
	cdc.MustUnmarshalJSON(bz, &gs)

	err := am.keeper.InitGenesis(ctx, &gs)
	if err != nil {
		panic(err)
	}
}

// ExportGenesis returns the exported genesis state as raw bytes for the feegrant
// module.
func (am AppModule) ExportGenesis(ctx sdk.Context, cdc codec.JSONCodec) json.RawMessage {
	gs, err := am.keeper.ExportGenesis(ctx)
	if err != nil {
		panic(err)
	}

	return cdc.MustMarshalJSON(gs)
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return 2 }

// EndBlock returns the end blocker for the feegrant module. It returns no validator
// updates.
func (am AppModule) EndBlock(ctx context.Context) error {
	return EndBlocker(ctx, am.keeper)
}

func init() {
	appmodule.Register(&modulev1.Module{},
		appmodule.Provide(ProvideModule),
	)
}

type FeegrantInputs struct {
	depinject.In

	StoreService  store.KVStoreService
	Cdc           codec.Codec
	AccountKeeper feegrant.AccountKeeper
	BankKeeper    feegrant.BankKeeper
	Registry      cdctypes.InterfaceRegistry
}

func ProvideModule(in FeegrantInputs) (keeper.Keeper, appmodule.AppModule) {
	k := keeper.NewKeeper(in.Cdc, in.StoreService, in.AccountKeeper)
	m := NewAppModule(in.Cdc, in.AccountKeeper, in.BankKeeper, k, in.Registry)
	return k, m
}

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the feegrant module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// RegisterStoreDecoder registers a decoder for feegrant module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[feegrant.StoreKey] = simulation.NewDecodeStore(am.cdc)
}

// WeightedOperations returns all the feegrant module operations with their respective weights.
func (am AppModule) WeightedOperations(simState module.SimulationState) []simtypes.WeightedOperation {
	return simulation.WeightedOperations(
		am.registry, simState.AppParams, simState.Cdc, simState.TxConfig,
		am.accountKeeper, am.bankKeeper, am.keeper, am.ac,
	)
}
