package gov

import (
	"fmt"
	"maps"
	"slices"
	"strings"

	modulev1 "cosmossdk.io/api/cosmos/gov/module/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	govclient "cosmossdk.io/x/gov/client"
	"cosmossdk.io/x/gov/keeper"
	govtypes "cosmossdk.io/x/gov/types"
	"cosmossdk.io/x/gov/types/v1beta1"

	"github.com/cosmos/cosmos-sdk/codec"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var _ depinject.OnePerModuleType = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

func init() {
	appconfig.RegisterModule(
		&modulev1.Module{},
		appconfig.Invoke(InvokeAddRoutes, InvokeSetHooks),
		appconfig.Provide(ProvideModule))
}

type ModuleInputs struct {
	depinject.In

	Config                *modulev1.Module
	Cdc                   codec.Codec
	Environment           appmodule.Environment
	ModuleKey             depinject.OwnModuleKey
	LegacyProposalHandler []govclient.ProposalHandler `optional:"true"`

	AccountKeeper govtypes.AccountKeeper
	BankKeeper    govtypes.BankKeeper
	StakingKeeper govtypes.StakingKeeper
	PoolKeeper    govtypes.PoolKeeper
}

type ModuleOutputs struct {
	depinject.Out

	Module       appmodule.AppModule
	Keeper       *keeper.Keeper
	HandlerRoute v1beta1.HandlerRoute
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	defaultConfig := keeper.DefaultConfig()
	if in.Config.MaxTitleLen != 0 {
		defaultConfig.MaxTitleLen = in.Config.MaxTitleLen
	}
	if in.Config.MaxMetadataLen != 0 {
		defaultConfig.MaxMetadataLen = in.Config.MaxMetadataLen
	}
	if in.Config.MaxSummaryLen != 0 {
		defaultConfig.MaxSummaryLen = in.Config.MaxSummaryLen
	}
	if in.LegacyProposalHandler == nil {
		in.LegacyProposalHandler = []govclient.ProposalHandler{}
	}

	// default to governance authority if not provided
	authority := authtypes.NewModuleAddress(govtypes.ModuleName)
	if in.Config.Authority != "" {
		authority = authtypes.NewModuleAddressOrBech32Address(in.Config.Authority)
	}
	authorityAddr, err := in.AccountKeeper.AddressCodec().BytesToString(authority)
	if err != nil {
		panic(err)
	}

	k := keeper.NewKeeper(
		in.Cdc,
		in.Environment,
		in.AccountKeeper,
		in.BankKeeper,
		in.StakingKeeper,
		in.PoolKeeper,
		defaultConfig,
		authorityAddr,
	)
	m := NewAppModule(in.Cdc, k, in.AccountKeeper, in.BankKeeper, in.PoolKeeper, in.LegacyProposalHandler...)
	hr := v1beta1.HandlerRoute{Handler: v1beta1.ProposalHandler, RouteKey: govtypes.RouterKey}

	return ModuleOutputs{Module: m, Keeper: k, HandlerRoute: hr}
}

func InvokeAddRoutes(keeper *keeper.Keeper, routes []v1beta1.HandlerRoute) {
	if keeper == nil || routes == nil {
		return
	}

	// Default route order is a lexical sort by RouteKey.
	// Explicit ordering can be added to the module config if required.
	slices.SortFunc(routes, func(x, y v1beta1.HandlerRoute) int {
		return strings.Compare(x.RouteKey, y.RouteKey)
	})

	router := v1beta1.NewRouter()
	for _, r := range routes {
		router.AddRoute(r.RouteKey, r.Handler)
	}
	keeper.SetLegacyRouter(router)
}

func InvokeSetHooks(keeper *keeper.Keeper, govHooks map[string]govtypes.GovHooksWrapper) error {
	if keeper == nil || govHooks == nil {
		return nil
	}

	// Default ordering is lexical by module name.
	// Explicit ordering can be added to the module config if required.
	modNames := slices.Sorted(maps.Keys(govHooks))
	var multiHooks govtypes.MultiGovHooks
	for _, modName := range modNames {
		hook, ok := govHooks[modName]
		if !ok {
			return fmt.Errorf("can't find staking hooks for module %s", modName)
		}
		multiHooks = append(multiHooks, hook)
	}

	keeper.SetHooks(multiHooks)
	return nil
}
