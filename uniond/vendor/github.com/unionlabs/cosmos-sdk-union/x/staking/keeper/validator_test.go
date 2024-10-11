package keeper_test

import (
	"time"

	"github.com/golang/mock/gomock"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/header"
	"cosmossdk.io/math"
	stakingkeeper "cosmossdk.io/x/staking/keeper"
	"cosmossdk.io/x/staking/testutil"
	stakingtypes "cosmossdk.io/x/staking/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

func (s *KeeperTestSuite) applyValidatorSetUpdates(ctx sdk.Context, keeper *stakingkeeper.Keeper, expectedUpdatesLen int) []appmodule.ValidatorUpdate {
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
	require.NoError(keeper.SetValidator(ctx, validator))
	require.NoError(keeper.SetValidatorByPowerIndex(ctx, validator))
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))

	// ensure update
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	updates := s.applyValidatorSetUpdates(ctx, keeper, 1)
	validator, err := keeper.GetValidator(ctx, valAddr)
	require.NoError(err)
	require.Equal(validator.ModuleValidatorUpdate(keeper.PowerReduction(ctx)), updates[0])

	// after the save the validator should be bonded
	require.Equal(stakingtypes.Bonded, validator.Status)
	require.Equal(valTokens, validator.Tokens)
	require.Equal(valTokens, validator.DelegatorShares.RoundInt())

	// check each store for being saved
	consAddr, err := validator.GetConsAddr()
	require.NoError(err)
	resVal, err := keeper.GetValidatorByConsAddr(ctx, consAddr)
	require.NoError(err)
	require.True(validator.MinEqual(&resVal))

	resVals, err := keeper.GetLastValidators(ctx)
	require.NoError(err)
	require.Equal(1, len(resVals))
	require.True(validator.MinEqual(&resVals[0]))

	resVals, err = keeper.GetBondedValidatorsByPower(ctx)
	require.NoError(err)
	require.Equal(1, len(resVals))
	require.True(validator.MinEqual(&resVals[0]))

	allVals, err := keeper.GetAllValidators(ctx)
	require.NoError(err)
	require.Equal(1, len(allVals))

	// check the last validator power
	power := int64(100)
	require.NoError(keeper.SetLastValidatorPower(ctx, valAddr, power))
	resPower, err := keeper.GetLastValidatorPower(ctx, valAddr)
	require.NoError(err)
	require.Equal(power, resPower)
	require.NoError(keeper.DeleteLastValidatorPower(ctx, valAddr))
	resPower, err = keeper.GetLastValidatorPower(ctx, valAddr)
	require.Error(err, collections.ErrNotFound)
	require.Equal(int64(0), resPower)
}

