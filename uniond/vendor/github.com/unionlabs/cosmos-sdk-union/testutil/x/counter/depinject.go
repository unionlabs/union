package counter

import (
	"fmt"
	"maps"
	"slices"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"

	"github.com/cosmos/cosmos-sdk/testutil/x/counter/keeper"
	"github.com/cosmos/cosmos-sdk/testutil/x/counter/types"
)

var _ depinject.OnePerModuleType = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

func init() {
	appconfig.RegisterModule(
		&types.Module{},
		appconfig.Provide(ProvideModule),
		appconfig.Invoke(InvokeSetHooks),
	)
}

type ModuleInputs struct {
	depinject.In

	Config      *types.Module
	Environment appmodule.Environment
}

type ModuleOutputs struct {
	depinject.Out

	Keeper *keeper.Keeper
	Module appmodule.AppModule
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	k := keeper.NewKeeper(in.Environment)
	m := NewAppModule(k)

	return ModuleOutputs{
		Keeper: k,
		Module: m,
	}
}

func InvokeSetHooks(keeper *keeper.Keeper, counterHooks map[string]types.CounterHooksWrapper) error {
	if keeper == nil {
		return fmt.Errorf("keeper is nil")
	}
	if counterHooks == nil {
		return fmt.Errorf("counterHooks is nil")
	}

	// Default ordering is lexical by module name.
	// Explicit ordering can be added to the module config if required.
	modNames := slices.Sorted(maps.Keys(counterHooks))
	var multiHooks types.MultiCounterHooks
	for _, modName := range modNames {
		hook, ok := counterHooks[modName]
		if !ok {
			return fmt.Errorf("can't find hooks for module %s", modName)
		}
		multiHooks = append(multiHooks, hook)
	}

	keeper.SetHooks(multiHooks)
	return nil
}
