package simulation

import (
	"math/rand"
	"time"

	v1 "cosmossdk.io/api/cosmos/gov/v1"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/authz"
	banktypes "cosmossdk.io/x/bank/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// RandomizedGenState generates a random GenesisState for authz.
func RandomizedGenState(simState *module.SimulationState) {
	var grants []authz.GrantAuthorization
	simState.AppParams.GetOrGenerate("authz", &grants, simState.Rand, func(r *rand.Rand) {
		grants = genGrant(r, simState.Accounts, simState.GenTimestamp)
	})

	authzGrantsGenesis := authz.NewGenesisState(grants)

	simState.GenState[authz.ModuleName] = simState.Cdc.MustMarshalJSON(authzGrantsGenesis)
}

// genGrant returns a slice of authorization grants.
func genGrant(r *rand.Rand, accounts []simtypes.Account, genT time.Time) []authz.GrantAuthorization {
	authorizations := make([]authz.GrantAuthorization, len(accounts)-1)
	for i := 0; i < len(accounts)-1; i++ {
		var expiration *time.Time
		if i%3 != 0 { // generate some grants with no expire time
			e := genT.AddDate(1, 0, 0)
			expiration = &e
		}
		authorizations[i] = authz.GrantAuthorization{
			Granter:       accounts[i].AddressBech32,
			Grantee:       accounts[i+1].AddressBech32,
			Authorization: generateRandomGrant(r),
			Expiration:    expiration,
		}
	}

	return authorizations
}

func generateRandomGrant(r *rand.Rand) *codectypes.Any {
	examples := []*codectypes.Any{
		must(codectypes.NewAnyWithValue(banktypes.NewSendAuthorization(sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(1000))), nil, nil))),
		must(codectypes.NewAnyWithValue(authz.NewGenericAuthorization(sdk.MsgTypeURL(&v1.MsgSubmitProposal{})))),
	}
	return examples[r.Intn(len(examples))]
}

func must[T any](r T, err error) T {
	if err != nil {
		panic(err)
	}
	return r
}
