package staking

import (
	"fmt"
	"maps"
	"slices"
	"sort"

	modulev1 "cosmossdk.io/api/cosmos/staking/module/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/comet"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	"cosmossdk.io/x/staking/keeper"
	"cosmossdk.io/x/staking/simulation"
	"cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/simsx"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var _ depinject.OnePerModuleType = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

func init() {
	appconfig.RegisterModule(
		&modulev1.Module{},
		appconfig.Provide(ProvideModule),
		appconfig.Invoke(InvokeSetStakingHooks),
	)
}

type ModuleInputs struct {
	depinject.In

	Config                *modulev1.Module
	ValidatorAddressCodec address.ValidatorAddressCodec
	ConsensusAddressCodec address.ConsensusAddressCodec
	AccountKeeper         types.AccountKeeper
	BankKeeper            types.BankKeeper
	ConsensusKeeper       types.ConsensusKeeper
	Cdc                   codec.Codec
	Environment           appmodule.Environment
	CometInfoService      comet.Service
}

// Dependency Injection Outputs
type ModuleOutputs struct {
	depinject.Out

	StakingKeeper *keeper.Keeper
	Module        appmodule.AppModule
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
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
		in.AccountKeeper,
		in.BankKeeper,
		in.ConsensusKeeper,
		as,
		in.ValidatorAddressCodec,
		in.ConsensusAddressCodec,
		in.CometInfoService,
	)
	m := NewAppModule(in.Cdc, k)
	return ModuleOutputs{StakingKeeper: k, Module: m}
}

func InvokeSetStakingHooks(
	config *modulev1.Module,
	keeper *keeper.Keeper,
	stakingHooks map[string]types.StakingHooksWrapper,
) error {
	// all arguments to invokers are optional
	if keeper == nil || config == nil {
		return nil
	}

	if len(stakingHooks) == 0 {
		return nil
	}

	modNames := slices.Collect(maps.Keys(stakingHooks))
	order := config.HooksOrder
	if len(order) == 0 {
		order = modNames
		sort.Strings(order)
	}

	if len(order) != len(modNames) {
		return fmt.Errorf("len(hooks_order: %v) != len(hooks modules: %v)", order, modNames)
	}

	var multiHooks types.MultiStakingHooks
	for _, modName := range order {
		hook, ok := stakingHooks[modName]
		if !ok {
			return fmt.Errorf("can't find staking hooks for module %s", modName)
		}

		multiHooks = append(multiHooks, hook)
	}

	keeper.SetHooks(multiHooks)
	return nil
}

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the staking module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// RegisterStoreDecoder registers a decoder for staking module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
	sdr[types.StoreKey] = simulation.NewDecodeStore(am.cdc)
}

// ProposalMsgsX returns msgs used for governance proposals for simulations.
func (AppModule) ProposalMsgsX(weights simsx.WeightSource, reg simsx.Registry) {
	reg.Add(weights.Get("msg_update_params", 100), simulation.MsgUpdateParamsFactory())
}

// WeightedOperationsX returns the all the staking module operations with their respective weights.
func (am AppModule) WeightedOperationsX(weights simsx.WeightSource, reg simsx.Registry) {
	reg.Add(weights.Get("msg_create_validator", 100), simulation.MsgCreateValidatorFactory(am.keeper))
	reg.Add(weights.Get("msg_delegate", 100), simulation.MsgDelegateFactory(am.keeper))
	reg.Add(weights.Get("msg_undelegate", 100), simulation.MsgUndelegateFactory(am.keeper))
	reg.Add(weights.Get("msg_edit_validator", 5), simulation.MsgEditValidatorFactory(am.keeper))
	reg.Add(weights.Get("msg_begin_redelegate", 100), simulation.MsgBeginRedelegateFactory(am.keeper))
	reg.Add(weights.Get("msg_cancel_unbonding_delegation", 100), simulation.MsgCancelUnbondingDelegationFactory(am.keeper))
	reg.Add(weights.Get("msg_rotate_cons_pubkey", 100), simulation.MsgRotateConsPubKeyFactory(am.keeper))
}
