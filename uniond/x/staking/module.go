package staking

import (
	"google.golang.org/grpc"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/cosmos-sdk/types/module"

	"github.com/cosmos/cosmos-sdk/types/msgservice"
	"github.com/unionlabs/union/uniond/x/staking/client/cli"
	"github.com/unionlabs/union/uniond/x/staking/keeper"
	"github.com/unionlabs/union/uniond/x/staking/types"
)

const (
	ModuleName = "unionstaking"
)

var (
	_ module.AppModuleBasic = AppModuleBasic{}
	_ module.AppModule      = AppModule{}
)

// AppModuleBasic defines the basic application module used by the staking module.
type AppModuleBasic struct {
	cdc codec.Codec
}

// Name implements module.AppModuleBasic.
func (a AppModuleBasic) Name() string {
	return "unionstaking"
}

// RegisterGRPCGatewayRoutes implements module.AppModuleBasic.
func (a AppModuleBasic) RegisterGRPCGatewayRoutes(client.Context, *gwruntime.ServeMux) {
}

// RegisterInterfaces implements module.AppModuleBasic.
func (a AppModuleBasic) RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	registry.RegisterImplementations((*sdk.Msg)(nil),
		&types.MsgCreateUnionValidator{},
	)
	msgservice.RegisterMsgServiceDesc(registry, &types.Msg_serviceDesc)
}

// RegisterLegacyAminoCodec implements module.AppModuleBasic.
func (a AppModuleBasic) RegisterLegacyAminoCodec(*codec.LegacyAmino) {
}

// GetTxCmd returns the root tx command for the staking module.
func (amb AppModuleBasic) GetTxCmd() *cobra.Command {
	return cli.NewTxCmd(addresscodec.NewBech32Codec(sdk.GetConfig().GetBech32ValidatorAddrPrefix()))
}

// AppModule implements an application module
type AppModule struct {
	AppModuleBasic
	keeper keeper.Keeper
}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	types.RegisterMsgServer(registrar, keeper.NewMsgServerImpl(am.keeper))
	return nil
}

// NewAppModule creates a new AppModule object
func NewAppModule(keeper keeper.Keeper, cdc codec.Codec) AppModule {
	return AppModule{
		AppModuleBasic: AppModuleBasic{
			cdc,
		},
		keeper: keeper,
	}
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

// ConsensusVersion implements HasConsensusVersion
func (AppModule) ConsensusVersion() uint64 { return 1 }

// Name returns the module's name.
// Deprecated: kept for legacy reasons.
func (AppModule) Name() string { return "unionstaking" }
