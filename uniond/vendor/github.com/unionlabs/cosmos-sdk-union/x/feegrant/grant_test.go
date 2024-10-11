package feegrant_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/header"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/module"

	codecaddress "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func TestGrant(t *testing.T) {
	addressCodec := codecaddress.NewBech32Codec("cosmos")
	key := storetypes.NewKVStoreKey(feegrant.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{})

	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now()})

	addr, err := addressCodec.StringToBytes("cosmos1qk93t4j0yyzgqgt6k5qf8deh8fq6smpn3ntu3x")
	require.NoError(t, err)
	addr2, err := addressCodec.StringToBytes("cosmos1p9qh4ldfd6n0qehujsal4k7g0e37kel90rc4ts")
	require.NoError(t, err)
	atom := sdk.NewCoins(sdk.NewInt64Coin("atom", 555))
	now := ctx.HeaderInfo().Time
	oneYear := now.AddDate(1, 0, 0)

	zeroAtoms := sdk.NewCoins(sdk.NewInt64Coin("atom", 0))

	cases := map[string]struct {
		granter sdk.AccAddress
		grantee sdk.AccAddress
		limit   sdk.Coins
		expires time.Time
		valid   bool
	}{
		"good": {
			granter: addr2,
			grantee: addr,
			limit:   atom,
			expires: oneYear,
			valid:   true,
		},
		"no grantee": {
			granter: addr2,
			grantee: nil,
			limit:   atom,
			expires: oneYear,
			valid:   false,
		},
		"no granter": {
			granter: nil,
			grantee: addr,
			limit:   atom,
			expires: oneYear,
			valid:   false,
		},
		"self-grant": {
			granter: addr2,
			grantee: addr2,
			limit:   atom,
			expires: oneYear,
			valid:   false,
		},
		"zero allowance": {
			granter: addr2,
			grantee: addr,
			limit:   zeroAtoms,
			expires: oneYear,
			valid:   false,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			granterStr, err := addressCodec.BytesToString(tc.granter)
			require.NoError(t, err)
			granteeStr, err := addressCodec.BytesToString(tc.grantee)
			require.NoError(t, err)
			grant, err := feegrant.NewGrant(granterStr, granteeStr, &feegrant.BasicAllowance{
				SpendLimit: tc.limit,
				Expiration: &tc.expires,
			})
			require.NoError(t, err)
			err = grant.ValidateBasic()

			if !tc.valid {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)

			// if it is valid, let's try to serialize, deserialize, and make sure it matches
			bz, err := encCfg.Codec.Marshal(&grant)
			require.NoError(t, err)
			var loaded feegrant.Grant
			err = encCfg.Codec.Unmarshal(bz, &loaded)
			require.NoError(t, err)

			err = loaded.ValidateBasic()
			require.NoError(t, err)

			require.Equal(t, grant, loaded)
		})
	}
}
