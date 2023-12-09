package simulation

import (
	"encoding/json"
	"fmt"
	"math/rand"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

// RandomGenesisDefaultSendEnabledParam computes randomized allow all send transfers param for the bank module
func RandomGenesisDefaultSendEnabledParam(r *rand.Rand) bool {
	// 90% chance of transfers being enabled or P(a) = 0.9 for success
	return r.Int63n(100) < 90
}

// RandomGenesisSendEnabled creates randomized values for the SendEnabled slice.
func RandomGenesisSendEnabled(r *rand.Rand, bondDenom string) []types.SendEnabled {
	rv := make([]types.SendEnabled, 0, 2)
	// 60% of the time, add a denom specific record.
	if r.Int63n(100) < 60 {
		// 75% of the those times, set send enabled to true.
		bondEnabled := r.Int63n(100) < 75
		rv = append(rv, types.SendEnabled{Denom: bondDenom, Enabled: bondEnabled})
	}
	// Probabilities:
	//   P(a)    = 60.0% = There's SendEnable entry for the bond denom = .600
	//   P(a)'   = 40.0% = There is NOT a SendEnable entry for the bond denom  = 1 - P(a) = 1 - .600 = .400
	//   P(b)    = 75.0% = The SendEnable entry is true (if there is such an entry) = .750
	//   P(b)'   = 25.0% = The SendEnable entry is false (if there is such an entry)  = 1 - P(b) = 1 - .750 = .250
	//   P(c)    = 90.0% = The default send enabled is true (defined in RandomGenesisDefaultSendEnabledParam) = .900
	//   P(c)'   = 10.0% = The default send enabled is false  = 1 - P(c) = 1 - .900 = .100
	//
	//   P(st)   = 45.0% = There's a SendEnable entry that's true   = P(a)*P(b)  = .600*.750 = .450
	//   P(sf)   = 15.0% = There's a SendEnable entry that's false  = P(a)*P(b)' = .600*.250 = .150
	//
	//   P(a'c)  = 36.0% = No SendEnabled entry AND default is true         = P(a)'*P(c)  = .400*.900 = .360
	//   P(a'c') =  4.0% = No SendEnabled entry AND default is false        = P(a)'*P(c)' = .400*.100 = .040
	//   P(stc)  = 40.5% = SendEnabled entry is true AND default is true    = P(st)*P(c)  = .450*.900 = .405
	//   P(stc') =  4.5% = SendEnabled entry is true AND default is false   = P(st)*P(c)' = .450*.100 = .045
	//   P(sfc)  = 13.5% = SendEnabled entry is false AND default is true   = P(sf)*P(c)  = .150*.900 = .135
	//   P(sfc') =  1.5% = SendEnabled entry is false AND default is false  = P(sf)*P(c)' = .150*.100 = .015
	//
	//   P(set)  = 42.0% = SendEnabled entry that equals the default           = P(stc) + P(sfc') = .405 + .015 = .420
	//   P(sef)  = 18.0% = SendEnabled entry that does not equal the default   = P(stc') + P(sfc) = .045 + .135 = .180
	//
	//   P(t)    = 81.0% = Bond denom is sendable      = P(a'c) + P(st)  = .360 + .450 = .810
	//   P(f)    = 19.0% = Bond demon is NOT sendable  = P(a'c') + P(sf) = .040 + .150 = .190

	return rv
}

// RandomGenesisBalances returns a slice of account balances. Each account has
// a balance of simState.InitialStake for simState.BondDenom.
func RandomGenesisBalances(simState *module.SimulationState) []types.Balance {
	genesisBalances := []types.Balance{}

	for _, acc := range simState.Accounts {
		genesisBalances = append(genesisBalances, types.Balance{
			Address: acc.Address.String(),
			Coins:   sdk.NewCoins(sdk.NewCoin(simState.BondDenom, simState.InitialStake)),
		})
	}

	return genesisBalances
}

// RandomizedGenState generates a random GenesisState for bank
func RandomizedGenState(simState *module.SimulationState) {
	var defaultSendEnabledParam bool
	simState.AppParams.GetOrGenerate(string(types.KeyDefaultSendEnabled), &defaultSendEnabledParam, simState.Rand, func(r *rand.Rand) { defaultSendEnabledParam = RandomGenesisDefaultSendEnabledParam(r) })

	sendEnabled := RandomGenesisSendEnabled(simState.Rand, simState.BondDenom)

	numAccs := int64(len(simState.Accounts))
	totalSupply := simState.InitialStake.Mul(sdkmath.NewInt((numAccs + simState.NumBonded)))
	supply := sdk.NewCoins(sdk.NewCoin(simState.BondDenom, totalSupply))

	bankGenesis := types.GenesisState{
		Params:      types.NewParams(defaultSendEnabledParam),
		Balances:    RandomGenesisBalances(simState),
		Supply:      supply,
		SendEnabled: sendEnabled,
	}

	paramsBytes, err := json.MarshalIndent(&bankGenesis.Params, "", " ")
	if err != nil {
		panic(err)
	}
	fmt.Printf("Selected randomly generated bank parameters:\n%s\n", paramsBytes)
	simState.GenState[types.ModuleName] = simState.Cdc.MustMarshalJSON(&bankGenesis)
}
