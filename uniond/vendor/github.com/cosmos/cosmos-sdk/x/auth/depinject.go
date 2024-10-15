package auth

import (
	modulev1 "cosmossdk.io/api/cosmos/auth/module/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/cosmos/cosmos-sdk/x/auth/keeper"
	"github.com/cosmos/cosmos-sdk/x/auth/simulation"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

var _ depinject.OnePerModuleType = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

func init() {
	appconfig.RegisterModule(&modulev1.Module{},
		appconfig.Provide(ProvideModule),
	)
}

type ModuleInputs struct {
	depinject.In

	Config            *modulev1.Module
	Environment       appmodule.Environment
	Cdc               codec.Codec
	AccountsModKeeper types.AccountsModKeeper

	AddressCodec            address.Codec
	RandomGenesisAccountsFn types.RandomGenesisAccountsFn `optional:"true"`
	AccountI                func() sdk.AccountI           `optional:"true"`

	ExtensionOptionChecker ante.ExtensionOptionChecker `optional:"true"`
}

type ModuleOutputs struct {
	depinject.Out

	AccountKeeper keeper.AccountKeeper
	Module        appmodule.AppModule
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	maccPerms := map[string][]string{}
	for _, permission := range in.Config.ModuleAccountPermissions {
		maccPerms[permission.Account] = permission.Permissions
	}

	// default to governance authority if not provided
	authority := types.NewModuleAddress(GovModuleName)
	if in.Config.Authority != "" {
		authority = types.NewModuleAddressOrBech32Address(in.Config.Authority)
	}

	if in.RandomGenesisAccountsFn == nil {
		in.RandomGenesisAccountsFn = simulation.RandomGenesisAccounts
	}

	if in.AccountI == nil {
		in.AccountI = types.ProtoBaseAccount
	}

	auth, err := in.AddressCodec.BytesToString(authority)
	if err != nil {
		panic(err)
	}

	k := keeper.NewAccountKeeper(in.Environment, in.Cdc, in.AccountI, in.AccountsModKeeper, maccPerms, in.AddressCodec, in.Config.Bech32Prefix, auth)
	m := NewAppModule(in.Cdc, k, in.AccountsModKeeper, in.RandomGenesisAccountsFn, in.ExtensionOptionChecker)

	return ModuleOutputs{AccountKeeper: k, Module: m}
}
