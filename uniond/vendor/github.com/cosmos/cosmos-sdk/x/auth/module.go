package auth

import (
	"context"
	"encoding/json"
	"fmt"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"google.golang.org/grpc"

	"cosmossdk.io/core/appmodule"
	appmodulev2 "cosmossdk.io/core/appmodule/v2"
	"cosmossdk.io/core/registry"
	"cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/simsx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/cosmos/cosmos-sdk/x/auth/keeper"
	"github.com/cosmos/cosmos-sdk/x/auth/simulation"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

// ConsensusVersion defines the current x/auth module consensus version.
const (
	ConsensusVersion = 5
	GovModuleName    = "gov"
)

var (
	_ module.AppModuleSimulation = AppModule{}

	_ appmodulev2.HasGenesis    = AppModule{}
	_ appmodulev2.AppModule     = AppModule{}
	_ appmodulev2.HasMigrations = AppModule{}
)

// AppModule implements an application module for the auth module.
type AppModule struct {
	accountKeeper     keeper.AccountKeeper
	randGenAccountsFn types.RandomGenesisAccountsFn
	accountsModKeeper types.AccountsModKeeper
	cdc               codec.Codec
	extOptChecker     ante.ExtensionOptionChecker
}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// NewAppModule creates a new AppModule object.
func NewAppModule(
	cdc codec.Codec,
	accountKeeper keeper.AccountKeeper,
	ak types.AccountsModKeeper,
	randGenAccountsFn types.RandomGenesisAccountsFn,
	extOptChecker ante.ExtensionOptionChecker,
) AppModule {
	return AppModule{
		accountKeeper:     accountKeeper,
		randGenAccountsFn: randGenAccountsFn,
		accountsModKeeper: ak,
		cdc:               cdc,
		extOptChecker:     extOptChecker,
	}
}

// Name returns the module's name.
// Deprecated: kept for legacy reasons.
func (AppModule) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec registers the auth module's types for the given codec.
func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	types.RegisterLegacyAminoCodec(registrar)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the auth module.
func (AppModule) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *gwruntime.ServeMux) {
	if err := types.RegisterQueryHandlerClient(context.Background(), mux, types.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// RegisterInterfaces registers interfaces and implementations of the auth module.
func (AppModule) RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	types.RegisterInterfaces(registrar)
}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	types.RegisterMsgServer(registrar, keeper.NewMsgServerImpl(am.accountKeeper))
	types.RegisterQueryServer(registrar, keeper.NewQueryServer(am.accountKeeper))

	return nil
}

// RegisterMigrations registers module migrations
func (am AppModule) RegisterMigrations(mr appmodule.MigrationRegistrar) error {
	m := keeper.NewMigrator(am.accountKeeper)
	if err := mr.Register(types.ModuleName, 1, m.Migrate1to2); err != nil {
		return fmt.Errorf("failed to migrate x/%s from version 1 to 2: %w", types.ModuleName, err)
	}

	if err := mr.Register(types.ModuleName, 2, m.Migrate2to3); err != nil {
		return fmt.Errorf("failed to migrate x/%s from version 2 to 3: %w", types.ModuleName, err)
	}
	if err := mr.Register(types.ModuleName, 3, m.Migrate3to4); err != nil {
		return fmt.Errorf("failed to migrate x/%s from version 3 to 4: %w", types.ModuleName, err)
	}
	if err := mr.Register(types.ModuleName, 4, m.Migrate4To5); err != nil {
		return fmt.Errorf("failed to migrate x/%s from version 4 to 5: %w", types.ModuleName, err)
	}

	return nil
}

// DefaultGenesis returns default genesis state as raw bytes for the auth module.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(types.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the auth module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var data types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &data); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return types.ValidateGenesis(data)
}

// InitGenesis performs genesis initialization for the auth module.
func (am AppModule) InitGenesis(ctx context.Context, data json.RawMessage) error {
	var genesisState types.GenesisState
	if err := am.cdc.UnmarshalJSON(data, &genesisState); err != nil {
		return err
	}
	return am.accountKeeper.InitGenesis(ctx, genesisState)
}

// ExportGenesis returns the exported genesis state as raw bytes for the auth
// module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs, err := am.accountKeeper.ExportGenesis(ctx)
	if err != nil {
		return nil, err
	}
	return am.cdc.MarshalJSON(gs)
}

// TxValidator implements appmodulev2.HasTxValidator.
// It replaces auth ante handlers for server/v2
func (am AppModule) TxValidator(ctx context.Context, tx transaction.Tx) error {
	validators := []appmodulev2.TxValidator[sdk.Tx]{
		ante.NewValidateBasicDecorator(am.accountKeeper.GetEnvironment()),
		ante.NewTxTimeoutHeightDecorator(am.accountKeeper.GetEnvironment()),
		ante.NewValidateMemoDecorator(am.accountKeeper),
		ante.NewConsumeGasForTxSizeDecorator(am.accountKeeper),
		ante.NewValidateSigCountDecorator(am.accountKeeper),
		ante.NewExtensionOptionsDecorator(am.extOptChecker),
	}

	sdkTx, ok := tx.(sdk.Tx)
	if !ok {
		return fmt.Errorf("invalid tx type %T, expected sdk.Tx", tx)
	}

	for _, validator := range validators {
		if err := validator.ValidateTx(ctx, sdkTx); err != nil {
			return err
		}
	}

	return nil
}

// ConsensusVersion implements appmodule.HasConsensusVersion
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the auth module
func (am AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState, am.randGenAccountsFn)
}

// ProposalMsgsX returns msgs used for governance proposals for simulations.
func (AppModule) ProposalMsgsX(weights simsx.WeightSource, reg simsx.Registry) {
	reg.Add(weights.Get("msg_update_params", 100), simulation.MsgUpdateParamsFactory())
}

// RegisterStoreDecoder registers a decoder for auth module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[types.StoreKey] = simtypes.NewStoreDecoderFuncFromCollectionsSchema(am.accountKeeper.Schema)
}