func (s *KeeperTestSuite) TestGetLastValidators() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	params, err := keeper.Params.Get(ctx)
	require.NoError(err)

	params.MaxValidators = 50
	require.NoError(keeper.Params.Set(ctx, params))

	// construct 50 validators all with equal power of 100
	var validators [50]stakingtypes.Validator
	for i := 0; i < 50; i++ {
		validators[i] = testutil.NewValidator(s.T(), sdk.ValAddress(PKs[i].Address().Bytes()), PKs[i])
		validators[i].Status = stakingtypes.Unbonded
		validators[i].Tokens = math.ZeroInt()
		tokens := keeper.TokensFromConsensusPower(ctx, 100)

		validators[i], _ = validators[i].AddTokensFromDel(tokens)
		require.Equal(keeper.TokensFromConsensusPower(ctx, 100), validators[i].Tokens)

		s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())

		validators[i] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[i], true)
		require.NoError(keeper.SetValidatorByConsAddr(ctx, validators[i]))

		resVal, err := keeper.GetValidator(ctx, sdk.ValAddress(PKs[i].Address().Bytes()))
		require.NoError(err)
		require.True(validators[i].MinEqual(&resVal))
	}

	res, err := keeper.GetLastValidators(ctx)
	require.NoError(err)
	require.Len(res, 50)

	// reduce max validators to 30 and ensure we only get 30 back
	params.MaxValidators = 30
	require.NoError(keeper.Params.Set(ctx, params))

	res, err = keeper.GetLastValidators(ctx)
	require.NoError(err)
	require.Len(res, 30)
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
	_, err := keeper.GetValidator(ctx, sdk.ValAddress(PKs[0].Address().Bytes()))
	require.ErrorIs(err, stakingtypes.ErrNoValidatorFound)
	resVals, err := keeper.GetLastValidators(ctx)
	require.NoError(err)
	require.Zero(len(resVals))

	resVals, err = keeper.GetValidators(ctx, 2)
	require.NoError(err)
	require.Len(resVals, 0)

	// set and retrieve a record
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validators[0] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[0], true)
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validators[0]))
	resVal, err := keeper.GetValidator(ctx, sdk.ValAddress(PKs[0].Address().Bytes()))
	require.NoError(err)
	require.True(validators[0].MinEqual(&resVal))

	// retrieve from consensus
	resVal, err = keeper.GetValidatorByConsAddr(ctx, sdk.ConsAddress(PKs[0].Address()))
	require.NoError(err)
	require.True(validators[0].MinEqual(&resVal))
	resVal, err = keeper.GetValidatorByConsAddr(ctx, sdk.GetConsAddress(PKs[0]))
	require.NoError(err)
	require.True(validators[0].MinEqual(&resVal))

	resVals, err = keeper.GetLastValidators(ctx)
	require.NoError(err)
	require.Equal(1, len(resVals))
	require.True(validators[0].MinEqual(&resVals[0]))
	require.Equal(stakingtypes.Bonded, validators[0].Status)
	require.True(keeper.TokensFromConsensusPower(ctx, 9).Equal(validators[0].BondedTokens()))

	// modify a records, save, and retrieve
	validators[0].Status = stakingtypes.Bonded
	validators[0].Tokens = keeper.TokensFromConsensusPower(ctx, 10)
	validators[0].DelegatorShares = math.LegacyNewDecFromInt(validators[0].Tokens)
	validators[0] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[0], true)
	resVal, err = keeper.GetValidator(ctx, sdk.ValAddress(PKs[0].Address().Bytes()))
	require.NoError(err)
	require.True(validators[0].MinEqual(&resVal))

	resVals, err = keeper.GetLastValidators(ctx)
	require.NoError(err)
	require.Equal(1, len(resVals))
	require.True(validators[0].MinEqual(&resVals[0]))

	// add other validators
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validators[1] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[1], true)
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validators[2] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[2], true)
	resVal, err = keeper.GetValidator(ctx, sdk.ValAddress(PKs[1].Address().Bytes()))
	require.NoError(err)
	require.True(validators[1].MinEqual(&resVal))
	resVal, err = keeper.GetValidator(ctx, sdk.ValAddress(PKs[2].Address().Bytes()))
	require.NoError(err)
	require.True(validators[2].MinEqual(&resVal))

	resVals, err = keeper.GetLastValidators(ctx)
	require.NoError(err)
	require.Equal(3, len(resVals))

	// remove a record

	bz, err := keeper.ValidatorAddressCodec().StringToBytes(validators[1].GetOperator())
	require.NoError(err)

	// shouldn't be able to remove if status is not unbonded
	require.EqualError(keeper.RemoveValidator(ctx, bz), "cannot call RemoveValidator on bonded or unbonding validators: failed to remove validator")

	// shouldn't be able to remove if there are still tokens left
	validators[1].Status = stakingtypes.Unbonded
	require.NoError(keeper.SetValidator(ctx, validators[1]))
	require.EqualError(keeper.RemoveValidator(ctx, bz), "attempting to remove a validator which still contains tokens: failed to remove validator")

	validators[1].Tokens = math.ZeroInt()                    // ...remove all tokens
	require.NoError(keeper.SetValidator(ctx, validators[1])) // ...set the validator
	require.NoError(keeper.RemoveValidator(ctx, bz))         // Now it can be removed.
	_, err = keeper.GetValidator(ctx, sdk.ValAddress(PKs[1].Address().Bytes()))
	require.ErrorIs(err, stakingtypes.ErrNoValidatorFound)
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
	validator, err := keeper.GetValidator(ctx, valAddr)
	require.NoError(err)
	require.Equal(valTokens, validator.Tokens)

	power := stakingtypes.GetValidatorsByPowerIndexKey(validator, keeper.PowerReduction(ctx), keeper.ValidatorAddressCodec())
	require.True(stakingkeeper.ValidatorByPowerIndexExists(ctx, keeper, power))

	// burn half the delegator shares
	require.NoError(keeper.DeleteValidatorByPowerIndex(ctx, validator))
	validator, burned := validator.RemoveDelShares(delSharesCreated.Quo(math.LegacyNewDec(2)))
	require.Equal(keeper.TokensFromConsensusPower(ctx, 50), burned)
	stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true) // update the validator, possibly kicking it out
	require.False(stakingkeeper.ValidatorByPowerIndexExists(ctx, keeper, power))

	validator, err = keeper.GetValidator(ctx, valAddr)
	require.NoError(err)

	power = stakingtypes.GetValidatorsByPowerIndexKey(validator, keeper.PowerReduction(ctx), keeper.ValidatorAddressCodec())
	require.True(stakingkeeper.ValidatorByPowerIndexExists(ctx, keeper, power))

	// set new validator by power index
	require.NoError(keeper.DeleteValidatorByPowerIndex(ctx, validator))
	require.False(stakingkeeper.ValidatorByPowerIndexExists(ctx, keeper, power))
	require.NoError(keeper.SetNewValidatorByPowerIndex(ctx, validator))
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
	// tendermintUpdate set: {c1, c3} -> {c1', c3'}
	delTokens1 := keeper.TokensFromConsensusPower(ctx, 20)
	delTokens2 := keeper.TokensFromConsensusPower(ctx, 30)
	validators[0], _ = validators[0].RemoveDelShares(math.LegacyNewDecFromInt(delTokens1))
	validators[1], _ = validators[1].RemoveDelShares(math.LegacyNewDecFromInt(delTokens2))
	validators[0] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[0], false)
	validators[1] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[1], false)

	// power has changed
	require.Equal(int64(80), validators[0].GetConsensusPower(keeper.PowerReduction(ctx)))
	require.Equal(int64(70), validators[1].GetConsensusPower(keeper.PowerReduction(ctx)))

	// CometBFT updates should reflect power change
	updates := s.applyValidatorSetUpdates(ctx, keeper, 2)
	require.Equal(validators[0].ModuleValidatorUpdate(keeper.PowerReduction(ctx)), updates[0])
	require.Equal(validators[1].ModuleValidatorUpdate(keeper.PowerReduction(ctx)), updates[1])
}

