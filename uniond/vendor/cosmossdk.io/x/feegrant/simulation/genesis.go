package simulation

import (
	"math/rand"
	"time"

	"cosmossdk.io/math"
	"cosmossdk.io/x/feegrant"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// genFeeGrants returns a slice of randomly generated allowances.
func genFeeGrants(r *rand.Rand, accounts []simtypes.Account) []feegrant.Grant {
	allowances := make([]feegrant.Grant, len(accounts)-1)
	for i := 0; i < len(accounts)-1; i++ {
		granter := accounts[i].Address
		grantee := accounts[i+1].Address
		allowances[i] = generateRandomAllowances(granter, grantee, r)
	}
	return allowances
}

func generateRandomAllowances(granter, grantee sdk.AccAddress, r *rand.Rand) feegrant.Grant {
	allowances := make([]feegrant.Grant, 3)
	spendLimit := sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(100)))
	periodSpendLimit := sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(10)))

	basic := feegrant.BasicAllowance{
		SpendLimit: spendLimit,
	}

	basicAllowance, err := feegrant.NewGrant(granter, grantee, &basic)
	if err != nil {
		panic(err)
	}
	allowances[0] = basicAllowance

	periodicAllowance, err := feegrant.NewGrant(granter, grantee, &feegrant.PeriodicAllowance{
		Basic:            basic,
		PeriodSpendLimit: periodSpendLimit,
		Period:           time.Hour,
	})
	if err != nil {
		panic(err)
	}
	allowances[1] = periodicAllowance

	filteredAllowance, err := feegrant.NewGrant(granter, grantee, &feegrant.AllowedMsgAllowance{
		Allowance:       basicAllowance.GetAllowance(),
		AllowedMessages: []string{"/cosmos.gov.v1.MsgSubmitProposal"},
	})
	if err != nil {
		panic(err)
	}
	allowances[2] = filteredAllowance

	return allowances[r.Intn(len(allowances))]
}

// RandomizedGenState generates a random GenesisState for feegrant
func RandomizedGenState(simState *module.SimulationState) {
	var feegrants []feegrant.Grant

	simState.AppParams.GetOrGenerate(
		"feegrant", &feegrants, simState.Rand,
		func(r *rand.Rand) { feegrants = genFeeGrants(r, simState.Accounts) },
	)

	feegrantGenesis := feegrant.NewGenesisState(feegrants)
	bz, err := simState.Cdc.MarshalJSON(feegrantGenesis)
	if err != nil {
		panic(err)
	}

	simState.GenState[feegrant.ModuleName] = bz
}
