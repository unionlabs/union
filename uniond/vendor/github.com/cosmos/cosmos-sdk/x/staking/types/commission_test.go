package types_test

import (
	"testing"
	"time"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

func TestCommissionValidate(t *testing.T) {
	testCases := []struct {
		input     types.Commission
		expectErr bool
	}{
		// invalid commission; max rate < 0%
		{types.NewCommission(math.LegacyZeroDec(), sdk.MustNewDecFromStr("-1.00"), math.LegacyZeroDec()), true},
		// invalid commission; max rate > 100%
		{types.NewCommission(math.LegacyZeroDec(), sdk.MustNewDecFromStr("2.00"), math.LegacyZeroDec()), true},
		// invalid commission; rate < 0%
		{types.NewCommission(sdk.MustNewDecFromStr("-1.00"), math.LegacyZeroDec(), math.LegacyZeroDec()), true},
		// invalid commission; rate > max rate
		{types.NewCommission(sdk.MustNewDecFromStr("0.75"), sdk.MustNewDecFromStr("0.50"), math.LegacyZeroDec()), true},
		// invalid commission; max change rate < 0%
		{types.NewCommission(math.LegacyOneDec(), math.LegacyOneDec(), sdk.MustNewDecFromStr("-1.00")), true},
		// invalid commission; max change rate > max rate
		{types.NewCommission(math.LegacyOneDec(), sdk.MustNewDecFromStr("0.75"), sdk.MustNewDecFromStr("0.90")), true},
		// valid commission
		{types.NewCommission(sdk.MustNewDecFromStr("0.20"), math.LegacyOneDec(), sdk.MustNewDecFromStr("0.10")), false},
	}

	for i, tc := range testCases {
		err := tc.input.Validate()
		require.Equal(t, tc.expectErr, err != nil, "unexpected result; tc #%d, input: %v", i, tc.input)
	}
}

func TestCommissionValidateNewRate(t *testing.T) {
	now := time.Now().UTC()
	c1 := types.NewCommission(sdk.MustNewDecFromStr("0.40"), sdk.MustNewDecFromStr("0.80"), sdk.MustNewDecFromStr("0.10"))
	c1.UpdateTime = now

	testCases := []struct {
		input     types.Commission
		newRate   sdk.Dec
		blockTime time.Time
		expectErr bool
	}{
		// invalid new commission rate; last update < 24h ago
		{c1, sdk.MustNewDecFromStr("0.50"), now, true},
		// invalid new commission rate; new rate < 0%
		{c1, sdk.MustNewDecFromStr("-1.00"), now.Add(48 * time.Hour), true},
		// invalid new commission rate; new rate > max rate
		{c1, sdk.MustNewDecFromStr("0.90"), now.Add(48 * time.Hour), true},
		// invalid new commission rate; new rate > max change rate
		{c1, sdk.MustNewDecFromStr("0.60"), now.Add(48 * time.Hour), true},
		// valid commission
		{c1, sdk.MustNewDecFromStr("0.50"), now.Add(48 * time.Hour), false},
		// valid commission
		{c1, sdk.MustNewDecFromStr("0.10"), now.Add(48 * time.Hour), false},
	}

	for i, tc := range testCases {
		err := tc.input.ValidateNewRate(tc.newRate, tc.blockTime)
		require.Equal(
			t, tc.expectErr, err != nil,
			"unexpected result; tc #%d, input: %v, newRate: %s, blockTime: %s",
			i, tc.input, tc.newRate, tc.blockTime,
		)
	}
}
