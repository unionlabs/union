package distribution

import (
	"context"
	"encoding/json"
	"fmt"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"
	"cosmossdk.io/x/distribution/client/cli"
	"cosmossdk.io/x/distribution/keeper"
	"cosmossdk.io/x/distribution/simulation"
	"cosmossdk.io/x/distribution/types"

	sdkclient "github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/simsx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// ConsensusVersion defines the current x/distribution module consensus version.
const ConsensusVersion = 4

var (
	_ module.HasAminoCodec       = AppModule{}
	_ module.HasGRPCGateway      = AppModule{}
	_ module.AppModuleSimulation = AppModule{}
	_ module.HasInvariants       = AppModule{}

	_ appmodule.AppModule             = AppModule{}
	_ appmodule.HasBeginBlocker       = AppModule{}
	_ appmodule.HasMigrations         = AppModule{}
	_ appmodule.HasRegisterInterfaces = AppModule{}
	_ appmodule.HasGenesis            = AppModule{}
)

// AppModule implements an application module for the distribution module.
type AppModule struct {
	cdc           codec.Codec
	keeper        keeper.Keeper
	stakingKeeper types.StakingKeeper
}

// NewAppModule creates a new AppModule object
func NewAppModule(cdc codec.Codec, keeper keeper.Keeper, stakingKeeper types.StakingKeeper) AppModule {
	return AppModule{
		cdc:           cdc,
		keeper:        keeper,
		stakingKeeper: stakingKeeper,
	}
}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// Name returns the distribution module's name.
// Deprecated: kept for legacy reasons.
func (AppModule) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec registers the distribution module's types for the given codec.
func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	types.RegisterLegacyAminoCodec(registrar)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the distribution module.
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx sdkclient.Context, mux *gwruntime.ServeMux) {
	if err := types.RegisterQueryHandlerClient(context.Background(), mux, types.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// GetTxCmd returns the root tx command for the distribution module.
func (AppModule) GetTxCmd() *cobra.Command {
	return cli.NewTxCmd()
}

// RegisterInterfaces implements InterfaceModule
func (AppModule) RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	types.RegisterInterfaces(registrar)
}

// RegisterInvariants registers the distribution module invariants.
func (am AppModule) RegisterInvariants(ir sdk.InvariantRegistry) {
	keeper.RegisterInvariants(ir, am.keeper)
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	types.RegisterMsgServer(registrar, keeper.NewMsgServerImpl(am.keeper))
	types.RegisterQueryServer(registrar, keeper.NewQuerier(am.keeper))

	return nil
}

// RegisterMigrations registers the distribution module's migrations.
func (am AppModule) RegisterMigrations(mr appmodule.MigrationRegistrar) error {
	m := keeper.NewMigrator(am.keeper)
	if err := mr.Register(types.ModuleName, 1, m.Migrate1to2); err != nil {
		return fmt.Errorf("failed to migrate x/%s from version 1 to 2: %w", types.ModuleName, err)
	}

	if err := mr.Register(types.ModuleName, 2, m.Migrate2to3); err != nil {
		return fmt.Errorf("failed to migrate x/%s from version 2 to 3: %w", types.ModuleName, err)
	}

	if err := mr.Register(types.ModuleName, 3, m.Migrate3to4); err != nil {
		return fmt.Errorf("failed to migrate x/%s from version 3 to 4: %w", types.ModuleName, err)
	}

	return nil
}

// DefaultGenesis returns default genesis state as raw bytes for the distribution module.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(types.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the distribution module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var data types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &data); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return types.ValidateGenesis(&data)
}

// InitGenesis performs genesis initialization for the distribution module.
func (am AppModule) InitGenesis(ctx context.Context, data json.RawMessage) error {
	var genesisState types.GenesisState
	if err := am.cdc.UnmarshalJSON(data, &genesisState); err != nil {
		return err
	}
	return am.keeper.InitGenesis(ctx, genesisState)
}

// ExportGenesis returns the exported genesis state as raw bytes for the distribution
// module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs, err := am.keeper.ExportGenesis(ctx)
	if err != nil {
		return nil, err
	}
	return am.cdc.MarshalJSON(gs)
}

// ConsensusVersion implements HasConsensusVersion
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }

// BeginBlock returns the begin blocker for the distribution module.
func (am AppModule) BeginBlock(ctx context.Context) error {
	return am.keeper.BeginBlocker(ctx)
}

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the distribution module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// RegisterStoreDecoder registers a decoder for distribution module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[types.StoreKey] = simulation.NewDecodeStore(am.cdc)
}

// ProposalMsgsX returns msgs used for governance proposals for simulations.
func (AppModule) ProposalMsgsX(weights simsx.WeightSource, reg simsx.Registry) {
	reg.Add(weights.Get("msg_update_params", 100), simulation.MsgUpdateParamsFactory())
}

func (am AppModule) WeightedOperationsX(weights simsx.WeightSource, reg simsx.Registry) {
	reg.Add(weights.Get("msg_set_withdraw_address", 50), simulation.MsgSetWithdrawAddressFactory(am.keeper))
	reg.Add(weights.Get("msg_withdraw_delegation_reward", 50), simulation.MsgWithdrawDelegatorRewardFactory(am.keeper, am.stakingKeeper))
	reg.Add(weights.Get("msg_withdraw_validator_commission", 50), simulation.MsgWithdrawValidatorCommissionFactory(am.keeper, am.stakingKeeper))
}
