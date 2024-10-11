package keeper_test

import (
	"cosmossdk.io/x/upgrade/types"
)

func (s *KeeperTestSuite) TestSoftwareUpgrade() {
	testCases := []struct {
		name      string
		req       *types.MsgSoftwareUpgrade
		expectErr bool
		errMsg    string
	}{
		{
			"invalid authority address",
			&types.MsgSoftwareUpgrade{
				Authority: "authority",
				Plan: types.Plan{
					Name:   "all-good",
					Height: 123450000,
				},
			},
			true,
			"expected authority account as only signer for proposal message",
		},
		{
			"unauthorized authority address",
			&types.MsgSoftwareUpgrade{
				Authority: s.encodedAddrs[0],
				Plan: types.Plan{
					Name:   "all-good",
					Info:   "some text here",
					Height: 123450000,
				},
			},
			true,
			"expected authority account as only signer for proposal message",
		},
		{
			"invalid plan",
			&types.MsgSoftwareUpgrade{
				Authority: s.encodedAuthority,
				Plan: types.Plan{
					Height: 123450000,
				},
			},
			true,
			"name cannot be empty: invalid request",
		},
		{
			"successful upgrade scheduled",
			&types.MsgSoftwareUpgrade{
				Authority: s.encodedAuthority,
				Plan: types.Plan{
					Name:   "all-good",
					Info:   "some text here",
					Height: 123450000,
				},
			},
			false,
			"",
		},
	}
	for _, tc := range testCases {
		s.Run(tc.name, func() {
			_, err := s.msgSrvr.SoftwareUpgrade(s.ctx, tc.req)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.errMsg)
			} else {
				s.Require().NoError(err)
				plan, err := s.upgradeKeeper.GetUpgradePlan(s.ctx)
				s.Require().NoError(err)
				s.Require().Equal(tc.req.Plan, plan)
			}
		})
	}
}

func (s *KeeperTestSuite) TestCancelUpgrade() {
	err := s.upgradeKeeper.ScheduleUpgrade(s.ctx, types.Plan{
		Name:   "some name",
		Info:   "some info",
		Height: 123450000,
	})
	s.Require().NoError(err)

	testCases := []struct {
		name      string
		req       *types.MsgCancelUpgrade
		expectErr bool
		errMsg    string
	}{
		{
			"invalid authority address",
			&types.MsgCancelUpgrade{
				Authority: "authority",
			},
			true,
			"expected authority account as only signer for proposal message",
		},
		{
			"unauthorized authority address",
			&types.MsgCancelUpgrade{
				Authority: s.encodedAddrs[0],
			},
			true,
			"expected authority account as only signer for proposal message",
		},
		{
			"upgrade canceled successfully",
			&types.MsgCancelUpgrade{
				Authority: s.encodedAuthority,
			},
			false,
			"",
		},
	}
	for _, tc := range testCases {
		s.Run(tc.name, func() {
			_, err := s.msgSrvr.CancelUpgrade(s.ctx, tc.req)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.errMsg)
			} else {
				s.Require().NoError(err)
				_, err := s.upgradeKeeper.GetUpgradePlan(s.ctx)
				s.Require().ErrorIs(err, types.ErrNoUpgradePlanFound)
			}
		})
	}
}
