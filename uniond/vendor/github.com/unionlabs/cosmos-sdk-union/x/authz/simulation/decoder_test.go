package simulation_test

import (
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/authz"
	"cosmossdk.io/x/authz/keeper"
	authzmodule "cosmossdk.io/x/authz/module"
	"cosmossdk.io/x/authz/simulation"
	banktypes "cosmossdk.io/x/bank/types"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func TestDecodeStore(t *testing.T) {
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, authzmodule.AppModule{})
	banktypes.RegisterInterfaces(encCfg.InterfaceRegistry)

	dec := simulation.NewDecodeStore(encCfg.Codec)

	now := time.Now().UTC()
	e := now.Add(1)
	sendAuthz := banktypes.NewSendAuthorization(sdk.NewCoins(sdk.NewInt64Coin("foo", 123)), nil, codectestutil.CodecOptions{}.GetAddressCodec())
	grant, _ := authz.NewGrant(now, sendAuthz, &e)
	grantBz, err := encCfg.Codec.Marshal(&grant)
	require.NoError(t, err)
	kvPairs := kv.Pairs{
		Pairs: []kv.Pair{
			{Key: keeper.GrantKey, Value: grantBz},
			{Key: []byte{0x99}, Value: []byte{0x99}},
		},
	}

	tests := []struct {
		name        string
		expectErr   bool
		expectedLog string
	}{
		{"Grant", false, fmt.Sprintf("%v\n%v", grant, grant)},
		{"other", true, ""},
	}

	for i, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.expectErr {
				require.Panics(t, func() { dec(kvPairs.Pairs[i], kvPairs.Pairs[i]) }, tt.name)
			} else {
				require.Equal(t, tt.expectedLog, dec(kvPairs.Pairs[i], kvPairs.Pairs[i]), tt.name)
			}
		})
	}
}
