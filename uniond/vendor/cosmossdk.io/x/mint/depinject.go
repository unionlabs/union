package mint

import (
	modulev1 "cosmossdk.io/api/cosmos/mint/module/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	epochstypes "cosmossdk.io/x/epochs/types"
	"cosmossdk.io/x/mint/keeper"
	"cosmossdk.io/x/mint/types"

	"github.com/cosmos/cosmos-sdk/codec"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
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

	ModuleKey              depinject.OwnModuleKey
	Config                 *modulev1.Module
	Environment            appmodule.Environment
	Cdc                    codec.Codec
	MintFn                 types.MintFn                 `optional:"true"`
	InflationCalculationFn types.InflationCalculationFn `optional:"true"` // deprecated

	AccountKeeper types.AccountKeeper
	BankKeeper    types.BankKeeper
	StakingKeeper types.StakingKeeper
}

type ModuleOutputs struct {
	depinject.Out

	MintKeeper keeper.Keeper
	Module     appmodule.AppModule
	EpochHooks epochstypes.EpochHooksWrapper
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	feeCollectorName := in.Config.FeeCollectorName
	if feeCollectorName == "" {
		feeCollectorName = authtypes.FeeCollectorName
	}

	// default to governance authority if not provided
	authority := authtypes.NewModuleAddress(types.GovModuleName)
	if in.Config.Authority != "" {
		authority = authtypes.NewModuleAddressOrBech32Address(in.Config.Authority)
	}

	as, err := in.AccountKeeper.AddressCodec().BytesToString(authority)
	if err != nil {
		panic(err)
	}

	k := keeper.NewKeeper(
		in.Cdc,
		in.Environment,
		in.StakingKeeper,
		in.AccountKeeper,
		in.BankKeeper,
		feeCollectorName,
		as,
	)

	if in.MintFn != nil && in.InflationCalculationFn != nil {
		panic("MintFn and InflationCalculationFn cannot both be set")
	}

	// if no mintFn is provided, use the default minting function
	if in.MintFn == nil {
		// if no inflationCalculationFn is provided, use the default inflation calculation function
		if in.InflationCalculationFn == nil {
			in.InflationCalculationFn = types.DefaultInflationCalculationFn
		}

		in.MintFn = k.DefaultMintFn(in.InflationCalculationFn)
	}

	m := NewAppModule(in.Cdc, k, in.AccountKeeper, in.MintFn)

	return ModuleOutputs{MintKeeper: k, Module: m, EpochHooks: epochstypes.EpochHooksWrapper{EpochHooks: m}}
}
