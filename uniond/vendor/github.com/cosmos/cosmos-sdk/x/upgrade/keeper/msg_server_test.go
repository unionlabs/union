package keeper_test

import (
	"github.com/cosmos/cosmos-sdk/x/upgrade/types"
)

func (s *KeeperTestSuite) TestSoftwareUpgrade() {
	govAccAddr := "cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn" // TODO
	// govAccAddr := s.govKeeper.GetGovernanceAccount(s.ctx).GetAddress().String()

	testCases := []struct {
		name      string
		req       *types.MsgSoftwareUpgrade
		expectErr bool
		errMsg    string
	}{
		{
			"unauthorized authority address",
			&types.MsgSoftwareUpgrade{
				Authority: s.addrs[0].String(),
				Plan: types.Plan{
					Name:   "all-good",
					Info:   "some text here",
					Height: 123450000,
				},
			},
			true,
			"expected gov account as only signer for proposal message",
		},
		{
			"invalid plan",
			&types.MsgSoftwareUpgrade{
				Authority: govAccAddr,
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
				Authority: govAccAddr,
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
				plan, found := s.upgradeKeeper.GetUpgradePlan(s.ctx)
				s.Require().Equal(true, found)
				s.Require().Equal(tc.req.Plan, plan)
			}
		})
	}
}

func (s *KeeperTestSuite) TestCancelUpgrade() {
	govAccAddr := "cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn" // TODO
	// govAccAddr := s.govKeeper.GetGovernanceAccount(s.ctx).GetAddress().String()
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
			"unauthorized authority address",
			&types.MsgCancelUpgrade{
				Authority: s.addrs[0].String(),
			},
			true,
			"expected gov account as only signer for proposal message",
		},
		{
			"upgrade cancelled successfully",
			&types.MsgCancelUpgrade{
				Authority: govAccAddr,
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
				_, found := s.upgradeKeeper.GetUpgradePlan(s.ctx)
				s.Require().Equal(false, found)
			}
		})
	}
}
