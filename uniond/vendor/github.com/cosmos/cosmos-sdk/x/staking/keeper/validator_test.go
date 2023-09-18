package keeper_test

import (
	"time"

	"github.com/golang/mock/gomock"

	"cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"

	abci "github.com/cometbft/cometbft/abci/types"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
	"github.com/cosmos/cosmos-sdk/x/staking/testutil"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

func (s *KeeperTestSuite) applyValidatorSetUpdates(ctx sdk.Context, keeper *stakingkeeper.Keeper, expectedUpdatesLen int) []abci.ValidatorUpdate {
	updates, err := keeper.ApplyAndReturnValidatorSetUpdates(ctx)
	s.Require().NoError(err)
	if expectedUpdatesLen >= 0 {
		s.Require().Equal(expectedUpdatesLen, len(updates), "%v", updates)
	}
	return updates
}

func (s *KeeperTestSuite) TestValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	valPubKey := PKs[0]
	valAddr := sdk.ValAddress(valPubKey.Address().Bytes())
	valTokens := keeper.TokensFromConsensusPower(ctx, 10)

	// test how the validator is set from a purely unbonbed pool
	validator := testutil.NewValidator(s.T(), valAddr, valPubKey)
	validator, _ = validator.AddTokensFromDel(valTokens)
	require.Equal(stakingtypes.Unbonded, validator.Status)
	require.Equal(valTokens, validator.Tokens)
	require.Equal(valTokens, validator.DelegatorShares.RoundInt())
	keeper.SetValidator(ctx, validator)
	keeper.SetValidatorByPowerIndex(ctx, validator)
	keeper.SetValidatorByConsAddr(ctx, validator)

	// ensure update
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	updates := s.applyValidatorSetUpdates(ctx, keeper, 1)
	validator, found := keeper.GetValidator(ctx, valAddr)
	require.True(found)
	require.Equal(validator.ABCIValidatorUpdate(keeper.PowerReduction(ctx)), updates[0])

	// after the save the validator should be bonded
	require.Equal(stakingtypes.Bonded, validator.Status)
	require.Equal(valTokens, validator.Tokens)
	require.Equal(valTokens, validator.DelegatorShares.RoundInt())

	// check each store for being saved
	consAddr, err := validator.GetConsAddr()
	require.NoError(err)
	resVal, found := keeper.GetValidatorByConsAddr(ctx, consAddr)
	require.True(found)
	require.True(validator.MinEqual(&resVal))

	resVals := keeper.GetLastValidators(ctx)
	require.Equal(1, len(resVals))
	require.True(validator.MinEqual(&resVals[0]))

	resVals = keeper.GetBondedValidatorsByPower(ctx)
	require.Equal(1, len(resVals))
	require.True(validator.MinEqual(&resVals[0]))

	allVals := keeper.GetAllValidators(ctx)
	require.Equal(1, len(allVals))

	// check the last validator power
	power := int64(100)
	keeper.SetLastValidatorPower(ctx, valAddr, power)
	resPower := keeper.GetLastValidatorPower(ctx, valAddr)
	require.Equal(power, resPower)
	keeper.DeleteLastValidatorPower(ctx, valAddr)
	resPower = keeper.GetLastValidatorPower(ctx, valAddr)
	require.Equal(int64(0), resPower)
}