func (s *KeeperTestSuite) TestUpdateValidatorCommission() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	// Set MinCommissionRate to 0.05
	params, err := keeper.Params.Get(ctx)
	require.NoError(err)
	params.MinCommissionRate = math.LegacyNewDecWithPrec(5, 2)
	require.NoError(keeper.Params.Set(ctx, params))

	commission1 := stakingtypes.NewCommissionWithTime(
		math.LegacyNewDecWithPrec(1, 1), math.LegacyNewDecWithPrec(3, 1),
		math.LegacyNewDecWithPrec(1, 1), time.Now().UTC().Add(time.Duration(-1)*time.Hour),
	)
	commission2 := stakingtypes.NewCommission(math.LegacyNewDecWithPrec(1, 1), math.LegacyNewDecWithPrec(3, 1), math.LegacyNewDecWithPrec(1, 1))

	val1 := testutil.NewValidator(s.T(), sdk.ValAddress(PKs[0].Address().Bytes()), PKs[0])
	val2 := testutil.NewValidator(s.T(), sdk.ValAddress(PKs[1].Address().Bytes()), PKs[1])

	val1, _ = val1.SetInitialCommission(commission1)
	val2, _ = val2.SetInitialCommission(commission2)

	require.NoError(keeper.SetValidator(ctx, val1))
	require.NoError(keeper.SetValidator(ctx, val2))

	testCases := []struct {
		validator   stakingtypes.Validator
		newRate     math.LegacyDec
		expectedErr bool
	}{
		{val1, math.LegacyZeroDec(), true},
		{val2, math.LegacyNewDecWithPrec(-1, 1), true},
		{val2, math.LegacyNewDecWithPrec(4, 1), true},
		{val2, math.LegacyNewDecWithPrec(3, 1), true},
		{val2, math.LegacyNewDecWithPrec(1, 2), true},
		{val2, math.LegacyNewDecWithPrec(2, 1), false},
	}

	for i, tc := range testCases {
		commission, err := keeper.UpdateValidatorCommission(ctx, tc.validator, tc.newRate)

		if tc.expectedErr {
			require.Error(err, "expected error for test case #%d with rate: %s", i, tc.newRate)
		} else {
			require.NoError(err,
				"unexpected error for test case #%d with rate: %s", i, tc.newRate,
			)

			tc.validator.Commission = commission
			err = keeper.SetValidator(ctx, tc.validator)
			require.NoError(err)

			bz, err := keeper.ValidatorAddressCodec().StringToBytes(tc.validator.GetOperator())
			require.NoError(err)

			val, err := keeper.GetValidator(ctx, bz)
			require.NoError(err,
				"expected to find validator for test case #%d with rate: %s", i, tc.newRate,
			)

			require.Equal(tc.newRate, val.Commission.Rate,
				"expected new validator commission rate for test case #%d with rate: %s", i, tc.newRate,
			)
			require.Equal(ctx.HeaderInfo().Time, val.Commission.UpdateTime,
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
	validator, _, err := keeper.AddValidatorTokensAndShares(ctx, validator, addTokens)
	require.NoError(err)
	require.Equal(addTokens, validator.Tokens)
	validator, _ = keeper.GetValidator(ctx, valAddr)
	require.Equal(math.LegacyNewDecFromInt(addTokens), validator.DelegatorShares)

	_, _, err = keeper.RemoveValidatorTokensAndShares(ctx, validator, math.LegacyNewDecFromInt(delTokens))
	require.NoError(err)
	validator, _ = keeper.GetValidator(ctx, valAddr)
	require.Equal(delTokens, validator.Tokens)
	require.True(validator.DelegatorShares.Equal(math.LegacyNewDecFromInt(delTokens)))

	_, err = keeper.RemoveValidatorTokens(ctx, validator, delTokens)
	require.NoError(err)
	validator, _ = keeper.GetValidator(ctx, valAddr)
	require.True(validator.Tokens.IsZero())
}

// TestUnbondingValidator tests the functionality of unbonding a validator.
func (s *KeeperTestSuite) TestUnbondingValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	valPubKey := PKs[0]
	valAddr := sdk.ValAddress(valPubKey.Address().Bytes())
	validator := testutil.NewValidator(s.T(), valAddr, valPubKey)
	addTokens := keeper.TokensFromConsensusPower(ctx, 10)

	// set unbonding validator
	endTime := time.Now()
	endHeight := ctx.HeaderInfo().Height + 10
	require.NoError(keeper.SetUnbondingValidatorsQueue(ctx, endTime, endHeight, []string{s.valAddressToString(valAddr)}))

	resVals, err := keeper.GetUnbondingValidators(ctx, endTime, endHeight)
	require.NoError(err)
	require.Equal(1, len(resVals))
	require.Equal(s.valAddressToString(valAddr), resVals[0])

	// add another unbonding validator
	valAddr1 := sdk.ValAddress(PKs[1].Address().Bytes())
	validator1 := testutil.NewValidator(s.T(), valAddr1, PKs[1])
	validator1.UnbondingHeight = endHeight
	validator1.UnbondingTime = endTime
	require.NoError(keeper.InsertUnbondingValidatorQueue(ctx, validator1))

	resVals, err = keeper.GetUnbondingValidators(ctx, endTime, endHeight)
	require.NoError(err)
	require.Equal(2, len(resVals))

	// delete unbonding validator from the queue
	require.NoError(keeper.DeleteValidatorQueue(ctx, validator1))
	resVals, err = keeper.GetUnbondingValidators(ctx, endTime, endHeight)
	require.NoError(err)
	require.Equal(1, len(resVals))
	require.Equal(s.valAddressToString(valAddr), resVals[0])

	// check unbonding mature validators
	ctx = ctx.WithHeaderInfo(header.Info{Height: endHeight, Time: endTime})
	err = keeper.UnbondAllMatureValidators(ctx)
	require.EqualError(err, "validator in the unbonding queue was not found: validator does not exist")

	require.NoError(keeper.SetValidator(ctx, validator))
	ctx = ctx.WithHeaderInfo(header.Info{Height: endHeight, Time: endTime})

	err = keeper.UnbondAllMatureValidators(ctx)
	require.EqualError(err, "unexpected validator in unbonding queue; status was not unbonding")

	validator.Status = stakingtypes.Unbonding
	require.NoError(keeper.SetValidator(ctx, validator))
	require.NoError(keeper.UnbondAllMatureValidators(ctx))
	validator, err = keeper.GetValidator(ctx, valAddr)
	require.ErrorIs(err, stakingtypes.ErrNoValidatorFound)

	require.NoError(keeper.SetUnbondingValidatorsQueue(ctx, endTime, endHeight, []string{s.valAddressToString(valAddr)}))
	validator = testutil.NewValidator(s.T(), valAddr, valPubKey)
	validator, _ = validator.AddTokensFromDel(addTokens)
	validator.Status = stakingtypes.Unbonding
	require.NoError(keeper.SetValidator(ctx, validator))
	require.NoError(keeper.UnbondAllMatureValidators(ctx))
	validator, err = keeper.GetValidator(ctx, valAddr)
	require.NoError(err)
	require.Equal(stakingtypes.Unbonded, validator.Status)
}

func (s *KeeperTestSuite) TestValidatorConsPubKeyUpdate() {
	ctx, keeper, msgServer, bk, ak := s.ctx, s.stakingKeeper, s.msgServer, s.bankKeeper, s.accountKeeper
	require := s.Require()

	powers := []int64{10, 20}
	var validators [2]stakingtypes.Validator

	bonedPool := authtypes.NewEmptyModuleAccount(stakingtypes.BondedPoolName)
	ak.EXPECT().GetModuleAccount(gomock.Any(), stakingtypes.BondedPoolName).Return(bonedPool).AnyTimes()
	bk.EXPECT().GetBalance(gomock.Any(), bonedPool.GetAddress(), sdk.DefaultBondDenom).Return(sdk.NewInt64Coin(sdk.DefaultBondDenom, 1000000)).AnyTimes()

	for i, power := range powers {
		valAddr := sdk.ValAddress(PKs[i].Address().Bytes())
		validators[i] = testutil.NewValidator(s.T(), valAddr, PKs[i])
		tokens := keeper.TokensFromConsensusPower(ctx, power)

		validators[i], _ = validators[i].AddTokensFromDel(tokens)
		require.NoError(keeper.SetValidator(ctx, validators[i]))
		require.NoError(keeper.SetValidatorByPowerIndex(ctx, validators[i]))
		require.NoError(keeper.SetValidatorByConsAddr(ctx, validators[i]))

		s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
		updates := s.applyValidatorSetUpdates(ctx, keeper, 1)
		validator, err := keeper.GetValidator(ctx, valAddr)
		require.NoError(err)
		require.Equal(validator.ModuleValidatorUpdate(keeper.PowerReduction(ctx)), updates[0])
	}

	params, err := keeper.Params.Get(ctx)
	require.NoError(err)

	params.KeyRotationFee = sdk.NewInt64Coin(sdk.DefaultBondDenom, 1000)
	err = keeper.Params.Set(ctx, params)
	require.NoError(err)

	valAddr1 := sdk.ValAddress(PKs[0].Address().Bytes())

	valStr, err := keeper.ValidatorAddressCodec().BytesToString(valAddr1)
	require.NoError(err)

	msg, err := stakingtypes.NewMsgRotateConsPubKey(
		valStr,
		PKs[499], // taking the last element from PKs
	)

	require.NoError(err)

	bk.EXPECT().SendCoinsFromAccountToModule(ctx, sdk.AccAddress(valAddr1), gomock.Any(), gomock.Any()).AnyTimes()
	_, err = msgServer.RotateConsPubKey(ctx, msg)
	require.NoError(err)

	updates := s.applyValidatorSetUpdates(ctx, keeper, 2)

	originalPubKey, err := validators[0].ConsPubKey()
	require.NoError(err)

	validator, err := keeper.GetValidator(ctx, valAddr1)
	require.NoError(err)

	newPubKey, err := validator.ConsPubKey()
	require.NoError(err)

	require.Equal(int64(0), updates[0].Power)
	require.Equal(originalPubKey.Bytes(), updates[0].PubKey)
	require.Equal(int64(10), updates[1].Power)
	require.Equal(newPubKey.Bytes(), updates[1].PubKey)
}
