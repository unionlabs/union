package simulation

import (
	"math/rand"

	"cosmossdk.io/math"
	"cosmossdk.io/x/distribution/types"

	"github.com/cosmos/cosmos-sdk/types/module"
)

// Simulation parameter constants
const (
	CommunityTax    = "community_tax"
	WithdrawEnabled = "withdraw_enabled"
)

// GenCommunityTax randomized CommunityTax
func GenCommunityTax(r *rand.Rand) math.LegacyDec {
	return math.LegacyNewDecWithPrec(1, 2).Add(math.LegacyNewDecWithPrec(int64(r.Intn(30)), 2))
}

// GenWithdrawEnabled returns a randomized WithdrawEnabled parameter.
func GenWithdrawEnabled(r *rand.Rand) bool {
	return r.Int63n(101) <= 95 // 95% chance of withdraws being enabled
}

// RandomizedGenState generates a random GenesisState for distribution
func RandomizedGenState(simState *module.SimulationState) {
	var communityTax math.LegacyDec
	simState.AppParams.GetOrGenerate(CommunityTax, &communityTax, simState.Rand, func(r *rand.Rand) { communityTax = GenCommunityTax(r) })

	var withdrawEnabled bool
	simState.AppParams.GetOrGenerate(WithdrawEnabled, &withdrawEnabled, simState.Rand, func(r *rand.Rand) { withdrawEnabled = GenWithdrawEnabled(r) })

	distrGenesis := types.GenesisState{
		FeePool: types.InitialFeePool(),
		Params: types.Params{
			CommunityTax:        communityTax,
			WithdrawAddrEnabled: withdrawEnabled,
		},
	}

	simState.GenState[types.ModuleName] = simState.Cdc.MustMarshalJSON(&distrGenesis)
}