// This function tests UpdateValidator, GetValidator, GetLastValidators, RemoveValidator
func (s *KeeperTestSuite) TestValidatorBasics() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	// construct the validators
	var validators [3]stakingtypes.Validator
	powers := []int64{9, 8, 7}
	for i, power := range powers {
		validators[i] = testutil.NewValidator(s.T(), sdk.ValAddress(PKs[i].Address().Bytes()), PKs[i])
		validators[i].Status = stakingtypes.Unbonded
		validators[i].Tokens = math.ZeroInt()
		tokens := keeper.TokensFromConsensusPower(ctx, power)

		validators[i], _ = validators[i].AddTokensFromDel(tokens)
	}

	require.Equal(keeper.TokensFromConsensusPower(ctx, 9), validators[0].Tokens)
	require.Equal(keeper.TokensFromConsensusPower(ctx, 8), validators[1].Tokens)
	require.Equal(keeper.TokensFromConsensusPower(ctx, 7), validators[2].Tokens)

	// check the empty keeper first
	_, found := keeper.GetValidator(ctx, sdk.ValAddress(PKs[0].Address().Bytes()))
	require.False(found)
	resVals := keeper.GetLastValidators(ctx)
	require.Zero(len(resVals))

	resVals = keeper.GetValidators(ctx, 2)
	require.Len(resVals, 0)

	// set and retrieve a record
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validators[0] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[0], true)
	keeper.SetValidatorByConsAddr(ctx, validators[0])
	resVal, found := keeper.GetValidator(ctx, sdk.ValAddress(PKs[0].Address().Bytes()))
	require.True(found)
	require.True(validators[0].MinEqual(&resVal))

	// retrieve from consensus
	resVal, found = keeper.GetValidatorByConsAddr(ctx, sdk.ConsAddress(PKs[0].Address()))
	require.True(found)
	require.True(validators[0].MinEqual(&resVal))
	resVal, found = keeper.GetValidatorByConsAddr(ctx, sdk.GetConsAddress(PKs[0]))
	require.True(found)
	require.True(validators[0].MinEqual(&resVal))

	resVals = keeper.GetLastValidators(ctx)
	require.Equal(1, len(resVals))
	require.True(validators[0].MinEqual(&resVals[0]))
	require.Equal(stakingtypes.Bonded, validators[0].Status)
	require.True(keeper.TokensFromConsensusPower(ctx, 9).Equal(validators[0].BondedTokens()))

	// modify a records, save, and retrieve
	validators[0].Status = stakingtypes.Bonded
	validators[0].Tokens = keeper.TokensFromConsensusPower(ctx, 10)
	validators[0].DelegatorShares = sdk.NewDecFromInt(validators[0].Tokens)
	validators[0] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[0], true)
	resVal, found = keeper.GetValidator(ctx, sdk.ValAddress(PKs[0].Address().Bytes()))
	require.True(found)
	require.True(validators[0].MinEqual(&resVal))

	resVals = keeper.GetLastValidators(ctx)
	require.Equal(1, len(resVals))
	require.True(validators[0].MinEqual(&resVals[0]))

	// add other validators
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validators[1] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[1], true)
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validators[2] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[2], true)
	resVal, found = keeper.GetValidator(ctx, sdk.ValAddress(PKs[1].Address().Bytes()))
	require.True(found)
	require.True(validators[1].MinEqual(&resVal))
	resVal, found = keeper.GetValidator(ctx, sdk.ValAddress(PKs[2].Address().Bytes()))
	require.True(found)
	require.True(validators[2].MinEqual(&resVal))

	resVals = keeper.GetLastValidators(ctx)
	require.Equal(3, len(resVals))

	// remove a record

	// shouldn't be able to remove if status is not unbonded
	require.PanicsWithValue("cannot call RemoveValidator on bonded or unbonding validators",
		func() { keeper.RemoveValidator(ctx, validators[1].GetOperator()) })

	// shouldn't be able to remove if there are still tokens left
	validators[1].Status = stakingtypes.Unbonded
	keeper.SetValidator(ctx, validators[1])
	require.PanicsWithValue("attempting to remove a validator which still contains tokens",
		func() { keeper.RemoveValidator(ctx, validators[1].GetOperator()) })

	validators[1].Tokens = math.ZeroInt()                    // ...remove all tokens
	keeper.SetValidator(ctx, validators[1])                  // ...set the validator
	keeper.RemoveValidator(ctx, validators[1].GetOperator()) // Now it can be removed.
	_, found = keeper.GetValidator(ctx, sdk.ValAddress(PKs[1].Address().Bytes()))
	require.False(found)
}

func (s *KeeperTestSuite) TestUpdateValidatorByPowerIndex() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	valPubKey := PKs[0]
	valAddr := sdk.ValAddress(valPubKey.Address().Bytes())
	valTokens := keeper.TokensFromConsensusPower(ctx, 100)

	// add a validator
	validator := testutil.NewValidator(s.T(), valAddr, PKs[0])
	validator, delSharesCreated := validator.AddTokensFromDel(valTokens)
	require.Equal(stakingtypes.Unbonded, validator.Status)
	require.Equal(valTokens, validator.Tokens)

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	validator, found := keeper.GetValidator(ctx, valAddr)
	require.True(found)
	require.Equal(valTokens, validator.Tokens)

	power := stakingtypes.GetValidatorsByPowerIndexKey(validator, keeper.PowerReduction(ctx))
	require.True(stakingkeeper.ValidatorByPowerIndexExists(ctx, keeper, power))

	// burn half the delegator shares
	keeper.DeleteValidatorByPowerIndex(ctx, validator)
	validator, burned := validator.RemoveDelShares(delSharesCreated.Quo(math.LegacyNewDec(2)))
	require.Equal(keeper.TokensFromConsensusPower(ctx, 50), burned)
	stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true) // update the validator, possibly kicking it out
	require.False(stakingkeeper.ValidatorByPowerIndexExists(ctx, keeper, power))

	validator, found = keeper.GetValidator(ctx, valAddr)
	require.True(found)

	power = stakingtypes.GetValidatorsByPowerIndexKey(validator, keeper.PowerReduction(ctx))
	require.True(stakingkeeper.ValidatorByPowerIndexExists(ctx, keeper, power))

	// set new validator by power index
	keeper.DeleteValidatorByPowerIndex(ctx, validator)
	require.False(stakingkeeper.ValidatorByPowerIndexExists(ctx, keeper, power))
	keeper.SetNewValidatorByPowerIndex(ctx, validator)
	require.True(stakingkeeper.ValidatorByPowerIndexExists(ctx, keeper, power))
}

