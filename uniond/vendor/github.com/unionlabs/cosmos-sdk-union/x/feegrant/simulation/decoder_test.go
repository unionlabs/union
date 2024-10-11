package simulation_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/module"
	"cosmossdk.io/x/feegrant/simulation"

	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

var (
	granterPk   = ed25519.GenPrivKey().PubKey()
	granterAddr = sdk.AccAddress(granterPk.Address())
	granteeAddr = sdk.AccAddress(granterPk.Address())
)

func TestDecodeStore(t *testing.T) {
	encodingConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{})
	cdc := encodingConfig.Codec
	dec := simulation.NewDecodeStore(cdc)
	ac := addresscodec.NewBech32Codec("cosmos")

	granterStr, err := ac.BytesToString(granterAddr)
	require.NoError(t, err)
	granteeStr, err := ac.BytesToString(granteeAddr)
	require.NoError(t, err)

	grant, err := feegrant.NewGrant(granterStr, granteeStr, &feegrant.BasicAllowance{
		SpendLimit: sdk.NewCoins(sdk.NewCoin("foo", sdkmath.NewInt(100))),
	})

	require.NoError(t, err)

	grantBz, err := cdc.Marshal(&grant)
	require.NoError(t, err)

	kvPairs := kv.Pairs{
		Pairs: []kv.Pair{
			{Key: feegrant.FeeAllowanceKeyPrefix, Value: grantBz},
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
