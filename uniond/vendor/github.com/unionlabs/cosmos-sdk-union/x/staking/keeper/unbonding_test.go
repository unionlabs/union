package keeper_test

import (
	"time"

	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/testutil"
	"cosmossdk.io/x/staking/types"

	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *KeeperTestSuite) TestIncrementUnbondingID() {
	for i := 1; i < 10; i++ {
		id, err := s.stakingKeeper.IncrementUnbondingID(s.ctx)
		s.Require().NoError(err)
		s.Require().Equal(uint64(i), id)
	}
}

func (s *KeeperTestSuite) TestUnbondingTypeAccessors() {
	require := s.Require()
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
				require.NoError(s.stakingKeeper.SetUnbondingType(s.ctx, uint64(i), tc.expected))
			}

			unbondingType, err := s.stakingKeeper.GetUnbondingType(s.ctx, uint64(i))
			if tc.exists {
				require.NoError(err)
				require.Equal(tc.expected, unbondingType)
			} else {
				require.ErrorIs(err, types.ErrNoUnbondingType)
			}
		})
	}
}

func (s *KeeperTestSuite) TestUnbondingDelegationByUnbondingIDAccessors() {
	delAddrs, valAddrs := createValAddrs(2)
	require := s.Require()

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
				math.NewInt(5),
				0,
				addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"),
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
				math.NewInt(5),
				0,
				addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"),
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
				math.NewInt(5),
				0,
				addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"),
			),
		},
	}

	for i, tc := range cases {
		s.Run(tc.name, func() {
			if tc.exists.setUnbondingDelegation {
				require.NoError(s.stakingKeeper.SetUnbondingDelegation(s.ctx, tc.expected))
			}

			if tc.exists.setUnbondingDelegationByUnbondingID {
				require.NoError(s.stakingKeeper.SetUnbondingDelegationByUnbondingID(s.ctx, tc.expected, uint64(i)))
			}

			ubd, err := s.stakingKeeper.GetUnbondingDelegationByUnbondingID(s.ctx, uint64(i))
			if tc.exists.setUnbondingDelegation && tc.exists.setUnbondingDelegationByUnbondingID {
				require.NoError(err)
				require.Equal(tc.expected, ubd)
			} else {
				require.ErrorIs(err, types.ErrNoUnbondingDelegation)
			}
		})
	}
}

func (s *KeeperTestSuite) TestRedelegationByUnbondingIDAccessors() {
	delAddrs, valAddrs := createValAddrs(2)
	require := s.Require()

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
				math.NewInt(10),
				math.LegacyNewDec(10),
				0,
				addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"),
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
				math.NewInt(10),
				math.LegacyNewDec(10),
				0,
				addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"),
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
				math.NewInt(10),
				math.LegacyNewDec(10),
				0,
				addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"),
			),
		},
	}

	for i, tc := range cases {
		s.Run(tc.name, func() {
			if tc.exists.setRedelegation {
				require.NoError(s.stakingKeeper.SetRedelegation(s.ctx, tc.expected))
			}

			if tc.exists.setRedelegationByUnbondingID {
				require.NoError(s.stakingKeeper.SetRedelegationByUnbondingID(s.ctx, tc.expected, uint64(i)))
			}

			red, err := s.stakingKeeper.GetRedelegationByUnbondingID(s.ctx, uint64(i))
			if tc.exists.setRedelegation && tc.exists.setRedelegationByUnbondingID {
				require.NoError(err)
				require.Equal(tc.expected, red)
			} else {
				require.ErrorIs(err, types.ErrNoRedelegation)
			}
		})
	}
}

