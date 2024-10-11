package types_test

import (
	"math/rand"
	"sort"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/codec/legacy"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
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

func TestShareTokens(t *testing.T) {
	validator := mkValidator(100, math.LegacyNewDec(100))
	assert.True(math.LegacyDecEq(t, math.LegacyNewDec(50), validator.TokensFromShares(math.LegacyNewDec(50))))

	validator.Tokens = math.NewInt(50)
	assert.True(math.LegacyDecEq(t, math.LegacyNewDec(25), validator.TokensFromShares(math.LegacyNewDec(50))))
	assert.True(math.LegacyDecEq(t, math.LegacyNewDec(5), validator.TokensFromShares(math.LegacyNewDec(10))))
}

func TestRemoveTokens(t *testing.T) {
	validator := mkValidator(100, math.LegacyNewDec(100))

	// remove tokens and test check everything
	validator = validator.RemoveTokens(math.NewInt(10))
	require.Equal(t, int64(90), validator.Tokens.Int64())

	// update validator to from bonded -> unbonded
	validator = validator.UpdateStatus(types.Unbonded)
	require.Equal(t, types.Unbonded, validator.Status)

	validator = validator.RemoveTokens(math.NewInt(10))
	require.Panics(t, func() { validator.RemoveTokens(math.NewInt(-1)) })
	require.Panics(t, func() { validator.RemoveTokens(math.NewInt(100)) })
}

func TestAddTokensValidatorBonded(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	validator = validator.UpdateStatus(types.Bonded)
	validator, delShares := validator.AddTokensFromDel(math.NewInt(10))

	assert.True(math.LegacyDecEq(t, math.LegacyNewDec(10), delShares))
	assert.True(math.IntEq(t, math.NewInt(10), validator.BondedTokens()))
	assert.True(math.LegacyDecEq(t, math.LegacyNewDec(10), validator.DelegatorShares))
}

func TestAddTokensValidatorUnbonding(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	validator = validator.UpdateStatus(types.Unbonding)
	validator, delShares := validator.AddTokensFromDel(math.NewInt(10))

	assert.True(math.LegacyDecEq(t, math.LegacyNewDec(10), delShares))
	assert.Equal(t, types.Unbonding, validator.Status)
	assert.True(math.IntEq(t, math.NewInt(10), validator.Tokens))
	assert.True(math.LegacyDecEq(t, math.LegacyNewDec(10), validator.DelegatorShares))
}

func TestAddTokensValidatorUnbonded(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	validator = validator.UpdateStatus(types.Unbonded)
	validator, delShares := validator.AddTokensFromDel(math.NewInt(10))

	assert.True(math.LegacyDecEq(t, math.LegacyNewDec(10), delShares))
	assert.Equal(t, types.Unbonded, validator.Status)
	assert.True(math.IntEq(t, math.NewInt(10), validator.Tokens))
	assert.True(math.LegacyDecEq(t, math.LegacyNewDec(10), validator.DelegatorShares))
}

// TODO refactor to make simpler like the AddToken tests above
func TestRemoveDelShares(t *testing.T) {
	addr1, err := codectestutil.CodecOptions{}.GetValidatorCodec().BytesToString(valAddr1)
	require.NoError(t, err)
	valA := types.Validator{
		OperatorAddress: addr1,
		ConsensusPubkey: pk1Any,
		Status:          types.Bonded,
		Tokens:          math.NewInt(100),
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

	require.True(math.IntEq(t, math.NewInt(1286), tokens))
}

func TestAddTokensFromDel(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)

	validator, shares := validator.AddTokensFromDel(math.NewInt(6))
	require.True(math.LegacyDecEq(t, math.LegacyNewDec(6), shares))
	require.True(math.LegacyDecEq(t, math.LegacyNewDec(6), validator.DelegatorShares))
	require.True(math.IntEq(t, math.NewInt(6), validator.Tokens))

	validator, shares = validator.AddTokensFromDel(math.NewInt(3))
	require.True(math.LegacyDecEq(t, math.LegacyNewDec(3), shares))
	require.True(math.LegacyDecEq(t, math.LegacyNewDec(9), validator.DelegatorShares))
	require.True(math.IntEq(t, math.NewInt(9), validator.Tokens))
}

func TestUpdateStatus(t *testing.T) {
	validator := newValidator(t, valAddr1, pk1)
	validator, _ = validator.AddTokensFromDel(math.NewInt(100))
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
	newValidator, _ := validator.AddTokensFromDel(math.NewInt(71))

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
		{val, types.NewCommission(math.LegacyZeroDec(), math.LegacyNewDecWithPrec(-1, 1), math.LegacyZeroDec()), true},
		{val, types.NewCommission(math.LegacyZeroDec(), math.LegacyNewDec(15000000000), math.LegacyZeroDec()), true},
		{val, types.NewCommission(math.LegacyNewDecWithPrec(-1, 1), math.LegacyZeroDec(), math.LegacyZeroDec()), true},
		{val, types.NewCommission(math.LegacyNewDecWithPrec(2, 1), math.LegacyNewDecWithPrec(1, 1), math.LegacyZeroDec()), true},
		{val, types.NewCommission(math.LegacyZeroDec(), math.LegacyZeroDec(), math.LegacyNewDecWithPrec(-1, 1)), true},
		{val, types.NewCommission(math.LegacyZeroDec(), math.LegacyNewDecWithPrec(1, 1), math.LegacyNewDecWithPrec(2, 1)), true},
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
	sort.Sort(types.Validators{Validators: vals, ValidatorCodec: address.NewBech32Codec("cosmosvaloper")})
	copy(sortedVals, vals)

	// Randomly shuffle validators, sort, and check it is equal to original sort
	for i := 0; i < 10; i++ {
		rand.Shuffle(10, func(i, j int) {
			vals[i], vals[j] = vals[j], vals[i]
		})

		types.Validators{Validators: vals, ValidatorCodec: address.NewBech32Codec("cosmosvaloper")}.Sort()
		require.Equal(t, sortedVals, vals, "Validator sort returned different slices")
	}
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

func mkValidator(tokens int64, shares math.LegacyDec) types.Validator {
	vAddr1, _ := codectestutil.CodecOptions{}.GetValidatorCodec().BytesToString(valAddr1)
	return types.Validator{
		OperatorAddress: vAddr1,
		ConsensusPubkey: pk1Any,
		Status:          types.Bonded,
		Tokens:          math.NewInt(tokens),
		DelegatorShares: shares,
	}
}

// Creates a new validators and asserts the error check.
func newValidator(t *testing.T, operator sdk.ValAddress, pubKey cryptotypes.PubKey) types.Validator {
	t.Helper()
	addr, err := codectestutil.CodecOptions{}.GetValidatorCodec().BytesToString(operator)
	require.NoError(t, err)
	v, err := types.NewValidator(addr, pubKey, types.Description{})
	require.NoError(t, err)
	return v
}
