package upgrade

import (
	"github.com/spf13/cast"

	modulev1 "cosmossdk.io/api/cosmos/upgrade/module/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	coreserver "cosmossdk.io/core/server"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	"cosmossdk.io/x/upgrade/keeper"
	"cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/server"
	"github.com/cosmos/cosmos-sdk/types/module"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var _ depinject.OnePerModuleType = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

func init() {
	appconfig.RegisterModule(&modulev1.Module{},
		appconfig.Provide(ProvideModule),
		appconfig.Invoke(PopulateVersionMap),
	)
}

type ModuleInputs struct {
	depinject.In

	Config             *modulev1.Module
	Environment        appmodule.Environment
	Cdc                codec.Codec
	AddressCodec       address.Codec
	AppVersionModifier coreserver.VersionModifier
	ConsensusKeeper    types.ConsensusKeeper

	DynamicConfig coreserver.DynamicConfig `optional:"true"`
}

type ModuleOutputs struct {
	depinject.Out

	UpgradeKeeper *keeper.Keeper
	Module        appmodule.AppModule
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	var (
		homePath           string
		skipUpgradeHeights = make(map[int64]bool)
	)

	if in.DynamicConfig != nil {
		skipUpgrades := cast.ToIntSlice(in.DynamicConfig.Get(server.FlagUnsafeSkipUpgrades))
		for _, h := range skipUpgrades {
			skipUpgradeHeights[int64(h)] = true
		}

		homePath = in.DynamicConfig.GetString(flags.FlagHome)
	}

	// default to governance authority if not provided
	authority := authtypes.NewModuleAddress(types.GovModuleName)
	if in.Config.Authority != "" {
		authority = authtypes.NewModuleAddressOrBech32Address(in.Config.Authority)
	}

	authorityStr, err := in.AddressCodec.BytesToString(authority)
	if err != nil {
		panic(err)
	}

	// set the governance module account as the authority for conducting upgrades
	k := keeper.NewKeeper(in.Environment, skipUpgradeHeights, in.Cdc, homePath, in.AppVersionModifier, authorityStr, in.ConsensusKeeper)
	m := NewAppModule(k)

	return ModuleOutputs{UpgradeKeeper: k, Module: m}
}

func PopulateVersionMap(upgradeKeeper *keeper.Keeper, modules map[string]appmodule.AppModule) {
	if upgradeKeeper == nil {
		return
	}

	upgradeKeeper.SetInitVersionMap(module.NewManagerFromMap(modules).GetVersionMap())
}
