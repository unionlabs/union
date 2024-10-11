package capability

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"

	"cosmossdk.io/core/appmodule"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"github.com/cosmos/ibc-go/modules/capability/keeper"
	"github.com/cosmos/ibc-go/modules/capability/simulation"
	"github.com/cosmos/ibc-go/modules/capability/types"
)

var (
	_ module.AppModule              = (*AppModule)(nil)
	_ module.AppModuleSimulation    = (*AppModule)(nil)
	_ appmodule.HasConsensusVersion = (*AppModule)(nil)
	_ module.HasGenesis             = (*AppModule)(nil)
	_ appmodule.AppModule           = (*AppModule)(nil)
	_ appmodule.HasBeginBlocker     = (*AppModule)(nil)
)

// ----------------------------------------------------------------------------
// AppModuleBasic
// ----------------------------------------------------------------------------

// AppModuleBasic implements the AppModuleBasic interface for the capability module.
type AppModuleBasic struct {
	cdc codec.Codec
}

func NewAppModuleBasic(cdc codec.Codec) AppModuleBasic {
	return AppModuleBasic{cdc: cdc}
}

// Name returns the capability module's name.
func (AppModule) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec does nothing. Capability does not support amino.
func (AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {}

// RegisterInterfaces registers the module's interface types
func (AppModuleBasic) RegisterInterfaces(_ cdctypes.InterfaceRegistry) {}

// DefaultGenesis returns the capability module's default genesis state.
func (am AppModuleBasic) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(types.DefaultGenesis())
}

// ValidateGenesis performs genesis state validation for the capability module.
func (am AppModuleBasic) ValidateGenesis(bz json.RawMessage) error {
	var genState types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &genState); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}
	return genState.Validate()
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the capability module.
func (AppModuleBasic) RegisterGRPCGatewayRoutes(_ client.Context, _ *gwruntime.ServeMux) {
}

// GetTxCmd returns the capability module's root tx command.
func (AppModuleBasic) GetTxCmd() *cobra.Command { return nil }

// GetQueryCmd returns the capability module's root query command.
func (AppModuleBasic) GetQueryCmd() *cobra.Command { return nil }

// ----------------------------------------------------------------------------
// AppModule
// ----------------------------------------------------------------------------

// AppModule implements the AppModule interface for the capability module.
type AppModule struct {
	AppModuleBasic

	keeper keeper.Keeper

	sealKeeper bool
}

func NewAppModule(cdc codec.Codec, capabilityKeeper keeper.Keeper, sealKeeper bool) AppModule {
	return AppModule{
		AppModuleBasic: NewAppModuleBasic(cdc),
		keeper:         capabilityKeeper,
		sealKeeper:     sealKeeper,
	}
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// InitGenesis performs the capability module's genesis initialization It returns
// no validator updates.
func (am AppModule) InitGenesis(ctx context.Context, gs json.RawMessage) error {
	var genState types.GenesisState
	// Initialize global index to index in genesis state
	am.cdc.MustUnmarshalJSON(gs, &genState)

	InitGenesis(ctx, am.keeper, genState)
	return nil
}

// ExportGenesis returns the capability module's exported genesis state as raw JSON bytes.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	genState := ExportGenesis(ctx, am.keeper)
	return am.cdc.MarshalJSON(genState)
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return 1 }

// BeginBlock executes all ABCI BeginBlock logic respective to the capability module.
// BeginBlocker calls InitMemStore to assert that the memory store is initialized.
// It's safe to run multiple times.
func (am AppModule) BeginBlock(ctx context.Context) error {
	defer telemetry.ModuleMeasureSince(types.ModuleName, time.Now(), telemetry.MetricKeyBeginBlocker)

	am.keeper.InitMemStore(sdk.UnwrapSDKContext(ctx))

	if am.sealKeeper && !am.keeper.IsSealed() {
		am.keeper.Seal()
	}

	return nil
}

// GenerateGenesisState creates a randomized GenState of the capability module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// RegisterStoreDecoder registers a decoder for capability module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[types.StoreKey] = simulation.NewDecodeStore(am.cdc)
}

// WeightedOperations returns the all the gov module operations with their respective weights.
func (AppModule) WeightedOperations(simState module.SimulationState) []simtypes.WeightedOperation {
	return nil
}