func (s *KeeperTestSuite) TestApplyAndReturnValidatorSetUpdatesPowerDecrease() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	powers := []int64{100, 100}
	var validators [2]stakingtypes.Validator

	for i, power := range powers {
		validators[i] = testutil.NewValidator(s.T(), sdk.ValAddress(PKs[i].Address().Bytes()), PKs[i])
		tokens := keeper.TokensFromConsensusPower(ctx, power)
		validators[i], _ = validators[i].AddTokensFromDel(tokens)

	}

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validators[0] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[0], false)
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validators[1] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[1], false)
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	s.applyValidatorSetUpdates(ctx, keeper, 2)

	// check initial power
	require.Equal(int64(100), validators[0].GetConsensusPower(keeper.PowerReduction(ctx)))
	require.Equal(int64(100), validators[1].GetConsensusPower(keeper.PowerReduction(ctx)))

	// test multiple value change
	//  tendermintUpdate set: {c1, c3} -> {c1', c3'}
	delTokens1 := keeper.TokensFromConsensusPower(ctx, 20)
	delTokens2 := keeper.TokensFromConsensusPower(ctx, 30)
	validators[0], _ = validators[0].RemoveDelShares(sdk.NewDecFromInt(delTokens1))
	validators[1], _ = validators[1].RemoveDelShares(sdk.NewDecFromInt(delTokens2))
	validators[0] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[0], false)
	validators[1] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[1], false)

	// power has changed
	require.Equal(int64(80), validators[0].GetConsensusPower(keeper.PowerReduction(ctx)))
	require.Equal(int64(70), validators[1].GetConsensusPower(keeper.PowerReduction(ctx)))

	// Tendermint updates should reflect power change
	updates := s.applyValidatorSetUpdates(ctx, keeper, 2)
	require.Equal(validators[0].ABCIValidatorUpdate(keeper.PowerReduction(ctx)), updates[0])
	require.Equal(validators[1].ABCIValidatorUpdate(keeper.PowerReduction(ctx)), updates[1])
}

func (s *KeeperTestSuite) TestUpdateValidatorCommission() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	// Set MinCommissionRate to 0.05
	params := keeper.GetParams(ctx)
	params.MinCommissionRate = sdk.NewDecWithPrec(5, 2)
	keeper.SetParams(ctx, params)

	commission1 := stakingtypes.NewCommissionWithTime(
		sdk.NewDecWithPrec(1, 1), sdk.NewDecWithPrec(3, 1),
		sdk.NewDecWithPrec(1, 1), time.Now().UTC().Add(time.Duration(-1)*time.Hour),
	)
	commission2 := stakingtypes.NewCommission(sdk.NewDecWithPrec(1, 1), sdk.NewDecWithPrec(3, 1), sdk.NewDecWithPrec(1, 1))

	val1 := testutil.NewValidator(s.T(), sdk.ValAddress(PKs[0].Address().Bytes()), PKs[0])
	val2 := testutil.NewValidator(s.T(), sdk.ValAddress(PKs[1].Address().Bytes()), PKs[1])

	val1, _ = val1.SetInitialCommission(commission1)
	val2, _ = val2.SetInitialCommission(commission2)

	keeper.SetValidator(ctx, val1)
	keeper.SetValidator(ctx, val2)

	testCases := []struct {
		validator   stakingtypes.Validator
		newRate     sdk.Dec
		expectedErr bool
	}{
		{val1, math.LegacyZeroDec(), true},
		{val2, sdk.NewDecWithPrec(-1, 1), true},
		{val2, sdk.NewDecWithPrec(4, 1), true},
		{val2, sdk.NewDecWithPrec(3, 1), true},
		{val2, sdk.NewDecWithPrec(1, 2), true},
		{val2, sdk.NewDecWithPrec(2, 1), false},
	}

	for i, tc := range testCases {
		commission, err := keeper.UpdateValidatorCommission(ctx, tc.validator, tc.newRate)

		if tc.expectedErr {
			require.Error(err, "expected error for test case #%d with rate: %s", i, tc.newRate)
		} else {
			tc.validator.Commission = commission
			keeper.SetValidator(ctx, tc.validator)
			val, found := keeper.GetValidator(ctx, tc.validator.GetOperator())

			require.True(found,
				"expected to find validator for test case #%d with rate: %s", i, tc.newRate,
			)
			require.NoError(err,
				"unexpected error for test case #%d with rate: %s", i, tc.newRate,
			)
			require.Equal(tc.newRate, val.Commission.Rate,
				"expected new validator commission rate for test case #%d with rate: %s", i, tc.newRate,
			)
			require.Equal(ctx.BlockHeader().Time, val.Commission.UpdateTime,
				"expected new validator commission update time for test case #%d with rate: %s", i, tc.newRate,
			)
		}
	}
}