func (s *KeeperTestSuite) TestValidatorByUnbondingIDAccessors() {
	_, valAddrs := createValAddrs(3)
	require := s.Require()

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
				require.NoError(s.stakingKeeper.SetValidator(s.ctx, tc.validator))
			}

			if tc.exists.setValidatorByUnbondingID {
				require.NoError(s.stakingKeeper.SetValidatorByUnbondingID(s.ctx, tc.validator, uint64(i)))
			}

			val, err := s.stakingKeeper.GetValidatorByUnbondingID(s.ctx, uint64(i))
			if tc.exists.setValidator && tc.exists.setValidatorByUnbondingID {
				require.NoError(err)
				require.Equal(tc.validator, val)
			} else {
				require.ErrorIs(err, types.ErrNoValidatorFound)
			}
		})
	}
}

func (s *KeeperTestSuite) TestUnbondingCanComplete() {
	delAddrs, valAddrs := createValAddrs(3)
	require := s.Require()

	unbondingID := uint64(1)

	// no unbondingID set
	err := s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	require.ErrorIs(err, types.ErrNoUnbondingType)

	// unbonding delegation
	require.NoError(s.stakingKeeper.SetUnbondingType(s.ctx, unbondingID, types.UnbondingType_UnbondingDelegation))
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	require.ErrorIs(err, types.ErrNoUnbondingDelegation)

	ubd := types.NewUnbondingDelegation(
		delAddrs[0],
		valAddrs[0],
		0,
		time.Unix(0, 0).UTC(),
		math.NewInt(5),
		unbondingID,
		addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"),
	)
	require.NoError(s.stakingKeeper.SetUnbondingDelegation(s.ctx, ubd))
	require.NoError(s.stakingKeeper.SetUnbondingDelegationByUnbondingID(s.ctx, ubd, unbondingID))
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	require.ErrorIs(err, types.ErrUnbondingOnHoldRefCountNegative)

	err = s.stakingKeeper.PutUnbondingOnHold(s.ctx, unbondingID)
	require.NoError(err)
	s.bankKeeper.EXPECT().UndelegateCoinsFromModuleToAccount(s.ctx, types.NotBondedPoolName, delAddrs[0], sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(5)))).Return(nil)
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	require.NoError(err)

	// redelegation
	unbondingID++
	require.NoError(s.stakingKeeper.SetUnbondingType(s.ctx, unbondingID, types.UnbondingType_Redelegation))
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	require.ErrorIs(err, types.ErrNoRedelegation)

	red := types.NewRedelegation(
		delAddrs[0],
		valAddrs[0],
		valAddrs[1],
		0,
		time.Unix(5, 0).UTC(),
		math.NewInt(10),
		math.LegacyNewDec(10),
		unbondingID,
		addresscodec.NewBech32Codec("cosmosvaloper"), addresscodec.NewBech32Codec("cosmos"),
	)
	require.NoError(s.stakingKeeper.SetRedelegation(s.ctx, red))
	require.NoError(s.stakingKeeper.SetRedelegationByUnbondingID(s.ctx, red, unbondingID))
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	require.ErrorIs(err, types.ErrUnbondingOnHoldRefCountNegative)

	require.NoError(s.stakingKeeper.PutUnbondingOnHold(s.ctx, unbondingID))
	require.NoError(s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID))

	// validator unbonding
	unbondingID++
	require.NoError(s.stakingKeeper.SetUnbondingType(s.ctx, unbondingID, types.UnbondingType_ValidatorUnbonding))
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	require.ErrorIs(err, types.ErrNoValidatorFound)

	val := testutil.NewValidator(s.T(), valAddrs[0], PKs[0])
	require.NoError(s.stakingKeeper.SetValidator(s.ctx, val))
	require.NoError(s.stakingKeeper.SetValidatorByUnbondingID(s.ctx, val, unbondingID))
	err = s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID)
	require.ErrorIs(err, types.ErrUnbondingOnHoldRefCountNegative)

	require.NoError(s.stakingKeeper.PutUnbondingOnHold(s.ctx, unbondingID))
	require.NoError(s.stakingKeeper.UnbondingCanComplete(s.ctx, unbondingID))
}
