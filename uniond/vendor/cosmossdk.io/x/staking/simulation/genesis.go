package simulation

import (
	"math/rand"
	"time"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/types/simulation"
)

// Simulation parameter constants
const (
	unbondingTime     = "unbonding_time"
	maxValidators     = "max_validators"
	historicalEntries = "historical_entries"
	keyRotationFee    = "cons_pubkey_rotation_fee"
)

// genUnbondingTime returns randomized UnbondingTime
func genUnbondingTime(r *rand.Rand) (ubdTime time.Duration) {
	return time.Duration(simulation.RandIntBetween(r, 60, 60*60*24*3*2)) * time.Second
}

// genMaxValidators returns randomized MaxValidators
func genMaxValidators(r *rand.Rand) (maxValidators uint32) {
	return uint32(r.Intn(250) + 1)
}

// getHistEntries returns randomized HistoricalEntries between 0-100.
func getHistEntries(r *rand.Rand) uint32 {
	return uint32(r.Intn(int(0 + 1)))
}

// getKeyRotationFee returns randomized keyRotationFee between 10000-1000000.
func getKeyRotationFee(r *rand.Rand) sdk.Coin {
	return sdk.NewInt64Coin(sdk.DefaultBondDenom, r.Int63n(types.DefaultKeyRotationFee.Amount.Int64()-10000)+10000)
}

// RandomizedGenState generates a random GenesisState for staking
func RandomizedGenState(simState *module.SimulationState) {
	// params
	var (
		unbondTime        time.Duration
		maxVals           uint32
		histEntries       uint32
		minCommissionRate sdkmath.LegacyDec
		rotationFee       sdk.Coin
	)

	simState.AppParams.GetOrGenerate(unbondingTime, &unbondTime, simState.Rand, func(r *rand.Rand) { unbondTime = genUnbondingTime(r) })

	simState.AppParams.GetOrGenerate(maxValidators, &maxVals, simState.Rand, func(r *rand.Rand) { maxVals = genMaxValidators(r) })

	simState.AppParams.GetOrGenerate(historicalEntries, &histEntries, simState.Rand, func(r *rand.Rand) { histEntries = getHistEntries(r) })

	simState.AppParams.GetOrGenerate(keyRotationFee, &histEntries, simState.Rand, func(r *rand.Rand) { rotationFee = getKeyRotationFee(r) })

	// NOTE: the slashing module need to be defined after the staking module on the
	// NewSimulationManager constructor for this to work
	simState.UnbondTime = unbondTime
	params := types.NewParams(simState.UnbondTime, maxVals, 7, simState.BondDenom, minCommissionRate, rotationFee, 50, 128)

	// validators & delegations
	var (
		validators  []types.Validator
		delegations []types.Delegation
	)

	valAddrs := make([]sdk.ValAddress, simState.NumBonded)

	for i := 0; i < int(simState.NumBonded); i++ {
		valAddr := sdk.ValAddress(simState.Accounts[i].Address)
		ValStrAddress, err := simState.ValidatorCodec.BytesToString(valAddr)
		if err != nil {
			panic(err)
		}
		valAddrs[i] = valAddr

		maxCommission := sdkmath.LegacyNewDecWithPrec(int64(simulation.RandIntBetween(simState.Rand, 1, 100)), 2)
		commission := types.NewCommission(
			simulation.RandomDecAmount(simState.Rand, maxCommission),
			maxCommission,
			simulation.RandomDecAmount(simState.Rand, maxCommission),
		)

		validator, err := types.NewValidator(ValStrAddress, simState.Accounts[i].ConsKey.PubKey(), types.Description{})
		if err != nil {
			panic(err)
		}
		validator.Tokens = simState.InitialStake
		validator.DelegatorShares = sdkmath.LegacyNewDecFromInt(simState.InitialStake)
		validator.Commission = commission

		accAddress, err := simState.AddressCodec.BytesToString(simState.Accounts[i].Address)
		if err != nil {
			panic(err)
		}
		delegation := types.NewDelegation(accAddress, ValStrAddress, sdkmath.LegacyNewDecFromInt(simState.InitialStake))

		validators = append(validators, validator)
		delegations = append(delegations, delegation)
	}

	stakingGenesis := types.NewGenesisState(params, validators, delegations)
	simState.GenState[types.ModuleName] = simState.Cdc.MustMarshalJSON(stakingGenesis)
}
