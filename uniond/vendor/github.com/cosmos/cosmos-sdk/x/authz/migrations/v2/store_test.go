package v2_test

import (
	"testing"
	"time"

	"cosmossdk.io/depinject"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/authz"
	v2 "github.com/cosmos/cosmos-sdk/x/authz/migrations/v2"
	authztestutil "github.com/cosmos/cosmos-sdk/x/authz/testutil"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	"github.com/stretchr/testify/require"
)

func TestMigration(t *testing.T) {
	var cdc codec.Codec
	depinject.Inject(authztestutil.AppConfig, &cdc)

	authzKey := sdk.NewKVStoreKey("authz")
	ctx := testutil.DefaultContext(authzKey, sdk.NewTransientStoreKey("transient_test"))
	granter1 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	grantee1 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	granter2 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	grantee2 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())

	sendMsgType := banktypes.SendAuthorization{}.MsgTypeURL()
	genericMsgType := sdk.MsgTypeURL(&govtypes.MsgVote{})
	coins100 := sdk.NewCoins(sdk.NewInt64Coin("atom", 100))
	blockTime := ctx.BlockTime()
	oneDay := blockTime.AddDate(0, 0, 1)
	oneYear := blockTime.AddDate(1, 0, 0)
	sendAuthz := banktypes.NewSendAuthorization(coins100, nil)

	grants := []struct {
		granter       sdk.AccAddress
		grantee       sdk.AccAddress
		msgType       string
		authorization func() authz.Grant
	}{
		{
			granter1,
			grantee1,
			sendMsgType,
			func() authz.Grant {
				any, err := codectypes.NewAnyWithValue(sendAuthz)
				require.NoError(t, err)
				return authz.Grant{
					Authorization: any,
					Expiration:    &oneDay,
				}
			},
		},
		{
			granter1,
			grantee2,
			sendMsgType,
			func() authz.Grant {
				any, err := codectypes.NewAnyWithValue(sendAuthz)
				require.NoError(t, err)
				return authz.Grant{
					Authorization: any,
					Expiration:    &oneDay,
				}
			},
		},
		{
			granter2,
			grantee1,
			genericMsgType,
			func() authz.Grant {
				any, err := codectypes.NewAnyWithValue(authz.NewGenericAuthorization(genericMsgType))
				require.NoError(t, err)
				return authz.Grant{
					Authorization: any,
					Expiration:    &oneYear,
				}
			},
		},
		{
			granter2,
			grantee2,
			genericMsgType,
			func() authz.Grant {
				any, err := codectypes.NewAnyWithValue(authz.NewGenericAuthorization(genericMsgType))
				require.NoError(t, err)
				return authz.Grant{
					Authorization: any,
					Expiration:    &blockTime,
				}
			},
		},
	}

	store := ctx.KVStore(authzKey)

	for _, g := range grants {
		grant := g.authorization()
		store.Set(v2.GrantStoreKey(g.grantee, g.granter, g.msgType), cdc.MustMarshal(&grant))
	}

	ctx = ctx.WithBlockTime(ctx.BlockTime().Add(1 * time.Hour))
	require.NoError(t, v2.MigrateStore(ctx, authzKey, cdc))

	require.NotNil(t, store.Get(v2.GrantStoreKey(grantee1, granter2, genericMsgType)))
	require.NotNil(t, store.Get(v2.GrantStoreKey(grantee1, granter1, sendMsgType)))
	require.Nil(t, store.Get(v2.GrantStoreKey(grantee2, granter2, genericMsgType)))
}
