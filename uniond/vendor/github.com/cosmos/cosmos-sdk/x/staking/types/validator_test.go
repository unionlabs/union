package types_test

import (
	"math/rand"
	"sort"
	"testing"

	"cosmossdk.io/math"
	tmtypes "github.com/cometbft/cometbft/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/testutil"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

func TestValidatorTestEquivalent(t *testing.T) {
	val1 := newValidator(t, valAddr1, pk1)
	val2 := newValidator(t, valAddr1, pk1)
	require.Equal(t, val1.String(), val2.String())

	val2 = newValidator(t, valAddr2, pk2)
	require.NotEqual(t, val1.String(), val2.String())
}

func TestUpdateDescription(t *testing.T) {
	d1 := types.Description{
		Website: "https://validator.cosmos",
		Details: "Test validator",
	}

	d2 := types.Description{
		Moniker:  types.DoNotModifyDesc,
		Identity: types.DoNotModifyDesc,
		Website:  types.DoNotModifyDesc,
		Details:  types.DoNotModifyDesc,
	}

	d3 := types.Description{
		Moniker:  "",
		Identity: "",
		Website:  "",
		Details:  "",
	}

	d, err := d1.UpdateDescription(d2)
	require.Nil(t, err)
	require.Equal(t, d, d1)

	d, err = d1.UpdateDescription(d3)
	require.Nil(t, err)
	require.Equal(t, d, d3)
}

func TestABCIValidatorUpdate(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	abciVal := validator.ABCIValidatorUpdate(sdk.DefaultPowerReduction)
	pk, err := validator.TmConsPublicKey()
	require.NoError(t, err)
	require.Equal(t, pk, abciVal.PubKey)
	require.Equal(t, validator.BondedTokens().Int64(), abciVal.Power)
}

func TestABCIValidatorUpdateZero(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	abciVal := validator.ABCIValidatorUpdateZero()
	pk, err := validator.TmConsPublicKey()
	require.NoError(t, err)
	require.Equal(t, pk, abciVal.PubKey)
	require.Equal(t, int64(0), abciVal.Power)
}

func TestShareTokens(t *testing.T) {
	validator := mkValidator(100, math.LegacyNewDec(100))
	assert.True(sdk.DecEq(t, math.LegacyNewDec(50), validator.TokensFromShares(math.LegacyNewDec(50))))

	validator.Tokens = sdk.NewInt(50)
	assert.True(sdk.DecEq(t, math.LegacyNewDec(25), validator.TokensFromShares(math.LegacyNewDec(50))))
	assert.True(sdk.DecEq(t, math.LegacyNewDec(5), validator.TokensFromShares(math.LegacyNewDec(10))))
}

func TestRemoveTokens(t *testing.T) {
	validator := mkValidator(100, math.LegacyNewDec(100))

	// remove tokens and test check everything
	validator = validator.RemoveTokens(sdk.NewInt(10))
	require.Equal(t, int64(90), validator.Tokens.Int64())

	// update validator to from bonded -> unbonded
	validator = validator.UpdateStatus(types.Unbonded)
	require.Equal(t, types.Unbonded, validator.Status)

	validator = validator.RemoveTokens(sdk.NewInt(10))
	require.Panics(t, func() { validator.RemoveTokens(sdk.NewInt(-1)) })
	require.Panics(t, func() { validator.RemoveTokens(sdk.NewInt(100)) })
}

func TestAddTokensValidatorBonded(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	validator = validator.UpdateStatus(types.Bonded)
	validator, delShares := validator.AddTokensFromDel(sdk.NewInt(10))

	assert.True(sdk.DecEq(t, math.LegacyNewDec(10), delShares))
	assert.True(math.IntEq(t, sdk.NewInt(10), validator.BondedTokens()))
	assert.True(sdk.DecEq(t, math.LegacyNewDec(10), validator.DelegatorShares))
}

func TestAddTokensValidatorUnbonding(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	validator = validator.UpdateStatus(types.Unbonding)
	validator, delShares := validator.AddTokensFromDel(sdk.NewInt(10))

	assert.True(sdk.DecEq(t, math.LegacyNewDec(10), delShares))
	assert.Equal(t, types.Unbonding, validator.Status)
	assert.True(math.IntEq(t, sdk.NewInt(10), validator.Tokens))
	assert.True(sdk.DecEq(t, math.LegacyNewDec(10), validator.DelegatorShares))
}

func TestAddTokensValidatorUnbonded(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	validator = validator.UpdateStatus(types.Unbonded)
	validator, delShares := validator.AddTokensFromDel(sdk.NewInt(10))

	assert.True(sdk.DecEq(t, math.LegacyNewDec(10), delShares))
	assert.Equal(t, types.Unbonded, validator.Status)
	assert.True(math.IntEq(t, sdk.NewInt(10), validator.Tokens))
	assert.True(sdk.DecEq(t, math.LegacyNewDec(10), validator.DelegatorShares))
}

// TODO refactor to make simpler like the AddToken tests above
func TestRemoveDelShares(t *testing.T) {
	valA := types.Validator{
		OperatorAddress: valAddr1.String(),
		ConsensusPubkey: pk1Any,
		Status:          types.Bonded,
		Tokens:          sdk.NewInt(100),
		DelegatorShares: math.LegacyNewDec(100),
	}

	// Remove delegator shares
	valB, coinsB := valA.RemoveDelShares(math.LegacyNewDec(10))
	require.Equal(t, int64(10), coinsB.Int64())
	require.Equal(t, int64(90), valB.DelegatorShares.RoundInt64())
	require.Equal(t, int64(90), valB.BondedTokens().Int64())

	// specific case from random tests
	validator := mkValidator(5102, math.LegacyNewDec(115))
	_, tokens := validator.RemoveDelShares(math.LegacyNewDec(29))

	require.True(math.IntEq(t, sdk.NewInt(1286), tokens))
}

func TestAddTokensFromDel(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)

	validator, shares := validator.AddTokensFromDel(sdk.NewInt(6))
	require.True(sdk.DecEq(t, math.LegacyNewDec(6), shares))
	require.True(sdk.DecEq(t, math.LegacyNewDec(6), validator.DelegatorShares))
	require.True(math.IntEq(t, sdk.NewInt(6), validator.Tokens))

	validator, shares = validator.AddTokensFromDel(sdk.NewInt(3))
	require.True(sdk.DecEq(t, math.LegacyNewDec(3), shares))
	require.True(sdk.DecEq(t, math.LegacyNewDec(9), validator.DelegatorShares))
	require.True(math.IntEq(t, sdk.NewInt(9), validator.Tokens))
}

