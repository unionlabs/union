package test

import (
	"context"
	"fmt"
	"sort"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto/ed25519"
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
	t.Helper()

	var (
		valz           = make([]*types.Validator, numValidators)
		privValidators = make([]types.PrivValidator, numValidators)
	)

	for i := 0; i < numValidators; i++ {
		val, privValidator, err := Validator(ctx, votingPower)
		require.NoError(t, err)
		valz[i] = val
		privValidators[i] = privValidator
	}

	sort.Sort(types.PrivValidatorsByAddress(privValidators))

	return types.NewValidatorSet(valz), privValidators
}

func GenesisValidatorSet(nVals int) ([]types.GenesisValidator, map[string]types.PrivValidator) {
	vals := make([]types.GenesisValidator, nVals)
	privVals := make(map[string]types.PrivValidator, nVals)
	for i := 0; i < nVals; i++ {
		secret := []byte(fmt.Sprintf("test%d", i))
		pk := ed25519.GenPrivKeyFromSecret(secret)
		valAddr := pk.PubKey().Address()
		vals[i] = types.GenesisValidator{
			Address: valAddr,
			PubKey:  pk.PubKey(),
			Power:   1000,
			Name:    fmt.Sprintf("test%d", i),
		}
		privVals[valAddr.String()] = types.NewMockPVWithParams(pk, false, false)
	}

	return vals, privVals
}
