package vesting

import (
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"

	"github.com/cosmos/cosmos-sdk/x/auth/keeper"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
)

var _ appmodule.AppModule = AppModule{}

// AppModule implementing the AppModule interface.
type AppModule struct {
	accountKeeper keeper.AccountKeeper
	bankKeeper    types.BankKeeper
}

func NewAppModule(ak keeper.AccountKeeper, bk types.BankKeeper) AppModule {
	return AppModule{
		accountKeeper: ak,
		bankKeeper:    bk,
	}
}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// Name returns the module's name.
// Deprecated: kept for legacy reasons.
func (AppModule) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec registers the module's types with the given codec.
func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	types.RegisterLegacyAminoCodec(registrar)
}

// RegisterInterfaces registers the module's interfaces and implementations with
// the given interface registry.
func (AppModule) RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	types.RegisterInterfaces(registrar)
}

// ConsensusVersion implements HasConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return 1 }
