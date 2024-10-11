package simulation_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/simulation"
	"cosmossdk.io/x/staking/types"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
)

var (
	delPk1   = ed25519.GenPrivKey().PubKey()
	valAddr1 = sdk.ValAddress(delPk1.Address())
)

func TestDecodeStore(t *testing.T) {
	cdc := testutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	dec := simulation.NewDecodeStore(cdc)

	oneIntBz, err := math.OneInt().Marshal()
	require.NoError(t, err)

	kvPairs := kv.Pairs{
		Pairs: []kv.Pair{
			{Key: types.LastTotalPowerKey, Value: oneIntBz},
			{Key: types.LastValidatorPowerKey, Value: valAddr1.Bytes()},
			{Key: []byte{0x99}, Value: []byte{0x99}},
		},
	}

	tests := []struct {
		name        string
		expectedLog string
	}{
		{"LastTotalPower", fmt.Sprintf("%v\n%v", math.OneInt(), math.OneInt())},
		{"LastValidatorPower/ValidatorsByConsAddr/ValidatorsByPowerIndex", fmt.Sprintf("%v\n%v", valAddr1, valAddr1)},
		{"other", ""},
	}
	for i, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			switch i {
			case len(tests) - 1:
				require.Panics(t, func() { dec(kvPairs.Pairs[i], kvPairs.Pairs[i]) }, tt.name)
			default:
				require.Equal(t, tt.expectedLog, dec(kvPairs.Pairs[i], kvPairs.Pairs[i]), tt.name)
			}
		})
	}
}
