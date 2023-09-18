package keeper_test

import (
	"time"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

func (s *KeeperTestSuite) TestParams() {
	ctx, keeper := s.ctx, s.slashingKeeper
	require := s.Require()

	minSignedPerWindow, err := sdk.NewDecFromStr("0.60")
	require.NoError(err)

	slashFractionDoubleSign, err := sdk.NewDecFromStr("0.022")
	require.NoError(err)

	slashFractionDowntime, err := sdk.NewDecFromStr("0.0089")
	require.NoError(err)

	invalidVal, err := sdk.NewDecFromStr("-1")
	require.NoError(err)

	testCases := []struct {
		name      string
		input     types.Params
		expectErr bool
		expErrMsg string
	}{
		{
			name: "set invalid signed blocks window",
			input: types.Params{
				SignedBlocksWindow:      0,
				MinSignedPerWindow:      minSignedPerWindow,
				DowntimeJailDuration:    time.Duration(34800000000000),
				SlashFractionDoubleSign: slashFractionDoubleSign,
				SlashFractionDowntime:   slashFractionDowntime,
			},
			expectErr: true,
			expErrMsg: "signed blocks window must be positive",
		},
		{
			name: "set invalid min signed per window",
			input: types.Params{
				SignedBlocksWindow:      int64(750),
				MinSignedPerWindow:      invalidVal,
				DowntimeJailDuration:    time.Duration(34800000000000),
				SlashFractionDoubleSign: slashFractionDoubleSign,
				SlashFractionDowntime:   slashFractionDowntime,
			},
			expectErr: true,
			expErrMsg: "min signed per window cannot be negative",
		},
		{
			name: "set invalid downtime jail duration",
			input: types.Params{
				SignedBlocksWindow:      int64(750),
				MinSignedPerWindow:      minSignedPerWindow,
				DowntimeJailDuration:    time.Duration(0),
				SlashFractionDoubleSign: slashFractionDoubleSign,
				SlashFractionDowntime:   slashFractionDowntime,
			},
			expectErr: true,
			expErrMsg: "downtime jail duration must be positive",
		},
		{
			name: "set invalid slash fraction double sign",
			input: types.Params{
				SignedBlocksWindow:      int64(750),
				MinSignedPerWindow:      minSignedPerWindow,
				DowntimeJailDuration:    time.Duration(10),
				SlashFractionDoubleSign: invalidVal,
				SlashFractionDowntime:   slashFractionDowntime,
			},
			expectErr: true,
			expErrMsg: "double sign slash fraction cannot be negative",
		},
		{
			name: "set invalid slash fraction downtime",
			input: types.Params{
				SignedBlocksWindow:      int64(750),
				MinSignedPerWindow:      minSignedPerWindow,
				DowntimeJailDuration:    time.Duration(10),
				SlashFractionDoubleSign: slashFractionDoubleSign,
				SlashFractionDowntime:   invalidVal,
			},
			expectErr: true,
			expErrMsg: "downtime slash fraction cannot be negative",
		},
		{
			name: "set all valid params",
			input: types.Params{
				SignedBlocksWindow:      int64(750),
				MinSignedPerWindow:      minSignedPerWindow,
				DowntimeJailDuration:    time.Duration(34800000000000),
				SlashFractionDoubleSign: slashFractionDoubleSign,
				SlashFractionDowntime:   slashFractionDowntime,
			},
			expectErr: false,
		},
	}
	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			expected := keeper.GetParams(ctx)
			err := keeper.SetParams(ctx, tc.input)

			if tc.expectErr {
				require.Error(err)
				require.Contains(err.Error(), tc.expErrMsg)
			} else {
				expected = tc.input
				require.NoError(err)
			}

			params := keeper.GetParams(ctx)
			require.Equal(params, expected)
			require.Equal(keeper.SignedBlocksWindow(ctx), expected.SignedBlocksWindow)
			require.Equal(keeper.MinSignedPerWindow(ctx), expected.MinSignedPerWindow.MulInt64(expected.SignedBlocksWindow).RoundInt64())
			require.Equal(keeper.DowntimeJailDuration(ctx), expected.DowntimeJailDuration)
			require.Equal(keeper.SlashFractionDoubleSign(ctx), expected.SlashFractionDoubleSign)
			require.Equal(keeper.SlashFractionDowntime(ctx), expected.SlashFractionDowntime)
		})
	}
}
