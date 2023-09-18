package keeper_test

import (
	"time"

	"cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/testutil"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

func (s *KeeperTestSuite) TestIncrementUnbondingID() {
	for i := 1; i < 10; i++ {
		s.Require().Equal(uint64(i), s.stakingKeeper.IncrementUnbondingID(s.ctx))
	}
}

func (s *KeeperTestSuite) TestUnbondingTypeAccessors() {
	cases := []struct {
		exists   bool
		name     string
		expected types.UnbondingType
	}{
		{
			name:     "existing 1",
			exists:   true,
			expected: types.UnbondingType_UnbondingDelegation,
		},
		{
			name:     "existing 2",
			exists:   true,
			expected: types.UnbondingType_Redelegation,
		},
		{
			name:   "not existing",
			exists: false,
		},
	}

	for i, tc := range cases {
		s.Run(tc.name, func() {
			if tc.exists {
				s.stakingKeeper.SetUnbondingType(s.ctx, uint64(i), tc.expected)
			}

			unbondingType, found := s.stakingKeeper.GetUnbondingType(s.ctx, uint64(i))
			if tc.exists {
				s.Require().True(found)
				s.Require().Equal(tc.expected, unbondingType)
			} else {
				s.Require().False(found)
			}
		})
	}
}

func (s *KeeperTestSuite) TestUnbondingDelegationByUnbondingIDAccessors() {
	delAddrs, valAddrs := createValAddrs(2)

	type exists struct {
		setUnbondingDelegation              bool
		setUnbondingDelegationByUnbondingID bool
	}

	cases := []struct {
		exists   exists
		name     string
		expected types.UnbondingDelegation
	}{
		{
			name:   "existing 1",
			exists: exists{true, true},
			expected: types.NewUnbondingDelegation(
				delAddrs[0],
				valAddrs[0],
				0,
				time.Unix(0, 0).UTC(),
				sdk.NewInt(5),
				0,
			),
		},
		{
			name:   "not existing 1",
			exists: exists{false, true},
			expected: types.NewUnbondingDelegation(
				delAddrs[1],
				valAddrs[1],
				0,
				time.Unix(0, 0).UTC(),
				sdk.NewInt(5),
				0,
			),
		},
		{
			name:   "not existing 2",
			exists: exists{false, false},
			expected: types.NewUnbondingDelegation(
				delAddrs[0],
				valAddrs[0],
				0,
				time.Unix(0, 0).UTC(),
				sdk.NewInt(5),
				0,
			),
		},
	}

	for i, tc := range cases {
		s.Run(tc.name, func() {
			if tc.exists.setUnbondingDelegation {
				s.stakingKeeper.SetUnbondingDelegation(s.ctx, tc.expected)
			}

			if tc.exists.setUnbondingDelegationByUnbondingID {
				s.stakingKeeper.SetUnbondingDelegationByUnbondingID(s.ctx, tc.expected, uint64(i))
			}

			ubd, found := s.stakingKeeper.GetUnbondingDelegationByUnbondingID(s.ctx, uint64(i))
			if tc.exists.setUnbondingDelegation && tc.exists.setUnbondingDelegationByUnbondingID {
				s.Require().True(found)
				s.Require().Equal(tc.expected, ubd)
			} else {
				s.Require().False(found)
			}
		})
	}
}

func (s *KeeperTestSuite) TestRedelegationByUnbondingIDAccessors() {
	delAddrs, valAddrs := createValAddrs(2)

	type exists struct {
		setRedelegation              bool
		setRedelegationByUnbondingID bool
	}

	cases := []struct {
		exists   exists
		name     string
		expected types.Redelegation
	}{
		{
			name:   "existing 1",
			exists: exists{true, true},
			expected: types.NewRedelegation(
				delAddrs[0],
				valAddrs[0],
				valAddrs[1],
				0,
				time.Unix(5, 0).UTC(),
				sdk.NewInt(10),
				math.LegacyNewDec(10),
				0,
			),
		},
		{
			name:   "not existing 1",
			exists: exists{false, true},
			expected: types.NewRedelegation(
				delAddrs[1],
				valAddrs[0],
				valAddrs[1],
				0,
				time.Unix(5, 0).UTC(),
				sdk.NewInt(10),
				math.LegacyNewDec(10),
				0,
			),
		},
		{
			name:   "not existing 2",
			exists: exists{false, false},
			expected: types.NewRedelegation(
				delAddrs[1],
				valAddrs[1],
				valAddrs[0],
				0,
				time.Unix(5, 0).UTC(),
				sdk.NewInt(10),
				math.LegacyNewDec(10),
				0,
			),
		},
	}

	for i, tc := range cases {
		s.Run(tc.name, func() {
			if tc.exists.setRedelegation {
				s.stakingKeeper.SetRedelegation(s.ctx, tc.expected)
			}

			if tc.exists.setRedelegationByUnbondingID {
				s.stakingKeeper.SetRedelegationByUnbondingID(s.ctx, tc.expected, uint64(i))
			}

			red, found := s.stakingKeeper.GetRedelegationByUnbondingID(s.ctx, uint64(i))
			if tc.exists.setRedelegation && tc.exists.setRedelegationByUnbondingID {
				s.Require().True(found)
				s.Require().Equal(tc.expected, red)
			} else {
				s.Require().False(found)
			}
		})
	}
}

func (s *KeeperTestSuite) TestValidatorByUnbondingIDAccessors() {
	_, valAddrs := createValAddrs(3)

	type exists struct {
		setValidator              bool
		setValidatorByUnbondingID bool
	}

	cases := []struct {
		exists    exists
		name      string
		validator types.Validator
	}{
		{
			name:      "existing 1",
			exists:    exists{true, true},
			validator: testutil.NewValidator(s.T(), valAddrs[0], PKs[0]),
		},
		{
			name:      "not existing 1",
			exists:    exists{false, true},
			validator: testutil.NewValidator(s.T(), valAddrs[1], PKs[1]),
		},
		{
			name:      "not existing 2",
			exists:    exists{false, false},
			validator: testutil.NewValidator(s.T(), valAddrs[2], PKs[0]),
		},
	}

	for i, tc := range cases {
		s.Run(tc.name, func() {
			if tc.exists.setValidator {
				s.stakingKeeper.SetValidator(s.ctx, tc.validator)
			}

			if tc.exists.setValidatorByUnbondingID {
				s.stakingKeeper.SetValidatorByUnbondingID(s.ctx, tc.validator, uint64(i))
			}

			val, found := s.stakingKeeper.GetValidatorByUnbondingID(s.ctx, uint64(i))
			if tc.exists.setValidator && tc.exists.setValidatorByUnbondingID {
				s.Require().True(found)
				s.Require().Equal(tc.validator, val)
			} else {
				s.Require().False(found)
			}
		})
	}
}

func (s *KeeperTestSuite) TestUnbondingCanComplete() {
	delAddrs, valAddrs := createValAddrs(3)
	unbondingID := uint64(1)

	// no unbondingID set
	err := s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().ErrorIs(err, types.ErrUnbondingNotFound)

	// unbonding delegation
	s.stakingKeeper.SetUnbondingType(s.ctx, unbondingID, types.UnbondingType_UnbondingDelegation)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().ErrorIs(err, types.ErrUnbondingNotFound)

	ubd := types.NewUnbondingDelegation(
		delAddrs[0],
		valAddrs[0],
		0,
		time.Unix(0, 0).UTC(),
		sdk.NewInt(5),
		unbondingID,
	)
	s.stakingKeeper.SetUnbondingDelegation(s.ctx, ubd)
	s.stakingKeeper.SetUnbondingDelegationByUnbondingID(s.ctx, ubd, unbondingID)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().ErrorIs(err, types.ErrUnbondingOnHoldRefCountNegative)

	err = s.stakingKeeper.PutUnbondingOnHold(s.ctx, unbondingID)
	s.Require().NoError(err)
	s.bankKeeper.EXPECT().UndelegateCoinsFromModuleToAccount(s.ctx, types.NotBondedPoolName, delAddrs[0], sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(5)))).Return(nil)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().NoError(err)

	// redelegation
	unbondingID++
	s.stakingKeeper.SetUnbondingType(s.ctx, unbondingID, types.UnbondingType_Redelegation)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().ErrorIs(err, types.ErrUnbondingNotFound)

	red := types.NewRedelegation(
		delAddrs[0],
		valAddrs[0],
		valAddrs[1],
		0,
		time.Unix(5, 0).UTC(),
		sdk.NewInt(10),
		math.LegacyNewDec(10),
		unbondingID,
	)
	s.stakingKeeper.SetRedelegation(s.ctx, red)
	s.stakingKeeper.SetRedelegationByUnbondingID(s.ctx, red, unbondingID)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().ErrorIs(err, types.ErrUnbondingOnHoldRefCountNegative)

	err = s.stakingKeeper.PutUnbondingOnHold(s.ctx, unbondingID)
	s.Require().NoError(err)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().NoError(err)

	// validator unbonding
	unbondingID++
	s.stakingKeeper.SetUnbondingType(s.ctx, unbondingID, types.UnbondingType_ValidatorUnbonding)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().ErrorIs(err, types.ErrUnbondingNotFound)

	val := testutil.NewValidator(s.T(), valAddrs[0], PKs[0])
	s.stakingKeeper.SetValidator(s.ctx, val)
	s.stakingKeeper.SetValidatorByUnbondingID(s.ctx, val, unbondingID)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().ErrorIs(err, types.ErrUnbondingOnHoldRefCountNegative)

	err = s.stakingKeeper.PutUnbondingOnHold(s.ctx, unbondingID)
	s.Require().NoError(err)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	s.Require().NoError(err)
}
