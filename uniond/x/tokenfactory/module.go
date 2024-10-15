/*
The tokenfactory module allows any account to create a new token with
the name `factory/{creator address}/{subdenom}`.

- Mint and burn user denom to and form any account
- Create a transfer of their denom between any two accounts
- Change the admin. In the future, more admin capabilities may be added.
*/
package tokenfactory

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"
	"google.golang.org/grpc"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"union/x/tokenfactory/client/cli"
	"union/x/tokenfactory/keeper"
	simulation "union/x/tokenfactory/simulation"
	"union/x/tokenfactory/types"
)

var (
	_ appmodule.AppModule             = AppModule{}
	_ appmodule.HasGenesis            = AppModule{}
	_ appmodule.HasRegisterInterfaces = AppModule{}

	_ module.HasAminoCodec  = AppModule{}
	_ module.HasGRPCGateway = AppModule{}
)

// ConsensusVersion defines the current x/tokenfactory module consensus version.
const ConsensusVersion = 1

// ----------------------------------------------------------------------------
// AppModule
// ----------------------------------------------------------------------------

// AppModule implements the AppModule interface for the capability module.
type AppModule struct {
	cdc           codec.Codec
	keeper        keeper.Keeper
	accountKeeper types.AccountKeeper
	bankKeeper    types.BankKeeper
}

func NewAppModule(
	cdc codec.Codec,
	keeper keeper.Keeper,
	accountKeeper types.AccountKeeper,
	bankKeeper types.BankKeeper,
) AppModule {
	return AppModule{
		cdc:           cdc,
		keeper:        keeper,
		accountKeeper: accountKeeper,
		bankKeeper:    bankKeeper,
	}
}

func (AppModule) IsAppModule() {}

func (AppModule) IsOnePerModuleType() {}

func NewAppModuleBasic() AppModule {
	return AppModule{}
}

// Name returns the x/tokenfactory module's name.
func (AppModule) Name() string {
	return types.ModuleName
}

func (AppModule) RegisterGRPCGatewayRoutes(clientCtx client.Context, serveMux *runtime.ServeMux) {
	err := types.RegisterQueryHandlerClient(context.Background(), serveMux, types.NewQueryClient(clientCtx))
	if err != nil {
		panic(err)
	}
}

func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	types.RegisterLegacyAminoCodec(registrar)
}

// RegisterInterfaces registers the module's interface types
func (a AppModule) RegisterInterfaces(reg registry.InterfaceRegistrar) {
	types.RegisterInterfaces(reg)
}

// DefaultGenesis returns the x/tokenfactory module's default genesis state.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(types.DefaultGenesis())
}

// ValidateGenesis performs genesis state validation for the wasm module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var gs types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &gs); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}
	return gs.Validate()
}

// GetTxCmd returns the x/tokenfactory module's root tx command.
func (a AppModule) GetTxCmd() *cobra.Command {
	return cli.GetTxCmd()
}

// GetQueryCmd returns the x/tokenfactory module's root query command.
func (AppModule) GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}

// QuerierRoute returns the x/tokenfactory module's query routing key.
func (AppModule) QuerierRoute() string { return types.QuerierRoute }

// RegisterServices registers a GRPC query service to respond to the
// module-specific GRPC queries.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	types.RegisterMsgServer(registrar, keeper.NewMsgServerImpl(am.keeper))
	types.RegisterQueryServer(registrar, am.keeper)
	return nil
}

// RegisterInvariants registers the x/tokenfactory module's invariants.
func (am AppModule) RegisterInvariants(_ sdk.InvariantRegistry) {}

// InitGenesis performs genesis initialization for the wasm module. It returns
// no validator updates.
func (am AppModule) InitGenesis(ctx context.Context, bz json.RawMessage) error {
	var gs types.GenesisState
	err := am.cdc.UnmarshalJSON(bz, &gs)
	if err != nil {
		return fmt.Errorf("failed to unmarshal genesis state: %s", err)
	}
	am.keeper.InitGenesis(ctx, &gs)
	return nil
}

// ExportGenesis returns the exported genesis state as raw bytes for the wasm
// module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs := am.keeper.ExportGenesis(ctx)
	return am.cdc.MarshalJSON(gs)
}

// ConsensusVersion implements ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 {
	return ConsensusVersion
}

// AppModuleSimulation functions
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// GenerateGenesisState creates a randomized GenState of the bank module.
func (am AppModule) ProposalContents(_ module.SimulationState) []simtypes.WeightedProposalMsg {
	return nil
}

// RegisterStoreDecoder registers a decoder for supply module's types
func (am AppModule) RegisterStoreDecoder(_ simtypes.StoreDecoderRegistry) {
}

// WeightedOperations returns the all the gov module operations with their respective weights.
func (am AppModule) WeightedOperations(simState module.SimulationState) []simtypes.WeightedOperation {
	return simulation.WeightedOperations(&simState, am.keeper, am.accountKeeper, am.bankKeeper)
}
