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
	"cosmossdk.io/x/feegrant"

	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestBasicFeeValidAllow(t *testing.T) {
	key := storetypes.NewKVStoreKey(feegrant.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))

	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	badTime := ctx.HeaderInfo().Time.AddDate(0, 0, -1)
	allowace := &feegrant.BasicAllowance{
		Expiration: &badTime,
	}
	require.Error(t, allowace.ValidateBasic())

	ctx = ctx.WithHeaderInfo(header.Info{Time: time.Now()})
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 10))
	atom := sdk.NewCoins(sdk.NewInt64Coin("atom", 555))
	smallAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 43))
	bigAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 1000))
	leftAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 512))
	now := ctx.HeaderInfo().Time
	oneHour := now.Add(1 * time.Hour)

	cases := map[string]struct {
		allowance *feegrant.BasicAllowance
		// all other checks are ignored if valid=false
		fee       sdk.Coins
		blockTime time.Time
		valid     bool
		accept    bool
		remove    bool
		remains   sdk.Coins
	}{
		"empty": {
			allowance: &feegrant.BasicAllowance{},
			accept:    true,
		},
		"small fee without expire": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: atom,
			},
			fee:     smallAtom,
			accept:  true,
			remove:  false,
			remains: leftAtom,
		},
		"all fee without expire": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: smallAtom,
			},
			fee:    smallAtom,
			accept: true,
			remove: true,
		},
		"wrong fee": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: smallAtom,
			},
			fee:    eth,
			accept: false,
		},
		"non-expired": {
			allowance: &feegrant.BasicAllowance{
				SpendLimit: atom,
				Expiration: &oneHour,
			},
			valid:     true,
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
			valid:     true,
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
			valid:     true,
			fee:       bigAtom,
			blockTime: now,
			accept:    false,
		},
		"with out spend limit": {
			allowance: &feegrant.BasicAllowance{
				Expiration: &oneHour,
			},
			valid:     true,
			fee:       bigAtom,
			blockTime: now,
			accept:    true,
		},
		"expired no spend limit": {
			allowance: &feegrant.BasicAllowance{
				Expiration: &now,
			},
			valid:     true,
			fee:       bigAtom,
			blockTime: oneHour,
			accept:    false,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.allowance.UpdatePeriodReset(tc.blockTime)
			require.NoError(t, err)

			err = tc.allowance.ValidateBasic()
			require.NoError(t, err)

			ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: tc.blockTime})
			// now try to deduct
			removed, err := tc.allowance.Accept(context.WithValue(ctx, corecontext.EnvironmentContextKey, appmodulev2.Environment{
				HeaderService: mockHeaderService{},
				GasService:    mockGasService{},
			}), tc.fee, []sdk.Msg{})
			if !tc.accept {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)

			require.Equal(t, tc.remove, removed)
			if !removed {
				assert.Equal(t, tc.allowance.SpendLimit, tc.remains)
			}
		})
	}
}