func TestUpdateStatus(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	validator, _ = validator.AddTokensFromDel(sdk.NewInt(100))
	require.Equal(t, types.Unbonded, validator.Status)
	require.Equal(t, int64(100), validator.Tokens.Int64())

	// Unbonded to Bonded
	validator = validator.UpdateStatus(types.Bonded)
	require.Equal(t, types.Bonded, validator.Status)

	// Bonded to Unbonding
	validator = validator.UpdateStatus(types.Unbonding)
	require.Equal(t, types.Unbonding, validator.Status)

	// Unbonding to Bonded
	validator = validator.UpdateStatus(types.Bonded)
	require.Equal(t, types.Bonded, validator.Status)
}

func TestPossibleOverflow(t *testing.T) {
	delShares := math.LegacyNewDec(391432570689183511).Quo(math.LegacyNewDec(40113011844664))
	validator := mkValidator(2159, delShares)
	newValidator, _ := validator.AddTokensFromDel(sdk.NewInt(71))

	require.False(t, newValidator.DelegatorShares.IsNegative())
	require.False(t, newValidator.Tokens.IsNegative())
}

func TestValidatorMarshalUnmarshalJSON(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	js, err := legacy.Cdc.MarshalJSON(validator)
	require.NoError(t, err)
	require.NotEmpty(t, js)
	require.Contains(t, string(js), "\"consensus_pubkey\":{\"type\":\"tendermint/PubKeyEd25519\"")
	got := &types.Validator{}
	err = legacy.Cdc.UnmarshalJSON(js, got)
	assert.NoError(t, err)
	assert.True(t, validator.Equal(got))
}

func TestValidatorSetInitialCommission(t *testing.T) {
	val := newValidator(t, valAddr1, pk1)
	testCases := []struct {
		validator   types.Validator
		commission  types.Commission
		expectedErr bool
	}{
		{val, types.NewCommission(math.LegacyZeroDec(), math.LegacyZeroDec(), math.LegacyZeroDec()), false},
		{val, types.NewCommission(math.LegacyZeroDec(), sdk.NewDecWithPrec(-1, 1), math.LegacyZeroDec()), true},
		{val, types.NewCommission(math.LegacyZeroDec(), math.LegacyNewDec(15000000000), math.LegacyZeroDec()), true},
		{val, types.NewCommission(sdk.NewDecWithPrec(-1, 1), math.LegacyZeroDec(), math.LegacyZeroDec()), true},
		{val, types.NewCommission(sdk.NewDecWithPrec(2, 1), sdk.NewDecWithPrec(1, 1), math.LegacyZeroDec()), true},
		{val, types.NewCommission(math.LegacyZeroDec(), math.LegacyZeroDec(), sdk.NewDecWithPrec(-1, 1)), true},
		{val, types.NewCommission(math.LegacyZeroDec(), sdk.NewDecWithPrec(1, 1), sdk.NewDecWithPrec(2, 1)), true},
	}

	for i, tc := range testCases {
		val, err := tc.validator.SetInitialCommission(tc.commission)

		if tc.expectedErr {
			require.Error(t, err,
				"expected error for test case #%d with commission: %s", i, tc.commission,
			)
		} else {
			require.NoError(t, err,
				"unexpected error for test case #%d with commission: %s", i, tc.commission,
			)
			require.Equal(t, tc.commission, val.Commission,
				"invalid validator commission for test case #%d with commission: %s", i, tc.commission,
			)
		}
	}
}

