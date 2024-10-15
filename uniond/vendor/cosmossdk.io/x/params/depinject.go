package params

import (
	modulev1 "cosmossdk.io/api/cosmos/params/module/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	store "cosmossdk.io/store/types"
	govv1beta1 "cosmossdk.io/x/gov/types/v1beta1"
	"cosmossdk.io/x/params/keeper"
	"cosmossdk.io/x/params/types"
	"cosmossdk.io/x/params/types/proposal"

	"github.com/cosmos/cosmos-sdk/codec"
)

var _ depinject.OnePerModuleType = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

func init() {
	appconfig.RegisterModule(&modulev1.Module{},
		appconfig.Provide(
			ProvideModule,
			ProvideSubspace,
		))
}

type ModuleInputs struct {
	depinject.In

	KvStoreKey        *store.KVStoreKey
	TransientStoreKey *store.TransientStoreKey
	Cdc               codec.Codec
	LegacyAmino       *codec.LegacyAmino
}

type ModuleOutputs struct {
	depinject.Out

	ParamsKeeper keeper.Keeper
	Module       appmodule.AppModule
	GovHandler   govv1beta1.HandlerRoute
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	k := keeper.NewKeeper(in.Cdc, in.LegacyAmino, in.KvStoreKey, in.TransientStoreKey)

	m := NewAppModule(k)
	govHandler := govv1beta1.HandlerRoute{RouteKey: proposal.RouterKey, Handler: NewParamChangeProposalHandler(k)}

	return ModuleOutputs{ParamsKeeper: k, Module: m, GovHandler: govHandler}
}

type SubspaceInputs struct {
	depinject.In

	Key       depinject.ModuleKey
	Keeper    keeper.Keeper
	KeyTables map[string]types.KeyTable
}

func ProvideSubspace(in SubspaceInputs) types.Subspace {
	moduleName := in.Key.Name()
	kt, exists := in.KeyTables[moduleName]
	if !exists {
		return in.Keeper.Subspace(moduleName)
	}
	return in.Keeper.Subspace(moduleName).WithKeyTable(kt)
}
