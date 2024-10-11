package module

import (
	"context"
	"encoding/json"
	"fmt"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"
	"cosmossdk.io/errors"
	"cosmossdk.io/x/authz"
	"cosmossdk.io/x/authz/client/cli"
	"cosmossdk.io/x/authz/keeper"
	"cosmossdk.io/x/authz/simulation"

	sdkclient "github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/simsx"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

const ConsensusVersion = 2

var (
	_ module.HasAminoCodec       = AppModule{}
	_ module.HasGRPCGateway      = AppModule{}
	_ module.AppModuleSimulation = AppModule{}

	_ appmodule.HasRegisterInterfaces = AppModule{}
	_ appmodule.HasGenesis            = AppModule{}
	_ appmodule.AppModule             = AppModule{}
	_ appmodule.HasBeginBlocker       = AppModule{}
	_ appmodule.HasMigrations         = AppModule{}
)

// AppModule implements the sdk.AppModule interface
type AppModule struct {
	cdc      codec.Codec
	keeper   keeper.Keeper
	registry cdctypes.InterfaceRegistry
}

// NewAppModule creates a new AppModule object
func NewAppModule(
	cdc codec.Codec,
	keeper keeper.Keeper,
	registry cdctypes.InterfaceRegistry,
) AppModule {
	return AppModule{
		cdc:      cdc,
		keeper:   keeper,
		registry: registry,
	}
}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// Name returns the authz module's name.
// Deprecated: kept for legacy reasons.
func (AppModule) Name() string {
	return authz.ModuleName
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	authz.RegisterQueryServer(registrar, am.keeper)
	authz.RegisterMsgServer(registrar, am.keeper)

	return nil
}

// RegisterMigrations registers the authz module migrations.
func (am AppModule) RegisterMigrations(mr appmodule.MigrationRegistrar) error {
	m := keeper.NewMigrator(am.keeper)
	if err := mr.Register(authz.ModuleName, 1, m.Migrate1to2); err != nil {
		return fmt.Errorf("failed to migrate x/%s from version 1 to 2: %w", authz.ModuleName, err)
	}

	return nil
}

// RegisterLegacyAminoCodec registers the authz module's types for the given codec.
func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	authz.RegisterLegacyAminoCodec(registrar)
}

// RegisterInterfaces registers the authz module's interface types
func (AppModule) RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	authz.RegisterInterfaces(registrar)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the authz module.
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx sdkclient.Context, mux *gwruntime.ServeMux) {
	if err := authz.RegisterQueryHandlerClient(context.Background(), mux, authz.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// GetTxCmd returns the transaction commands for the authz module
func (AppModule) GetTxCmd() *cobra.Command {
	return cli.GetTxCmd()
}

// DefaultGenesis returns default genesis state as raw bytes for the authz module.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(authz.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the authz module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var data authz.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &data); err != nil {
		return errors.Wrapf(err, "failed to unmarshal %s genesis state", authz.ModuleName)
	}

	return authz.ValidateGenesis(data)
}

// InitGenesis performs genesis initialization for the authz module.
func (am AppModule) InitGenesis(ctx context.Context, data json.RawMessage) error {
	var genesisState authz.GenesisState
	if err := am.cdc.UnmarshalJSON(data, &genesisState); err != nil {
		return err
	}
	return am.keeper.InitGenesis(ctx, &genesisState)
}

// ExportGenesis returns the exported genesis state as raw bytes for the authz module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs, err := am.keeper.ExportGenesis(ctx)
	if err != nil {
		return nil, err
	}
	return am.cdc.MarshalJSON(gs)
}

// ConsensusVersion implements HasConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }

// BeginBlock returns the begin blocker for the authz module.
func (am AppModule) BeginBlock(ctx context.Context) error {
	return BeginBlocker(ctx, am.keeper)
}

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the authz module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// RegisterStoreDecoder registers a decoder for authz module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[keeper.StoreKey] = simulation.NewDecodeStore(am.cdc)
}

func (am AppModule) WeightedOperationsX(weights simsx.WeightSource, reg simsx.Registry) {
	reg.Add(weights.Get("msg_grant", 100), simulation.MsgGrantFactory())
	reg.Add(weights.Get("msg_revoke", 90), simulation.MsgRevokeFactory(am.keeper))
	reg.Add(weights.Get("msg_exec", 90), simulation.MsgExecFactory(am.keeper))
}
