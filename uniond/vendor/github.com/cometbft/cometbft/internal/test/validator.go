package test

import (
	"context"
	"sort"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/types"
)

func Validator(_ context.Context, votingPower int64) (*types.Validator, types.PrivValidator, error) {
	privVal := types.NewMockPV()
	pubKey, err := privVal.GetPubKey()
	if err != nil {
		return nil, nil, err
	}

	val := types.NewValidator(pubKey, votingPower)
	return val, privVal, nil
}

func ValidatorSet(ctx context.Context, t *testing.T, numValidators int, votingPower int64) (*types.ValidatorSet, []types.PrivValidator) {
	var (
		valz           = make([]*types.Validator, numValidators)
		privValidators = make([]types.PrivValidator, numValidators)
	)
	t.Helper()

	for i := 0; i < numValidators; i++ {
		val, privValidator, err := Validator(ctx, votingPower)
		require.NoError(t, err)
		valz[i] = val
		privValidators[i] = privValidator
	}

	sort.Sort(types.PrivValidatorsByAddress(privValidators))

	return types.NewValidatorSet(valz), privValidators
}
