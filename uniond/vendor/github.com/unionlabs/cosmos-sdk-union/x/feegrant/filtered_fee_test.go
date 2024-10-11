package feegrant_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	appmodulev2 "cosmossdk.io/core/appmodule/v2"
	corecontext "cosmossdk.io/core/context"
	"cosmossdk.io/core/header"
	storetypes "cosmossdk.io/store/types"
	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/module"

	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func TestFilteredFeeValidAllow(t *testing.T) {
	key := storetypes.NewKVStoreKey(feegrant.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{})

	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now()})

	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 10))
	atom := sdk.NewCoins(sdk.NewInt64Coin("atom", 555))
	smallAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 43))
	bigAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 1000))
	leftAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 512))
	now := ctx.HeaderInfo().Time
	oneHour := now.Add(1 * time.Hour)

	ac := addresscodec.NewBech32Codec("cosmos")

	// msg we will call in the all cases
	call := banktypes.MsgSend{}
	cases := map[string]struct {
		allowance *feegrant.BasicAllowance
		msgs      []string
		fee       sdk.Coins
		blockTime time.Time
		accept    bool
		remove    bool
		remains   sdk.Coins
	}{
		"msg contained": {
			allowance: &feegrant.BasicAllowance{},
			msgs:      []string{sdk.MsgTypeURL(&call)},
			accept:    true,
		},
		"msg not contained": {
			allowance: &feegrant.BasicAllowance{},
			msgs:      []string{"/cosmos.gov.v1.MsgVote"},
			accept:    false,
		},
		"small fee without expire": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: atom,
			},
			msgs:    []string{sdk.MsgTypeURL(&call)},
			fee:     smallAtom,
			accept:  true,
			remove:  false,
			remains: leftAtom,
		},
		"all fee without expire": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: smallAtom,
			},
			msgs:   []string{sdk.MsgTypeURL(&call)},
			fee:    smallAtom,
			accept: true,
			remove: true,
		},
		"wrong fee": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: smallAtom,
			},
			msgs:   []string{sdk.MsgTypeURL(&call)},
			fee:    eth,
			accept: false,
		},
		"non-expired": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: atom,
				Expiration: &oneHour,
			},
			msgs:      []string{sdk.MsgTypeURL(&call)},
			fee:       smallAtom,
			blockTime: now,
			accept:    true,
			remove:    false,
			remains:   leftAtom,
		},
		"expired": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: atom,
				Expiration: &now,
			},
			msgs:      []string{sdk.MsgTypeURL(&call)},
			fee:       smallAtom,
			blockTime: oneHour,
			accept:    false,
			remove:    true,
		},
		"fee more than allowed": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: atom,
				Expiration: &oneHour,
			},
			msgs:      []string{sdk.MsgTypeURL(&call)},
			fee:       bigAtom,
			blockTime: now,
			accept:    false,
		},
		"with out spend limit": {
			allowance: &feegrant.BasicAllowance{
				Expiration: &oneHour,
			},
			msgs:      []string{sdk.MsgTypeURL(&call)},
			fee:       bigAtom,
			blockTime: now,
			accept:    true,
		},
		"expired no spend limit": {
			allowance: &feegrant.BasicAllowance{
				Expiration: &now,
			},
			msgs:      []string{sdk.MsgTypeURL(&call)},
			fee:       bigAtom,
			blockTime: oneHour,
			accept:    false,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.allowance.ValidateBasic()
			require.NoError(t, err)

			ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: tc.blockTime})

			// create grant
			granter, grantee := sdk.AccAddress("granter"), sdk.AccAddress("grantee")
			allowance, err := feegrant.NewAllowedMsgAllowance(tc.allowance, tc.msgs)
			require.NoError(t, err)
			granterStr, err := ac.BytesToString(granter)
			require.NoError(t, err)
			granteeStr, err := ac.BytesToString(grantee)
			require.NoError(t, err)

			// now try to deduct
			removed, err := allowance.Accept(context.WithValue(ctx, corecontext.EnvironmentContextKey, appmodulev2.Environment{
				HeaderService: mockHeaderService{},
				GasService:    mockGasService{},
			}), tc.fee, []sdk.Msg{&call})
			if !tc.accept {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)

			require.Equal(t, tc.remove, removed)
			if !removed {
				// mimic save & load process (#10564)
				// the cached allowance was correct even before the fix,
				// however, the saved value was not.
				// so we need this to catch the bug.

				// create a new updated grant
				newGrant, err := feegrant.NewGrant(
					granterStr,
					granteeStr,
					allowance)
				require.NoError(t, err)

				// save the grant
				bz, err := encCfg.Codec.Marshal(&newGrant)
				require.NoError(t, err)

				// load the grant
				var loadedGrant feegrant.Grant
				err = encCfg.Codec.Unmarshal(bz, &loadedGrant)
				require.NoError(t, err)

				newAllowance, err := loadedGrant.GetGrant()
				require.NoError(t, err)
				feeAllowance, err := newAllowance.(*feegrant.AllowedMsgAllowance).GetAllowance()
				require.NoError(t, err)
				assert.Equal(t, tc.remains, feeAllowance.(*feegrant.BasicAllowance).SpendLimit)
			}
		})
	}
}
