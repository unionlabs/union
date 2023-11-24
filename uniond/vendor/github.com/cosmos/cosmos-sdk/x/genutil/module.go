package genutil

import (
	"encoding/json"
	"fmt"

	abci "github.com/cometbft/cometbft/abci/types"
	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"

	modulev1 "cosmossdk.io/api/cosmos/genutil/module/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/genesis"
	"cosmossdk.io/depinject"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

var (
	_ module.AppModuleBasic = AppModule{}
	_ module.HasABCIGenesis = AppModule{}

	_ appmodule.AppModule = AppModule{}
)

// AppModuleBasic defines the basic application module used by the genutil module.
type AppModuleBasic struct {
	GenTxValidator types.MessageValidator
}

// NewAppModuleBasic creates AppModuleBasic, validator is a function used to validate genesis
// transactions.
func NewAppModuleBasic(validator types.MessageValidator) AppModuleBasic {
	return AppModuleBasic{validator}
}

// Name returns the genutil module's name.
func (AppModuleBasic) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec registers the genutil module's types on the given LegacyAmino codec.
func (AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {}

// RegisterInterfaces registers the module's interface types
func (b AppModuleBasic) RegisterInterfaces(cdctypes.InterfaceRegistry) {}

// DefaultGenesis returns default genesis state as raw bytes for the genutil
// module.
func (AppModuleBasic) DefaultGenesis(cdc codec.JSONCodec) json.RawMessage {
	return cdc.MustMarshalJSON(types.DefaultGenesisState())
}

// ValidateGenesis performs genesis state validation for the genutil module.
func (b AppModuleBasic) ValidateGenesis(cdc codec.JSONCodec, txEncodingConfig client.TxEncodingConfig, bz json.RawMessage) error {
	var data types.GenesisState
	if err := cdc.UnmarshalJSON(bz, &data); err != nil {
		return fmt.Errorf("failed to unmarshal %s genesis state: %w", types.ModuleName, err)
	}

	return types.ValidateGenesis(&data, txEncodingConfig.TxJSONDecoder(), b.GenTxValidator)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the genutil module.
func (AppModuleBasic) RegisterGRPCGatewayRoutes(_ client.Context, _ *gwruntime.ServeMux) {
}

// AppModule implements an application module for the genutil module.
type AppModule struct {
	AppModuleBasic

	accountKeeper    types.AccountKeeper
	stakingKeeper    types.StakingKeeper
	deliverTx        genesis.TxHandler
	txEncodingConfig client.TxEncodingConfig
}

// NewAppModule creates a new AppModule object
func NewAppModule(accountKeeper types.AccountKeeper,
	stakingKeeper types.StakingKeeper, deliverTx genesis.TxHandler,
	txEncodingConfig client.TxEncodingConfig,
) module.GenesisOnlyAppModule {
	return module.NewGenesisOnlyAppModule(AppModule{
		AppModuleBasic:   AppModuleBasic{},
		accountKeeper:    accountKeeper,
		stakingKeeper:    stakingKeeper,
		deliverTx:        deliverTx,
		txEncodingConfig: txEncodingConfig,
	})
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (AppModule) IsAppModule() {}

// InitGenesis performs genesis initialization for the genutil module.
func (am AppModule) InitGenesis(ctx sdk.Context, cdc codec.JSONCodec, data json.RawMessage) []abci.ValidatorUpdate {
	var genesisState types.GenesisState
	cdc.MustUnmarshalJSON(data, &genesisState)
	validators, err := InitGenesis(ctx, am.stakingKeeper, am.deliverTx, genesisState, am.txEncodingConfig)
	if err != nil {
		panic(err)
	}
	return validators
}

// ExportGenesis returns the exported genesis state as raw bytes for the genutil
// module.
func (am AppModule) ExportGenesis(_ sdk.Context, cdc codec.JSONCodec) json.RawMessage {
	return am.DefaultGenesis(cdc)
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return 1 }

func init() {
	appmodule.Register(&modulev1.Module{},
		appmodule.Provide(ProvideModule),
	)
}

// ModuleInputs defines the inputs needed for the genutil module.
type ModuleInputs struct {
	depinject.In

	AccountKeeper types.AccountKeeper
	StakingKeeper types.StakingKeeper
	DeliverTx     genesis.TxHandler
	Config        client.TxConfig
}

func ProvideModule(in ModuleInputs) appmodule.AppModule {
	m := NewAppModule(in.AccountKeeper, in.StakingKeeper, in.DeliverTx, in.Config)
	return m
}
