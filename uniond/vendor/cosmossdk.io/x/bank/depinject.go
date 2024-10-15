package bank

import (
	"fmt"
	"maps"
	"slices"
	"sort"

	modulev1 "cosmossdk.io/api/cosmos/bank/module/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	"cosmossdk.io/x/bank/keeper"
	"cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/codec"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var _ depinject.OnePerModuleType = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

func init() {
	appconfig.RegisterModule(
		&modulev1.Module{},
		appconfig.Provide(ProvideModule),
		appconfig.Invoke(InvokeSetSendRestrictions),
	)
}

type ModuleInputs struct {
	depinject.In

	Config      *modulev1.Module
	Cdc         codec.Codec
	Environment appmodule.Environment

	AccountKeeper types.AccountKeeper
}

type ModuleOutputs struct {
	depinject.Out

	BankKeeper keeper.BaseKeeper
	Module     appmodule.AppModule
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	// Configure blocked module accounts.
	//
	// Default behavior for blockedAddresses is to regard any module mentioned in
	// AccountKeeper's module account permissions as blocked.
	blockedAddresses := make(map[string]bool)
	if len(in.Config.BlockedModuleAccountsOverride) > 0 {
		for _, moduleName := range in.Config.BlockedModuleAccountsOverride {
			addrStr, err := in.AccountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress(moduleName))
			if err != nil {
				panic(err)
			}
			blockedAddresses[addrStr] = true
		}
	} else {
		for _, permission := range in.AccountKeeper.GetModulePermissions() {
			addrStr, err := in.AccountKeeper.AddressCodec().BytesToString(permission.GetAddress())
			if err != nil {
				panic(err)
			}
			blockedAddresses[addrStr] = true
		}
	}

	// default to governance authority if not provided
	authority := authtypes.NewModuleAddress(types.GovModuleName)
	if in.Config.Authority != "" {
		authority = authtypes.NewModuleAddressOrBech32Address(in.Config.Authority)
	}

	authStr, err := in.AccountKeeper.AddressCodec().BytesToString(authority)
	if err != nil {
		panic(err)
	}

	bankKeeper := keeper.NewBaseKeeper(
		in.Environment,
		in.Cdc,
		in.AccountKeeper,
		blockedAddresses,
		authStr,
	)
	m := NewAppModule(in.Cdc, bankKeeper, in.AccountKeeper)

	return ModuleOutputs{BankKeeper: bankKeeper, Module: m}
}

func InvokeSetSendRestrictions(
	config *modulev1.Module,
	keeper keeper.BaseKeeper,
	restrictions map[string]types.SendRestrictionFn,
) error {
	if config == nil {
		return nil
	}

	modules := slices.Collect(maps.Keys(restrictions))
	order := config.RestrictionsOrder
	if len(order) == 0 {
		order = modules
		sort.Strings(order)
	}

	if len(order) != len(modules) {
		return fmt.Errorf("len(restrictions order: %v) != len(restriction modules: %v)", order, modules)
	}

	if len(modules) == 0 {
		return nil
	}

	for _, module := range order {
		restriction, ok := restrictions[module]
		if !ok {
			return fmt.Errorf("can't find send restriction for module %s", module)
		}

		keeper.AppendSendRestriction(restriction)
	}

	return nil
}
