package simulation_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/x/feegrant"
	"github.com/cosmos/cosmos-sdk/x/feegrant/simulation"
	feegranttestutil "github.com/cosmos/cosmos-sdk/x/feegrant/testutil"
)

var (
	granterPk   = ed25519.GenPrivKey().PubKey()
	granterAddr = sdk.AccAddress(granterPk.Address())
	granteeAddr = sdk.AccAddress(granterPk.Address())
)

func TestDecodeStore(t *testing.T) {
	var cdc codec.Codec
	depinject.Inject(feegranttestutil.AppConfig, &cdc)
	dec := simulation.NewDecodeStore(cdc)

	grant, err := feegrant.NewGrant(granterAddr, granteeAddr, &feegrant.BasicAllowance{
		SpendLimit: sdk.NewCoins(sdk.NewCoin("foo", sdk.NewInt(100))),
	})

	require.NoError(t, err)

	grantBz, err := cdc.Marshal(&grant)
	require.NoError(t, err)

	kvPairs := kv.Pairs{
		Pairs: []kv.Pair{
			{Key: []byte(feegrant.FeeAllowanceKeyPrefix), Value: grantBz},
			{Key: []byte{0x99}, Value: []byte{0x99}},
		},
	}

	tests := []struct {
		name        string
		expectedLog string
	}{
		{"Grant", fmt.Sprintf("%v\n%v", grant, grant)},
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
