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
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/client/cli"
	"cosmossdk.io/x/feegrant/keeper"

	sdkclient "github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/types/module"
)

var (
	_ module.HasAminoCodec       = AppModule{}
	_ module.HasGRPCGateway      = AppModule{}
	_ module.AppModuleSimulation = AppModule{}

	_ appmodule.AppModule             = AppModule{}
	_ appmodule.HasEndBlocker         = AppModule{}
	_ appmodule.HasMigrations         = AppModule{}
	_ appmodule.HasGenesis            = AppModule{}
	_ appmodule.HasRegisterInterfaces = AppModule{}
)

// AppModule implements an application module for the feegrant module.
type AppModule struct {
	cdc      codec.Codec
	registry cdctypes.InterfaceRegistry

	keeper keeper.Keeper
}

// NewAppModule creates a new AppModule object
func NewAppModule(cdc codec.Codec, keeper keeper.Keeper, registry cdctypes.InterfaceRegistry) AppModule {
	return AppModule{
		cdc:      cdc,
		keeper:   keeper,
		registry: registry,
	}
}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// Name returns the feegrant module's name.
// Deprecated: kept for legacy reasons.
func (AppModule) Name() string {
	return feegrant.ModuleName
}

// RegisterLegacyAminoCodec registers the feegrant module's types for the given codec.
func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	feegrant.RegisterLegacyAminoCodec(registrar)
}

// RegisterInterfaces registers the feegrant module's interface types
func (AppModule) RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	feegrant.RegisterInterfaces(registrar)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the feegrant module.
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx sdkclient.Context, mux *gwruntime.ServeMux) {
	if err := feegrant.RegisterQueryHandlerClient(context.Background(), mux, feegrant.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// GetTxCmd returns the root tx command for the feegrant module.
func (AppModule) GetTxCmd() *cobra.Command {
	return cli.GetTxCmd()
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	feegrant.RegisterMsgServer(registrar, keeper.NewMsgServerImpl(am.keeper))
	feegrant.RegisterQueryServer(registrar, am.keeper)

	return nil
}

// RegisterMigrations registers module migrations.
func (am AppModule) RegisterMigrations(mr appmodule.MigrationRegistrar) error {
	m := keeper.NewMigrator(am.keeper)

	if err := mr.Register(feegrant.ModuleName, 1, m.Migrate1to2); err != nil {
		return fmt.Errorf("failed to migrate x/feegrant from version 1 to 2: %w", err)
	}

	return nil
}

// DefaultGenesis returns default genesis state as raw bytes for the feegrant module.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(feegrant.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the feegrant module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var data feegrant.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &data); err != nil {
		return errors.Wrapf(err, "failed to unmarshal %s genesis state", feegrant.ModuleName)
	}

	return feegrant.ValidateGenesis(data)
}

// InitGenesis performs genesis initialization for the feegrant module.
func (am AppModule) InitGenesis(ctx context.Context, bz json.RawMessage) error {
	var gs feegrant.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &gs); err != nil {
		return err
	}

	err := am.keeper.InitGenesis(ctx, &gs)
	if err != nil {
		return err
	}
	return nil
}

// ExportGenesis returns the exported genesis state as raw bytes for the feegrant module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs, err := am.keeper.ExportGenesis(ctx)
	if err != nil {
		return nil, err
	}

	return am.cdc.MarshalJSON(gs)
}

// ConsensusVersion implements HasConsensusVersion
func (AppModule) ConsensusVersion() uint64 { return 2 }

// EndBlock returns the end blocker for the feegrant module.
func (am AppModule) EndBlock(ctx context.Context) error {
	return EndBlocker(ctx, am.keeper)
}