// Check that sort will create deterministic ordering of validators
func TestValidatorsSortDeterminism(t *testing.T) {
	vals := make([]types.Validator, 10)
	sortedVals := make([]types.Validator, 10)

	// Create random validator slice
	for i := range vals {
		pk := ed25519.GenPrivKey().PubKey()
		vals[i] = newValidator(t, sdk.ValAddress(pk.Address()), pk)
	}

	// Save sorted copy
	sort.Sort(types.Validators(vals))
	copy(sortedVals, vals)

	// Randomly shuffle validators, sort, and check it is equal to original sort
	for i := 0; i < 10; i++ {
		rand.Shuffle(10, func(i, j int) {
			it := vals[i]
			vals[i] = vals[j]
			vals[j] = it
		})

		types.Validators(vals).Sort()
		require.Equal(t, sortedVals, vals, "Validator sort returned different slices")
	}
}

// Check SortTendermint sorts the same as tendermint
func TestValidatorsSortTendermint(t *testing.T) {
	vals := make([]types.Validator, 100)

	for i := range vals {
		pk := ed25519.GenPrivKey().PubKey()
		pk2 := ed25519.GenPrivKey().PubKey()
		vals[i] = newValidator(t, sdk.ValAddress(pk2.Address()), pk)
		vals[i].Status = types.Bonded
		vals[i].Tokens = sdk.NewInt(rand.Int63())
	}
	// create some validators with the same power
	for i := 0; i < 10; i++ {
		vals[i].Tokens = sdk.NewInt(1000000)
	}

	valz := types.Validators(vals)

	// create expected tendermint validators by converting to tendermint then sorting
	expectedVals, err := testutil.ToTmValidators(valz, sdk.DefaultPowerReduction)
	require.NoError(t, err)
	sort.Sort(tmtypes.ValidatorsByVotingPower(expectedVals))

	// sort in SDK and then convert to tendermint
	sort.SliceStable(valz, func(i, j int) bool {
		return types.ValidatorsByVotingPower(valz).Less(i, j, sdk.DefaultPowerReduction)
	})
	actualVals, err := testutil.ToTmValidators(valz, sdk.DefaultPowerReduction)
	require.NoError(t, err)

	require.Equal(t, expectedVals, actualVals, "sorting in SDK is not the same as sorting in Tendermint")
}

func TestValidatorToTm(t *testing.T) {
	vals := make(types.Validators, 10)
	expected := make([]*tmtypes.Validator, 10)

	for i := range vals {
		pk := ed25519.GenPrivKey().PubKey()
		val := newValidator(t, sdk.ValAddress(pk.Address()), pk)
		val.Status = types.Bonded
		val.Tokens = sdk.NewInt(rand.Int63())
		vals[i] = val
		tmPk, err := cryptocodec.ToTmPubKeyInterface(pk)
		require.NoError(t, err)
		expected[i] = tmtypes.NewValidator(tmPk, val.ConsensusPower(sdk.DefaultPowerReduction))
	}
	vs, err := testutil.ToTmValidators(vals, sdk.DefaultPowerReduction)
	require.NoError(t, err)
	require.Equal(t, expected, vs)
}

func TestBondStatus(t *testing.T) {
	require.False(t, types.Unbonded == types.Bonded)
	require.False(t, types.Unbonded == types.Unbonding)
	require.False(t, types.Bonded == types.Unbonding)
	require.Equal(t, types.BondStatus(4).String(), "4")
	require.Equal(t, types.BondStatusUnspecified, types.Unspecified.String())
	require.Equal(t, types.BondStatusUnbonded, types.Unbonded.String())
	require.Equal(t, types.BondStatusBonded, types.Bonded.String())
	require.Equal(t, types.BondStatusUnbonding, types.Unbonding.String())
}

func mkValidator(tokens int64, shares sdk.Dec) types.Validator {
	return types.Validator{
		OperatorAddress: valAddr1.String(),
		ConsensusPubkey: pk1Any,
		Status:          types.Bonded,
		Tokens:          sdk.NewInt(tokens),
		DelegatorShares: shares,
	}
}

// Creates a new validators and asserts the error check.
func newValidator(t *testing.T, operator sdk.ValAddress, pubKey cryptotypes.PubKey) types.Validator {
	v, err := types.NewValidator(operator, pubKey, types.Description{})
	require.NoError(t, err)
	return v
}
