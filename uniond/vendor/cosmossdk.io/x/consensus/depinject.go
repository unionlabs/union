package consensus

import (
	"context"

	modulev1 "cosmossdk.io/api/cosmos/consensus/module/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/server"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	"cosmossdk.io/x/consensus/keeper"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdkaddress "github.com/cosmos/cosmos-sdk/types/address"
)

var _ depinject.OnePerModuleType = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

func init() {
	appconfig.RegisterModule(
		&modulev1.Module{},
		appconfig.Provide(ProvideModule),
		appconfig.Provide(ProvideAppVersionModifier),
	)
}

type ModuleInputs struct {
	depinject.In

	Config       *modulev1.Module
	Cdc          codec.Codec
	Environment  appmodule.Environment
	AddressCodec address.Codec
}

type ModuleOutputs struct {
	depinject.Out

	Keeper        keeper.Keeper
	Module        appmodule.AppModule
	BaseAppOption runtime.BaseAppOption
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	// default to governance authority if not provided
	authority := sdkaddress.Module("gov")
	if in.Config.Authority != "" {
		bz, err := in.AddressCodec.StringToBytes(in.Config.Authority)
		if err != nil {
			authority = sdkaddress.Module(in.Config.Authority)
		} else {
			authority = bz
		}
	}

	authorityAddr, err := in.AddressCodec.BytesToString(authority)
	if err != nil {
		panic(err)
	}

	k := keeper.NewKeeper(in.Cdc, in.Environment, authorityAddr)
	m := NewAppModule(in.Cdc, k)
	baseappOpt := func(app *baseapp.BaseApp) {
		app.SetParamStore(k.ParamsStore)
		app.SetVersionModifier(versionModifier{Keeper: k})
	}

	return ModuleOutputs{
		Keeper:        k,
		Module:        m,
		BaseAppOption: baseappOpt,
	}
}

type versionModifier struct {
	Keeper keeper.Keeper
}

func (v versionModifier) SetAppVersion(ctx context.Context, version uint64) error {
	params, err := v.Keeper.Params(ctx, nil)
	if err != nil {
		return err
	}

	updatedParams := params.Params
	updatedParams.Version.App = version

	err = v.Keeper.ParamsStore.Set(ctx, *updatedParams)
	if err != nil {
		return err
	}

	return nil
}

func (v versionModifier) AppVersion(ctx context.Context) (uint64, error) {
	params, err := v.Keeper.Params(ctx, nil)
	if err != nil {
		return 0, err
	}

	return params.Params.Version.GetApp(), nil
}

func ProvideAppVersionModifier(k keeper.Keeper) server.VersionModifier {
	return versionModifier{Keeper: k}
}