func (s *KeeperTestSuite) TestValidatorToken() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	valPubKey := PKs[0]
	valAddr := sdk.ValAddress(valPubKey.Address().Bytes())
	addTokens := keeper.TokensFromConsensusPower(ctx, 10)
	delTokens := keeper.TokensFromConsensusPower(ctx, 5)

	validator := testutil.NewValidator(s.T(), valAddr, valPubKey)
	validator, _ = keeper.AddValidatorTokensAndShares(ctx, validator, addTokens)
	require.Equal(addTokens, validator.Tokens)
	validator, _ = keeper.GetValidator(ctx, valAddr)
	require.Equal(sdk.NewDecFromInt(addTokens), validator.DelegatorShares)

	keeper.RemoveValidatorTokensAndShares(ctx, validator, sdk.NewDecFromInt(delTokens))
	validator, _ = keeper.GetValidator(ctx, valAddr)
	require.Equal(delTokens, validator.Tokens)
	require.True(validator.DelegatorShares.Equal(sdk.NewDecFromInt(delTokens)))

	keeper.RemoveValidatorTokens(ctx, validator, delTokens)
	validator, _ = keeper.GetValidator(ctx, valAddr)
	require.True(validator.Tokens.IsZero())
}

func (s *KeeperTestSuite) TestUnbondingValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	valPubKey := PKs[0]
	valAddr := sdk.ValAddress(valPubKey.Address().Bytes())
	validator := testutil.NewValidator(s.T(), valAddr, valPubKey)
	addTokens := keeper.TokensFromConsensusPower(ctx, 10)

	// set unbonding validator
	endTime := time.Now()
	endHeight := ctx.BlockHeight() + 10
	keeper.SetUnbondingValidatorsQueue(ctx, endTime, endHeight, []string{valAddr.String()})

	resVals := keeper.GetUnbondingValidators(ctx, endTime, endHeight)
	require.Equal(1, len(resVals))
	require.Equal(valAddr.String(), resVals[0])

	// add another unbonding validator
	valAddr1 := sdk.ValAddress(PKs[1].Address().Bytes())
	validator1 := testutil.NewValidator(s.T(), valAddr1, PKs[1])
	validator1.UnbondingHeight = endHeight
	validator1.UnbondingTime = endTime
	keeper.InsertUnbondingValidatorQueue(ctx, validator1)

	resVals = keeper.GetUnbondingValidators(ctx, endTime, endHeight)
	require.Equal(2, len(resVals))

	// delete unbonding validator from the queue
	keeper.DeleteValidatorQueue(ctx, validator1)
	resVals = keeper.GetUnbondingValidators(ctx, endTime, endHeight)
	require.Equal(1, len(resVals))
	require.Equal(valAddr.String(), resVals[0])

	// check unbonding mature validators
	ctx = ctx.WithBlockHeight(endHeight).WithBlockTime(endTime)
	require.PanicsWithValue("validator in the unbonding queue was not found", func() {
		keeper.UnbondAllMatureValidators(ctx)
	})

	keeper.SetValidator(ctx, validator)
	ctx = ctx.WithBlockHeight(endHeight).WithBlockTime(endTime)
	require.PanicsWithValue("unexpected validator in unbonding queue; status was not unbonding", func() {
		keeper.UnbondAllMatureValidators(ctx)
	})

	validator.Status = stakingtypes.Unbonding
	keeper.SetValidator(ctx, validator)
	keeper.UnbondAllMatureValidators(ctx)
	validator, found := keeper.GetValidator(ctx, valAddr)
	require.False(found)

	keeper.SetUnbondingValidatorsQueue(ctx, endTime, endHeight, []string{valAddr.String()})
	validator = testutil.NewValidator(s.T(), valAddr, valPubKey)
	validator, _ = validator.AddTokensFromDel(addTokens)
	validator.Status = stakingtypes.Unbonding
	keeper.SetValidator(ctx, validator)
	keeper.UnbondAllMatureValidators(ctx)
	validator, found = keeper.GetValidator(ctx, valAddr)
	require.True(found)
	require.Equal(stakingtypes.Unbonded, validator.Status)
}
