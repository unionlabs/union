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

func TestPeriodicFeeValidAllow(t *testing.T) {
	key := storetypes.NewKVStoreKey(feegrant.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))

	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now()})

	atom := sdk.NewCoins(sdk.NewInt64Coin("atom", 555))
	smallAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 43))
	leftAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 512))
	oneAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 1))
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 1))
	emptyCoins := sdk.Coins{}

	now := ctx.HeaderInfo().Time
	oneHour := now.Add(1 * time.Hour)
	twoHours := now.Add(2 * time.Hour)
	tenMinutes := time.Duration(10) * time.Minute

	cases := map[string]struct {
		allow             feegrant.PeriodicAllowance
		fee               sdk.Coins
		blockTime         time.Time
		valid             bool // all other checks are ignored if valid=false
		accept            bool
		remove            bool
		remains           sdk.Coins
		remainsPeriod     sdk.Coins
		periodReset       time.Time
		updatePeriodReset bool
	}{
		"empty": {
			allow: feegrant.PeriodicAllowance{},
			valid: false,
		},
		"only basic": {
			allow: feegrant.PeriodicAllowance{
				Basic: feegrant.BasicAllowance{
					SpendLimit: atom,
					Expiration: &oneHour,
				},
			},
			valid: false,
		},
		"empty basic": {
			allow: feegrant.PeriodicAllowance{
				Period:           tenMinutes,
				PeriodSpendLimit: smallAtom,
				PeriodReset:      now.Add(30 * time.Minute),
			},
			blockTime:     now,
			valid:         true,
			accept:        true,
			remove:        false,
			remainsPeriod: emptyCoins,
			periodReset:   now.Add(30 * time.Minute),
		},
		"mismatched currencies": {
			allow: feegrant.PeriodicAllowance{
				Basic: feegrant.BasicAllowance{
					SpendLimit: atom,
					Expiration: &oneHour,
				},
				Period:           tenMinutes,
				PeriodSpendLimit: eth,
			},
			valid: false,
		},
		"same period": {
			allow: feegrant.PeriodicAllowance{
				Basic: feegrant.BasicAllowance{
					SpendLimit: atom,
					Expiration: &twoHours,
				},
				Period:           tenMinutes,
				PeriodReset:      now.Add(1 * time.Hour),
				PeriodSpendLimit: leftAtom,
				PeriodCanSpend:   smallAtom,
			},
			valid:         true,
			fee:           smallAtom,
			blockTime:     now,
			accept:        true,
			remove:        false,
			remainsPeriod: emptyCoins,
			remains:       leftAtom,
			periodReset:   now.Add(1 * time.Hour),
		},
		"step one period": {
			allow: feegrant.PeriodicAllowance{
				Basic: feegrant.BasicAllowance{
					SpendLimit: atom,
					Expiration: &twoHours,
				},
				Period:           tenMinutes,
				PeriodReset:      now,
				PeriodSpendLimit: leftAtom,
			},
			valid:         true,
			fee:           leftAtom,
			blockTime:     now.Add(1 * time.Hour),
			accept:        true,
			remove:        false,
			remainsPeriod: emptyCoins,
			remains:       smallAtom,
			periodReset:   oneHour.Add(tenMinutes), // one step from last reset, not now
		},
		"step limited by global allowance": {
			allow: feegrant.PeriodicAllowance{
				Basic: feegrant.BasicAllowance{
					SpendLimit: smallAtom,
					Expiration: &twoHours,
				},
				Period:           tenMinutes,
				PeriodReset:      now,
				PeriodSpendLimit: atom,
			},
			valid:         true,
			fee:           oneAtom,
			blockTime:     oneHour,
			accept:        true,
			remove:        false,
			remainsPeriod: smallAtom.Sub(oneAtom...),
			remains:       smallAtom.Sub(oneAtom...),
			periodReset:   oneHour.Add(tenMinutes), // one step from last reset, not now
		},
		"period reset no spend limit": {
			allow: feegrant.PeriodicAllowance{
				Period:           tenMinutes,
				PeriodReset:      now,
				PeriodSpendLimit: atom,
			},
			valid:         true,
			fee:           atom,
			blockTime:     oneHour,
			accept:        true,
			remove:        false,
			remainsPeriod: emptyCoins,
			periodReset:   oneHour.Add(tenMinutes), // one step from last reset, not now
		},
		"expired": {
			allow: feegrant.PeriodicAllowance{
				Basic: feegrant.BasicAllowance{
					SpendLimit: atom,
					Expiration: &now,
				},
				Period:           time.Hour,
				PeriodSpendLimit: smallAtom,
			},
			valid:     true,
			fee:       smallAtom,
			blockTime: oneHour,
			accept:    false,
			remove:    true,
		},
		"over period limit": {
			allow: feegrant.PeriodicAllowance{
				Basic: feegrant.BasicAllowance{
					SpendLimit: atom,
					Expiration: &now,
				},
				Period:           time.Hour,
				PeriodReset:      now.Add(1 * time.Hour),
				PeriodSpendLimit: leftAtom,
				PeriodCanSpend:   smallAtom,
			},
			valid:     true,
			fee:       leftAtom,
			blockTime: now,
			accept:    false,
			remove:    true,
		},
		"test update PeriodReset ": {
			allow: feegrant.PeriodicAllowance{
				Period:           tenMinutes,
				PeriodSpendLimit: smallAtom,
				PeriodReset:      now.Add(30 * time.Minute),
			},
			blockTime:         now,
			valid:             true,
			accept:            true,
			remove:            false,
			remainsPeriod:     emptyCoins,
			periodReset:       now.Add(10 * time.Minute),
			updatePeriodReset: true,
		},
	}

	for name, stc := range cases {
		tc := stc // to make scopelint happy
		t.Run(name, func(t *testing.T) {
			err := tc.allow.ValidateBasic()
			if !tc.valid {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)

			if tc.updatePeriodReset {
				err = tc.allow.UpdatePeriodReset(tc.blockTime)
				require.NoError(t, err)
			}

			ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: tc.blockTime})
			// now try to deduct
			// Set environment to ctx
			remove, err := tc.allow.Accept(context.WithValue(ctx, corecontext.EnvironmentContextKey, appmodulev2.Environment{
				HeaderService: mockHeaderService{},
				GasService:    mockGasService{},
			}), tc.fee, []sdk.Msg{})
			if !tc.accept {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)

			require.Equal(t, tc.remove, remove)
			if !remove {
				assert.Equal(t, tc.remains, tc.allow.Basic.SpendLimit)
				assert.Equal(t, tc.remainsPeriod, tc.allow.PeriodCanSpend)
				assert.Equal(t, tc.periodReset.String(), tc.allow.PeriodReset.String())
			}
		})
	}
}
