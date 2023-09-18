package v2_test

import (
	"testing"
	"time"

	"cosmossdk.io/depinject"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/feegrant"
	v2 "github.com/cosmos/cosmos-sdk/x/feegrant/migrations/v2"
	feegranttestutil "github.com/cosmos/cosmos-sdk/x/feegrant/testutil"
	"github.com/stretchr/testify/require"
)

func TestMigration(t *testing.T) {
	var cdc codec.Codec
	depinject.Inject(feegranttestutil.AppConfig, &cdc)

	feegrantKey := sdk.NewKVStoreKey(v2.ModuleName)
	ctx := testutil.DefaultContext(feegrantKey, sdk.NewTransientStoreKey("transient_test"))
	granter1 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	grantee1 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	granter2 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	grantee2 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())

	spendLimit := sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(1000)))
	now := ctx.BlockTime()
	oneDay := now.AddDate(0, 0, 1)
	twoDays := now.AddDate(0, 0, 2)

	grants := []struct {
		granter    sdk.AccAddress
		grantee    sdk.AccAddress
		spendLimit sdk.Coins
		expiration *time.Time
	}{
		{
			granter:    granter1,
			grantee:    grantee1,
			spendLimit: spendLimit,
			expiration: &twoDays,
		},
		{
			granter:    granter2,
			grantee:    grantee2,
			spendLimit: spendLimit,
			expiration: &oneDay,
		},
		{
			granter:    granter1,
			grantee:    grantee2,
			spendLimit: spendLimit,
		},
		{
			granter:    granter2,
			grantee:    grantee1,
			expiration: &oneDay,
		},
	}

	store := ctx.KVStore(feegrantKey)
	for _, grant := range grants {
		newGrant, err := feegrant.NewGrant(grant.granter, grant.grantee, &feegrant.BasicAllowance{
			SpendLimit: grant.spendLimit,
			Expiration: grant.expiration,
		})
		require.NoError(t, err)

		bz, err := cdc.Marshal(&newGrant)
		require.NoError(t, err)

		store.Set(v2.FeeAllowanceKey(grant.granter, grant.grantee), bz)
	}

	ctx = ctx.WithBlockTime(now.Add(30 * time.Hour))
	require.NoError(t, v2.MigrateStore(ctx, feegrantKey, cdc))
	store = ctx.KVStore(feegrantKey)

	require.NotNil(t, store.Get(v2.FeeAllowanceKey(granter1, grantee1)))
	require.Nil(t, store.Get(v2.FeeAllowanceKey(granter2, grantee2)))
	require.NotNil(t, store.Get(v2.FeeAllowanceKey(granter1, grantee2)))
	require.Nil(t, store.Get(v2.FeeAllowanceKey(granter2, grantee1)))
}
