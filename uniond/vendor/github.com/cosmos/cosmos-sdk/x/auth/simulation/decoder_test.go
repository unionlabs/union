package simulation_test

import (
	"fmt"
	"testing"

	gogotypes "github.com/cosmos/gogoproto/types"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/x/auth/simulation"
	"github.com/cosmos/cosmos-sdk/x/auth/testutil"

	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"

	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

var (
	delPk1   = ed25519.GenPrivKey().PubKey()
	delAddr1 = sdk.AccAddress(delPk1.Address())
)

func TestDecodeStore(t *testing.T) {
	var (
		cdc           codec.Codec
		accountKeeper authkeeper.AccountKeeper
	)
	err := depinject.Inject(testutil.AppConfig, &cdc, &accountKeeper)
	require.NoError(t, err)

	acc := types.NewBaseAccountWithAddress(delAddr1)
	dec := simulation.NewDecodeStore(accountKeeper)

	accBz, err := accountKeeper.MarshalAccount(acc)
	require.NoError(t, err)

	globalAccNumber := gogotypes.UInt64Value{Value: 10}

	kvPairs := kv.Pairs{
		Pairs: []kv.Pair{
			{
				Key:   types.AddressStoreKey(delAddr1),
				Value: accBz,
			},
			{
				Key:   types.GlobalAccountNumberKey,
				Value: cdc.MustMarshal(&globalAccNumber),
			},
			{
				Key:   types.AccountNumberStoreKey(5),
				Value: acc.GetAddress().Bytes(),
			},
			{
				Key:   []byte{0x99},
				Value: []byte{0x99},
			},
		},
	}
	tests := []struct {
		name        string
		expectedLog string
	}{
		{"Account", fmt.Sprintf("%v\n%v", acc, acc)},
		{"GlobalAccNumber", fmt.Sprintf("GlobalAccNumberA: %d\nGlobalAccNumberB: %d", globalAccNumber, globalAccNumber)},
		{"AccNum", fmt.Sprintf("AccNumA: %s\nAccNumB: %s", acc.GetAddress(), acc.GetAddress())},
		{"other", ""},
	}

	for i, tt := range tests {
		i, tt := i, tt
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
