package types_test

import (
	"math/rand"
	"sort"
	"testing"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/legacy"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

var header = tmproto.Header{
	ChainID: "hello",
	Height:  5,
}

func createValidators(t *testing.T) []types.Validator {
	return []types.Validator{
		newValidator(t, valAddr1, pk1),
		newValidator(t, valAddr2, pk2),
		newValidator(t, valAddr3, pk3),
	}
}

func TestHistoricalInfo(t *testing.T) {
	validators := createValidators(t)
	hi := types.NewHistoricalInfo(header, validators, sdk.DefaultPowerReduction)
	require.True(t, sort.IsSorted(types.Validators(hi.Valset)), "Validators are not sorted")

	var value []byte
	require.NotPanics(t, func() {
		value = legacy.Cdc.MustMarshal(&hi)
	})
	require.NotNil(t, value, "Marshalled HistoricalInfo is nil")

	recv, err := types.UnmarshalHistoricalInfo(codec.NewAminoCodec(legacy.Cdc), value)
	require.Nil(t, err, "Unmarshalling HistoricalInfo failed")
	require.Equal(t, hi.Header, recv.Header)
	for i := range hi.Valset {
		require.True(t, hi.Valset[i].Equal(&recv.Valset[i]))
	}
	require.True(t, sort.IsSorted(types.Validators(hi.Valset)), "Validators are not sorted")
}

func TestValidateBasic(t *testing.T) {
	validators := createValidators(t)
	hi := types.HistoricalInfo{
		Header: header,
	}
	err := types.ValidateBasic(hi)
	require.Error(t, err, "ValidateBasic passed on nil ValSet")

	// Ensure validators are not sorted
	for sort.IsSorted(types.Validators(validators)) {
		rand.Shuffle(len(validators), func(i, j int) {
			it := validators[i]
			validators[i] = validators[j]
			validators[j] = it
		})
	}
	hi = types.HistoricalInfo{
		Header: header,
		Valset: validators,
	}
	err = types.ValidateBasic(hi)
	require.Error(t, err, "ValidateBasic passed on unsorted ValSet")

	hi = types.NewHistoricalInfo(header, validators, sdk.DefaultPowerReduction)
	err = types.ValidateBasic(hi)
	require.NoError(t, err, "ValidateBasic failed on valid HistoricalInfo")
}
