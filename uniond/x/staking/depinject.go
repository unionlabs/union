package staking

import (
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/store"
	"cosmossdk.io/depinject"

	"github.com/unionlabs/union/uniond/x/staking/keeper"
	"github.com/unionlabs/union/uniond/x/staking/types"
)

var _ appmodule.AppModule = AppModule{}

func init() {
	appmodule.Register(
		&Module{},
		appmodule.Provide(ProvideModule),
	)
}

type ModuleInputs struct {
	depinject.In

	Config        *Module
	Cdc           codec.Codec
	StoreService  store.KVStoreService
	AddressCodec  address.Codec
	StakingKeeper types.StakingKeeper
}

type ModuleOutputs struct {
	depinject.Out

	Module appmodule.AppModule
	Keeper keeper.Keeper
	Hooks  stakingtypes.StakingHooksWrapper
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	k := keeper.NewKeeper(in.StakingKeeper)
	m := NewAppModule(k, in.Cdc)

	return ModuleOutputs{Module: m, Keeper: k, Out: depinject.Out{}, Hooks: stakingtypes.StakingHooksWrapper{StakingHooks: k.StakingHooks}}
}

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the slashing module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
}

// RegisterStoreDecoder registers a decoder for supply module's types
func (am AppModule) RegisterStoreDecoder(sdr simtypes.StoreDecoderRegistry) {
}
