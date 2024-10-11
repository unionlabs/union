package keeper_test

import (
	"time"

	"github.com/golang/mock/gomock"

	"cosmossdk.io/collections"
	coreheader "cosmossdk.io/core/header"
	"cosmossdk.io/math"
	stakingkeeper "cosmossdk.io/x/staking/keeper"
	"cosmossdk.io/x/staking/testutil"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func createValAddrs(count int) ([]sdk.AccAddress, []sdk.ValAddress) {
	addrs := simtestutil.CreateIncrementalAccounts(count)
	valAddrs := simtestutil.ConvertAddrsToValAddrs(addrs)

	return addrs, valAddrs
}

func (s *KeeperTestSuite) TestSharesToTokensConversion() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, valAddrs := createValAddrs(1)

	s.accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	initialTokens := math.NewInt(1000000)
	validator := testutil.NewValidator(s.T(), valAddrs[0], PKs[0])
	validator, issuedShares := validator.AddTokensFromDel(initialTokens)
	require.NoError(keeper.SetValidator(ctx, validator))

	// Delegate tokens
	delegation := stakingtypes.NewDelegation(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, delegation))

	// Re-get the validator after delegation
	validator, err := keeper.GetValidator(ctx, valAddrs[0])
	require.NoError(err)

	// Convert shares to tokens
	shares := math.LegacyNewDecFromInt(initialTokens)
	tokens := validator.TokensFromSharesTruncated(shares)
	require.Equal(initialTokens, tokens.RoundInt())

	// Convert tokens back to shares
	newShares, err := validator.SharesFromTokens(initialTokens)
	require.NoError(err)
	require.True(shares.Equal(newShares))
}

// tests GetDelegation, GetDelegatorDelegations, SetDelegation, RemoveDelegation, GetDelegatorDelegations
func (s *KeeperTestSuite) TestDelegation() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, valAddrs := createValAddrs(3)

	s.accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	// construct the validators
	amts := []math.Int{math.NewInt(9), math.NewInt(8), math.NewInt(7)}
	var validators [3]stakingtypes.Validator
	for i, amt := range amts {
		validators[i] = testutil.NewValidator(s.T(), valAddrs[i], PKs[i])
		validators[i], _ = validators[i].AddTokensFromDel(amt)

		validators[i] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[i], true)
	}

	// first add a validators[0] to delegate too
	bond1to1 := stakingtypes.NewDelegation(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[0]), math.LegacyNewDec(9))

	// check the empty keeper first
	_, err := keeper.Delegations.Get(ctx, collections.Join(addrDels[0], valAddrs[0]))
	require.ErrorIs(err, collections.ErrNotFound)

	// set and retrieve a record
	require.NoError(keeper.SetDelegation(ctx, bond1to1))
	resBond, err := keeper.Delegations.Get(ctx, collections.Join(addrDels[0], valAddrs[0]))
	require.NoError(err)
	require.Equal(bond1to1, resBond)

	// modify a records, save, and retrieve
	bond1to1.Shares = math.LegacyNewDec(99)
	require.NoError(keeper.SetDelegation(ctx, bond1to1))
	resBond, err = keeper.Delegations.Get(ctx, collections.Join(addrDels[0], valAddrs[0]))
	require.NoError(err)
	require.Equal(bond1to1, resBond)

	// add some more records
	bond1to2 := stakingtypes.NewDelegation(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[1]), math.LegacyNewDec(9))
	bond1to3 := stakingtypes.NewDelegation(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[2]), math.LegacyNewDec(9))
	bond2to1 := stakingtypes.NewDelegation(s.addressToString(addrDels[1]), s.valAddressToString(valAddrs[0]), math.LegacyNewDec(9))
	bond2to2 := stakingtypes.NewDelegation(s.addressToString(addrDels[1]), s.valAddressToString(valAddrs[1]), math.LegacyNewDec(9))
	bond2to3 := stakingtypes.NewDelegation(s.addressToString(addrDels[1]), s.valAddressToString(valAddrs[2]), math.LegacyNewDec(9))
	require.NoError(keeper.SetDelegation(ctx, bond1to2))
	require.NoError(keeper.SetDelegation(ctx, bond1to3))
	require.NoError(keeper.SetDelegation(ctx, bond2to1))
	require.NoError(keeper.SetDelegation(ctx, bond2to2))
	require.NoError(keeper.SetDelegation(ctx, bond2to3))

	// test all bond retrieve capabilities
	resBonds, err := keeper.GetDelegatorDelegations(ctx, addrDels[0], 5)
	require.NoError(err)
	require.Equal(3, len(resBonds))
	require.Equal(bond1to1, resBonds[0])
	require.Equal(bond1to2, resBonds[1])
	require.Equal(bond1to3, resBonds[2])
	resBonds, err = keeper.GetAllDelegatorDelegations(ctx, addrDels[0])
	require.NoError(err)
	require.Equal(3, len(resBonds))
	resBonds, err = keeper.GetDelegatorDelegations(ctx, addrDels[0], 2)
	require.NoError(err)
	require.Equal(2, len(resBonds))
	resBonds, err = keeper.GetDelegatorDelegations(ctx, addrDels[1], 5)
	require.NoError(err)
	require.Equal(3, len(resBonds))
	require.Equal(bond2to1, resBonds[0])
	require.Equal(bond2to2, resBonds[1])
	require.Equal(bond2to3, resBonds[2])
	allBonds, err := keeper.GetAllDelegations(ctx)
	require.NoError(err)
	require.Equal(6, len(allBonds))
	require.Equal(bond1to1, allBonds[0])
	require.Equal(bond1to2, allBonds[1])
	require.Equal(bond1to3, allBonds[2])
	require.Equal(bond2to1, allBonds[3])
	require.Equal(bond2to2, allBonds[4])
	require.Equal(bond2to3, allBonds[5])

	resVals, err := keeper.GetDelegatorValidators(ctx, addrDels[0], 3)
	require.NoError(err)
	require.Equal(3, len(resVals.Validators))
	resVals, err = keeper.GetDelegatorValidators(ctx, addrDels[1], 4)
	require.NoError(err)
	require.Equal(3, len(resVals.Validators))

	for i := 0; i < 3; i++ {
		resVal, err := keeper.GetDelegatorValidator(ctx, addrDels[0], valAddrs[i])
		require.Nil(err)
		require.Equal(s.valAddressToString(valAddrs[i]), resVal.GetOperator())

		resVal, err = keeper.GetDelegatorValidator(ctx, addrDels[1], valAddrs[i])
		require.Nil(err)
		require.Equal(s.valAddressToString(valAddrs[i]), resVal.GetOperator())

		resDels, err := keeper.GetValidatorDelegations(ctx, valAddrs[i])
		require.NoError(err)
		require.Len(resDels, 2)
	}

	// test total bonded for single delegator
	expBonded := bond1to1.Shares.Add(bond2to1.Shares).Add(bond1to3.Shares)
	resDelBond, err := keeper.GetDelegatorBonded(ctx, addrDels[0])
	require.NoError(err)
	require.Equal(expBonded, math.LegacyNewDecFromInt(resDelBond))

	// delete a record
	require.NoError(keeper.RemoveDelegation(ctx, bond2to3))
	_, err = keeper.Delegations.Get(ctx, collections.Join(addrDels[1], valAddrs[2]))
	require.ErrorIs(err, collections.ErrNotFound)
	resBonds, err = keeper.GetDelegatorDelegations(ctx, addrDels[1], 5)
	require.NoError(err)
	require.Equal(2, len(resBonds))
	require.Equal(bond2to1, resBonds[0])
	require.Equal(bond2to2, resBonds[1])

	resBonds, err = keeper.GetAllDelegatorDelegations(ctx, addrDels[1])
	require.NoError(err)
	require.Equal(2, len(resBonds))

	// delete all the records from delegator 2
	require.NoError(keeper.RemoveDelegation(ctx, bond2to1))
	require.NoError(keeper.RemoveDelegation(ctx, bond2to2))
	_, err = keeper.Delegations.Get(ctx, collections.Join(addrDels[1], valAddrs[0]))
	require.ErrorIs(err, collections.ErrNotFound)
	_, err = keeper.Delegations.Get(ctx, collections.Join(addrDels[1], valAddrs[1]))
	require.ErrorIs(err, collections.ErrNotFound)
	resBonds, err = keeper.GetDelegatorDelegations(ctx, addrDels[1], 5)
	require.NoError(err)
	require.Equal(0, len(resBonds))
}

func (s *KeeperTestSuite) TestDelegationsByValIndex() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, valAddrs := createValAddrs(3)

	for _, addr := range addrDels {
		s.bankKeeper.EXPECT().DelegateCoinsFromAccountToModule(gomock.Any(), addr, gomock.Any(), gomock.Any()).Return(nil).AnyTimes()
	}
	s.accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	// construct the validators
	amts := []math.Int{math.NewInt(9), math.NewInt(8), math.NewInt(7)}
	var validators [3]stakingtypes.Validator
	for i, amt := range amts {
		validators[i] = testutil.NewValidator(s.T(), valAddrs[i], PKs[i])
		validators[i], _ = validators[i].AddTokensFromDel(amt)

		validators[i] = stakingkeeper.TestingUpdateValidator(keeper, ctx, validators[i], true)
	}

	// delegate 2 tokens
	//
	// total delegations after delegating: del1 -> 2stake
	_, err := s.msgServer.Delegate(ctx, stakingtypes.NewMsgDelegate(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[0]), sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(2))))
	require.NoError(err)

	dels, err := s.stakingKeeper.GetValidatorDelegations(ctx, valAddrs[0])
	require.NoError(err)
	require.Len(dels, 1)

	// delegate 4 tokens
	//
	// total delegations after delegating: del1 -> 2stake, del2 -> 4stake
	_, err = s.msgServer.Delegate(ctx, stakingtypes.NewMsgDelegate(s.addressToString(addrDels[1]), s.valAddressToString(valAddrs[0]), sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(4))))
	require.NoError(err)

	dels, err = s.stakingKeeper.GetValidatorDelegations(ctx, valAddrs[0])
	require.NoError(err)
	require.Len(dels, 2)

	// undelegate 1 token from del1
	//
	// total delegations after undelegating: del1 -> 1stake, del2 -> 4stake
	_, err = s.msgServer.Undelegate(ctx, stakingtypes.NewMsgUndelegate(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[0]), sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(1))))
	require.NoError(err)

	dels, err = s.stakingKeeper.GetValidatorDelegations(ctx, valAddrs[0])
	require.NoError(err)
	require.Len(dels, 2)

	// undelegate 1 token from del1
	//
	// total delegations after undelegating: del2 -> 4stake
	_, err = s.msgServer.Undelegate(ctx, stakingtypes.NewMsgUndelegate(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[0]), sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(1))))
	require.NoError(err)

	dels, err = s.stakingKeeper.GetValidatorDelegations(ctx, valAddrs[0])
	require.NoError(err)
	require.Len(dels, 1)

	// undelegate 2 tokens from del2
	//
	// total delegations after undelegating: del2 -> 2stake
	_, err = s.msgServer.Undelegate(ctx, stakingtypes.NewMsgUndelegate(s.addressToString(addrDels[1]), s.valAddressToString(valAddrs[0]), sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(2))))
	require.NoError(err)

	dels, err = s.stakingKeeper.GetValidatorDelegations(ctx, valAddrs[0])
	require.NoError(err)
	require.Len(dels, 1)

	// undelegate 2 tokens from del2
	//
	// total delegations after undelegating: []
	_, err = s.msgServer.Undelegate(ctx, stakingtypes.NewMsgUndelegate(s.addressToString(addrDels[1]), s.valAddressToString(valAddrs[0]), sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(2))))
	require.NoError(err)

	dels, err = s.stakingKeeper.GetValidatorDelegations(ctx, valAddrs[0])
	require.NoError(err)
	require.Len(dels, 0)
}

// tests Get/Set/Remove UnbondingDelegation
func (s *KeeperTestSuite) TestUnbondingDelegation() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	delAddrs, valAddrs := createValAddrs(2)

	s.accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	ubd := stakingtypes.NewUnbondingDelegation(
		delAddrs[0],
		valAddrs[0],
		0,
		time.Unix(0, 0).UTC(),
		math.NewInt(5),
		0,
		address.NewBech32Codec("cosmosvaloper"), address.NewBech32Codec("cosmos"),
	)

	// set and retrieve a record
	require.NoError(keeper.SetUnbondingDelegation(ctx, ubd))
	resUnbond, err := keeper.GetUnbondingDelegation(ctx, delAddrs[0], valAddrs[0])
	require.NoError(err)
	require.Equal(ubd, resUnbond)

	// modify a records, save, and retrieve
	expUnbond := math.NewInt(21)
	ubd.Entries[0].Balance = expUnbond
	require.NoError(keeper.SetUnbondingDelegation(ctx, ubd))

	resUnbonds, err := keeper.GetUnbondingDelegations(ctx, delAddrs[0], 5)
	require.NoError(err)
	require.Equal(1, len(resUnbonds))

	resUnbonds, err = keeper.GetAllUnbondingDelegations(ctx, delAddrs[0])
	require.NoError(err)
	require.Equal(1, len(resUnbonds))

	resUnbond, err = keeper.GetUnbondingDelegation(ctx, delAddrs[0], valAddrs[0])
	require.NoError(err)
	require.Equal(ubd, resUnbond)

	resDelUnbond, err := keeper.GetDelegatorUnbonding(ctx, delAddrs[0])
	require.NoError(err)
	require.Equal(expUnbond, resDelUnbond)

	// delete a record
	require.NoError(keeper.RemoveUnbondingDelegation(ctx, ubd))
	_, err = keeper.GetUnbondingDelegation(ctx, delAddrs[0], valAddrs[0])
	require.ErrorIs(err, stakingtypes.ErrNoUnbondingDelegation)

	resUnbonds, err = keeper.GetUnbondingDelegations(ctx, delAddrs[0], 5)
	require.NoError(err)
	require.Equal(0, len(resUnbonds))

	resUnbonds, err = keeper.GetAllUnbondingDelegations(ctx, delAddrs[0])
	require.NoError(err)
	require.Equal(0, len(resUnbonds))
}

func (s *KeeperTestSuite) TestUnbondingDelegationsFromValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	delAddrs, valAddrs := createValAddrs(2)

	ubd := stakingtypes.NewUnbondingDelegation(
		delAddrs[0],
		valAddrs[0],
		0,
		time.Unix(0, 0).UTC(),
		math.NewInt(5),
		0,
		address.NewBech32Codec("cosmosvaloper"), address.NewBech32Codec("cosmos"),
	)

	// set and retrieve a record
	require.NoError(keeper.SetUnbondingDelegation(ctx, ubd))
	resUnbond, err := keeper.GetUnbondingDelegation(ctx, delAddrs[0], valAddrs[0])
	require.NoError(err)
	require.Equal(ubd, resUnbond)

	// modify a records, save, and retrieve
	expUnbond := math.NewInt(21)
	ubd.Entries[0].Balance = expUnbond
	require.NoError(keeper.SetUnbondingDelegation(ctx, ubd))

	resUnbonds, err := keeper.GetUnbondingDelegations(ctx, delAddrs[0], 5)
	require.NoError(err)
	require.Equal(1, len(resUnbonds))

	resUnbonds, err = keeper.GetAllUnbondingDelegations(ctx, delAddrs[0])
	require.NoError(err)
	require.Equal(1, len(resUnbonds))

	resUnbonds, err = keeper.GetUnbondingDelegationsFromValidator(ctx, valAddrs[0])
	require.NoError(err)
	require.Equal(1, len(resUnbonds))

	resUnbond, err = keeper.GetUnbondingDelegation(ctx, delAddrs[0], valAddrs[0])
	require.NoError(err)
	require.Equal(ubd, resUnbond)

	resDelUnbond, err := keeper.GetDelegatorUnbonding(ctx, delAddrs[0])
	require.NoError(err)
	require.Equal(expUnbond, resDelUnbond)

	// delete a record
	require.NoError(keeper.RemoveUnbondingDelegation(ctx, ubd))
	_, err = keeper.GetUnbondingDelegation(ctx, delAddrs[0], valAddrs[0])
	require.ErrorIs(err, stakingtypes.ErrNoUnbondingDelegation)

	resUnbonds, err = keeper.GetUnbondingDelegations(ctx, delAddrs[0], 5)
	require.NoError(err)
	require.Equal(0, len(resUnbonds))

	resUnbonds, err = keeper.GetAllUnbondingDelegations(ctx, delAddrs[0])
	require.NoError(err)
	require.Equal(0, len(resUnbonds))

	resUnbonds, err = keeper.GetUnbondingDelegationsFromValidator(ctx, valAddrs[0])
	require.NoError(err)
	require.Equal(0, len(resUnbonds))
}

func (s *KeeperTestSuite) TestUnbondDelegation() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	delAddrs, valAddrs := createValAddrs(1)

	startTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator := testutil.NewValidator(s.T(), valAddrs[0], PKs[0])

	validator, issuedShares := validator.AddTokensFromDel(startTokens)
	require.Equal(startTokens, issuedShares.RoundInt())

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	_ = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)

	delegation := stakingtypes.NewDelegation(s.addressToString(delAddrs[0]), s.valAddressToString(valAddrs[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, delegation))

	bondTokens := keeper.TokensFromConsensusPower(ctx, 6)
	amount, err := keeper.Unbond(ctx, delAddrs[0], valAddrs[0], math.LegacyNewDecFromInt(bondTokens))
	require.NoError(err)
	require.Equal(bondTokens, amount) // shares to be added to an unbonding delegation

	delegation, err = keeper.Delegations.Get(ctx, collections.Join(delAddrs[0], valAddrs[0]))
	require.NoError(err)
	validator, err = keeper.GetValidator(ctx, valAddrs[0])
	require.NoError(err)

	remainingTokens := startTokens.Sub(bondTokens)

	require.Equal(remainingTokens, delegation.Shares.RoundInt())
	require.Equal(remainingTokens, validator.BondedTokens())
}

// // test undelegating self delegation from a validator pushing it below MinSelfDelegation
// // shift it from the bonded to unbonding state and jailed
func (s *KeeperTestSuite) TestUndelegateSelfDelegationBelowMinSelfDelegation() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, valAddrs := createValAddrs(1)
	delTokens := keeper.TokensFromConsensusPower(ctx, 10)

	// create a validator with a self-delegation
	validator := testutil.NewValidator(s.T(), valAddrs[0], PKs[0])

	validator.MinSelfDelegation = delTokens
	validator, issuedShares := validator.AddTokensFromDel(delTokens)
	require.Equal(delTokens, issuedShares.RoundInt())

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))
	require.True(validator.IsBonded())

	selfDelegation := stakingtypes.NewDelegation(s.addressToString(valAddrs[0]), s.valAddressToString(valAddrs[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, selfDelegation))

	// create a second delegation to this validator
	require.NoError(keeper.DeleteValidatorByPowerIndex(ctx, validator))
	validator, issuedShares = validator.AddTokensFromDel(delTokens)
	require.True(validator.IsBonded())
	require.Equal(delTokens, issuedShares.RoundInt())

	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	delegation := stakingtypes.NewDelegation(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, delegation))

	val0AccAddr := sdk.AccAddress(valAddrs[0].Bytes())
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	_, _, err := keeper.Undelegate(ctx, val0AccAddr, valAddrs[0], math.LegacyNewDecFromInt(keeper.TokensFromConsensusPower(ctx, 6)))
	require.NoError(err)

	// end block
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	s.applyValidatorSetUpdates(ctx, keeper, 1)

	validator, err = keeper.GetValidator(ctx, valAddrs[0])
	require.NoError(err)
	require.Equal(keeper.TokensFromConsensusPower(ctx, 14), validator.Tokens)
	require.Equal(stakingtypes.Unbonding, validator.Status)
	require.True(validator.Jailed)
}

func (s *KeeperTestSuite) TestUndelegateFromUnbondingValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()
	delTokens := keeper.TokensFromConsensusPower(ctx, 10)

	addrDels, addrVals := createValAddrs(2)

	// create a validator with a self-delegation
	validator := testutil.NewValidator(s.T(), addrVals[0], PKs[0])
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))

	validator, issuedShares := validator.AddTokensFromDel(delTokens)
	require.Equal(delTokens, issuedShares.RoundInt())

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	require.True(validator.IsBonded())

	selfDelegation := stakingtypes.NewDelegation(s.addressToString(addrDels[0]), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, selfDelegation))

	// create a second delegation to this validator
	require.NoError(keeper.DeleteValidatorByPowerIndex(ctx, validator))

	validator, issuedShares = validator.AddTokensFromDel(delTokens)
	require.Equal(delTokens, issuedShares.RoundInt())

	stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	delegation := stakingtypes.NewDelegation(s.addressToString(addrDels[1]), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, delegation))

	header := ctx.HeaderInfo()
	blockHeight := int64(10)
	header.Height = blockHeight
	blockTime := time.Unix(333, 0)
	header.Time = blockTime
	ctx = ctx.WithHeaderInfo(header)

	// unbond the all self-delegation to put validator in unbonding state
	val0AccAddr := sdk.AccAddress(addrVals[0])
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	_, amount, err := keeper.Undelegate(ctx, val0AccAddr, addrVals[0], math.LegacyNewDecFromInt(delTokens))
	require.NoError(err)
	require.Equal(amount, delTokens)

	// end block
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	s.applyValidatorSetUpdates(ctx, keeper, 1)

	validator, err = keeper.GetValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(blockHeight, validator.UnbondingHeight)
	params, err := keeper.Params.Get(ctx)
	require.NoError(err)
	require.True(blockTime.Add(params.UnbondingTime).Equal(validator.UnbondingTime))

	blockHeight2 := int64(20)
	blockTime2 := time.Unix(444, 0).UTC()
	ctx = ctx.WithBlockHeight(blockHeight2)
	ctx = ctx.WithHeaderInfo(coreheader.Info{Height: blockHeight2, Time: blockTime2})

	// unbond some of the other delegation's shares
	undelegateAmount := math.LegacyNewDec(6)
	_, undelegatedAmount, err := keeper.Undelegate(ctx, addrDels[1], addrVals[0], undelegateAmount)
	require.NoError(err)
	require.Equal(math.LegacyNewDecFromInt(undelegatedAmount), undelegateAmount)

	// retrieve the unbonding delegation
	ubd, err := keeper.GetUnbondingDelegation(ctx, addrDels[1], addrVals[0])
	require.NoError(err)
	require.Len(ubd.Entries, 1)
	require.True(ubd.Entries[0].Balance.Equal(math.NewInt(6)))
	require.Equal(blockHeight2, ubd.Entries[0].CreationHeight)
	require.True(blockTime2.Add(params.UnbondingTime).Equal(ubd.Entries[0].CompletionTime))
}

// TestUndelegateFromUnbondedValidator tests the undelegation process from an unbonded validator.
// It creates a validator with a self-delegation and a second delegation to the same validator.
// Then it unbonds the self-delegation to put the validator in the unbonding state.
// Finally, it unbonds the remaining shares of the second delegation and verifies that the validator is deleted from the state.
func (s *KeeperTestSuite) TestUndelegateFromUnbondedValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	delTokens := keeper.TokensFromConsensusPower(ctx, 10)
	addrDels, addrVals := createValAddrs(2)

	// create a validator with a self-delegation
	validator := testutil.NewValidator(s.T(), addrVals[0], PKs[0])
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))

	valTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator, issuedShares := validator.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	require.True(validator.IsBonded())

	val0AccAddr := sdk.AccAddress(addrVals[0])
	selfDelegation := stakingtypes.NewDelegation(s.addressToString(val0AccAddr), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, selfDelegation))

	// create a second delegation to this validator
	require.NoError(keeper.DeleteValidatorByPowerIndex(ctx, validator))
	validator, issuedShares = validator.AddTokensFromDel(delTokens)
	require.Equal(delTokens, issuedShares.RoundInt())
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	require.True(validator.IsBonded())
	delegation := stakingtypes.NewDelegation(s.addressToString(addrDels[1]), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, delegation))

	ctx = ctx.WithBlockHeight(10)
	ctx = ctx.WithHeaderInfo(coreheader.Info{Height: 10, Time: time.Unix(333, 0)})

	// unbond the all self-delegation to put validator in unbonding state
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	_, amount, err := keeper.Undelegate(ctx, val0AccAddr, addrVals[0], math.LegacyNewDecFromInt(valTokens))
	require.NoError(err)
	require.Equal(amount, valTokens)

	// end block
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	s.applyValidatorSetUpdates(ctx, keeper, 1)

	validator, err = keeper.GetValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(ctx.BlockHeight(), validator.UnbondingHeight)
	params, err := keeper.Params.Get(ctx)
	require.NoError(err)
	require.True(ctx.HeaderInfo().Time.Add(params.UnbondingTime).Equal(validator.UnbondingTime))

	// unbond the validator
	ctx = ctx.WithHeaderInfo(coreheader.Info{Height: 10, Time: validator.UnbondingTime})
	err = keeper.UnbondAllMatureValidators(ctx)
	require.NoError(err)

	// Make sure validator is still in state because there is still an outstanding delegation
	validator, err = keeper.GetValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(validator.Status, stakingtypes.Unbonded)

	// unbond some of the other delegation's shares
	unbondTokens := keeper.TokensFromConsensusPower(ctx, 6)
	_, amount2, err := keeper.Undelegate(ctx, addrDels[1], addrVals[0], math.LegacyNewDecFromInt(unbondTokens))
	require.NoError(err)
	require.Equal(amount2, unbondTokens)

	// unbond rest of the other delegation's shares
	remainingTokens := delTokens.Sub(unbondTokens)
	_, amount3, err := keeper.Undelegate(ctx, addrDels[1], addrVals[0], math.LegacyNewDecFromInt(remainingTokens))
	require.NoError(err)
	require.Equal(amount3, remainingTokens)

	//  now validator should be deleted from state
	validator, err = keeper.GetValidator(ctx, addrVals[0])
	require.ErrorIs(err, stakingtypes.ErrNoValidatorFound)
}

// TestUnbondingAllDelegationFromValidator tests the process of unbonding all delegations from a validator.
// It creates a validator with a self-delegation and a second delegation, then unbonds all the delegations
// to put the validator in an unbonding state. Finally, it verifies that the validator is deleted from the state.
func (s *KeeperTestSuite) TestUnbondingAllDelegationFromValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	delTokens := keeper.TokensFromConsensusPower(ctx, 10)
	addrDels, addrVals := createValAddrs(2)

	// create a validator with a self-delegation
	validator := testutil.NewValidator(s.T(), addrVals[0], PKs[0])
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))

	valTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator, issuedShares := validator.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	require.True(validator.IsBonded())
	val0AccAddr := sdk.AccAddress(addrVals[0].Bytes())

	selfDelegation := stakingtypes.NewDelegation(s.addressToString(val0AccAddr), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, selfDelegation))

	// create a second delegation to this validator
	require.NoError(keeper.DeleteValidatorByPowerIndex(ctx, validator))
	validator, issuedShares = validator.AddTokensFromDel(delTokens)
	require.Equal(delTokens, issuedShares.RoundInt())

	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	require.True(validator.IsBonded())

	delegation := stakingtypes.NewDelegation(s.addressToString(addrDels[1]), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, delegation))

	ctx = ctx.WithHeaderInfo(coreheader.Info{Height: 10, Time: time.Unix(333, 0)})

	// unbond the all self-delegation to put validator in unbonding state
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	_, amount, err := keeper.Undelegate(ctx, val0AccAddr, addrVals[0], math.LegacyNewDecFromInt(valTokens))
	require.NoError(err)
	require.Equal(amount, valTokens)

	// end block
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	s.applyValidatorSetUpdates(ctx, keeper, 1)

	// unbond all the remaining delegation
	_, amount2, err := keeper.Undelegate(ctx, addrDels[1], addrVals[0], math.LegacyNewDecFromInt(delTokens))
	require.NoError(err)
	require.Equal(amount2, delTokens)

	// validator should still be in state and still be in unbonding state
	validator, err = keeper.GetValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(validator.Status, stakingtypes.Unbonding)

	// unbond the validator
	ctx = ctx.WithHeaderInfo(coreheader.Info{Height: 10, Time: validator.UnbondingTime})
	err = keeper.UnbondAllMatureValidators(ctx)
	require.NoError(err)

	// validator should now be deleted from state
	_, err = keeper.GetValidator(ctx, addrVals[0])
	require.ErrorIs(err, stakingtypes.ErrNoValidatorFound)
}

// Make sure that the retrieving the delegations doesn't affect the state
func (s *KeeperTestSuite) TestGetRedelegationsFromSrcValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, addrVals := createValAddrs(2)

	rd := stakingtypes.NewRedelegation(addrDels[0], addrVals[0], addrVals[1], 0,
		time.Unix(0, 0), math.NewInt(5),
		math.LegacyNewDec(5), 0, address.NewBech32Codec("cosmosvaloper"), address.NewBech32Codec("cosmos"))

	// set and retrieve a record
	err := keeper.SetRedelegation(ctx, rd)
	require.NoError(err)
	resBond, err := keeper.Redelegations.Get(ctx, collections.Join3(addrDels[0].Bytes(), addrVals[0].Bytes(), addrVals[1].Bytes()))
	require.NoError(err)

	// get the redelegations one time
	redelegations, err := keeper.GetRedelegationsFromSrcValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(1, len(redelegations))
	require.Equal(redelegations[0], resBond)

	// get the redelegations a second time, should be exactly the same
	redelegations, err = keeper.GetRedelegationsFromSrcValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(1, len(redelegations))
	require.Equal(redelegations[0], resBond)
}

// tests Get/Set/Remove/Has UnbondingDelegation
func (s *KeeperTestSuite) TestRedelegation() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, addrVals := createValAddrs(2)

	rd := stakingtypes.NewRedelegation(addrDels[0], addrVals[0], addrVals[1], 0,
		time.Unix(0, 0).UTC(), math.NewInt(5),
		math.LegacyNewDec(5), 0, address.NewBech32Codec("cosmosvaloper"), address.NewBech32Codec("cosmos"))

	// test shouldn't have and redelegations
	has, err := keeper.HasReceivingRedelegation(ctx, addrDels[0], addrVals[1])
	require.NoError(err)
	require.False(has)

	// set and retrieve a record
	err = keeper.SetRedelegation(ctx, rd)
	require.NoError(err)
	resRed, err := keeper.Redelegations.Get(ctx, collections.Join3(addrDels[0].Bytes(), addrVals[0].Bytes(), addrVals[1].Bytes()))
	require.NoError(err)

	redelegations, err := keeper.GetRedelegationsFromSrcValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(1, len(redelegations))
	require.Equal(redelegations[0], resRed)

	redelegations, err = keeper.GetRedelegations(ctx, addrDels[0], 5)
	require.NoError(err)
	require.Equal(1, len(redelegations))
	require.Equal(redelegations[0], resRed)

	redelegations, err = keeper.GetAllRedelegations(ctx, addrDels[0], nil, nil)
	require.NoError(err)
	require.Equal(1, len(redelegations))
	require.Equal(redelegations[0], resRed)

	// check if it has the redelegation
	has, err = keeper.HasReceivingRedelegation(ctx, addrDels[0], addrVals[1])
	require.NoError(err)
	require.True(has)

	// modify a records, save, and retrieve
	rd.Entries[0].SharesDst = math.LegacyNewDec(21)
	err = keeper.SetRedelegation(ctx, rd)
	require.NoError(err)

	resRed, err = keeper.Redelegations.Get(ctx, collections.Join3(addrDels[0].Bytes(), addrVals[0].Bytes(), addrVals[1].Bytes()))
	require.NoError(err)
	require.Equal(rd, resRed)

	redelegations, err = keeper.GetRedelegationsFromSrcValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(1, len(redelegations))
	require.Equal(redelegations[0], resRed)

	redelegations, err = keeper.GetRedelegations(ctx, addrDels[0], 5)
	require.NoError(err)
	require.Equal(1, len(redelegations))
	require.Equal(redelegations[0], resRed)

	// delete a record
	err = keeper.RemoveRedelegation(ctx, rd)
	require.NoError(err)
	_, err = keeper.Redelegations.Get(ctx, collections.Join3(addrDels[0].Bytes(), addrVals[0].Bytes(), addrVals[1].Bytes()))
	require.ErrorIs(err, collections.ErrNotFound)

	redelegations, err = keeper.GetRedelegations(ctx, addrDels[0], 5)
	require.NoError(err)
	require.Equal(0, len(redelegations))

	redelegations, err = keeper.GetAllRedelegations(ctx, addrDels[0], nil, nil)
	require.NoError(err)
	require.Equal(0, len(redelegations))
}

func (s *KeeperTestSuite) TestRedelegateToSameValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	_, addrVals := createValAddrs(1)
	valTokens := keeper.TokensFromConsensusPower(ctx, 10)

	// create a validator with a self-delegation
	validator := testutil.NewValidator(s.T(), addrVals[0], PKs[0])
	validator, issuedShares := validator.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	require.True(validator.IsBonded())

	val0AccAddr := sdk.AccAddress(addrVals[0].Bytes())

	selfDelegation := stakingtypes.NewDelegation(s.addressToString(val0AccAddr), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, selfDelegation))

	_, err := keeper.BeginRedelegation(ctx, val0AccAddr, addrVals[0], addrVals[0], math.LegacyNewDec(5))
	require.Error(err)
}

func (s *KeeperTestSuite) TestRedelegationMaxEntries() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	_, addrVals := createValAddrs(2)

	// create a validator with a self-delegation
	validator := testutil.NewValidator(s.T(), addrVals[0], PKs[0])
	valTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator, issuedShares := validator.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	_ = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	val0AccAddr := sdk.AccAddress(addrVals[0].Bytes())
	selfDelegation := stakingtypes.NewDelegation(s.addressToString(val0AccAddr), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, selfDelegation))

	// create a second validator
	validator2 := testutil.NewValidator(s.T(), addrVals[1], PKs[1])
	validator2, issuedShares = validator2.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator2 = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator2, true)
	require.Equal(stakingtypes.Bonded, validator2.Status)

	maxEntries, err := keeper.MaxEntries(ctx)
	require.NoError(err)

	// redelegations should pass
	var completionTime time.Time
	for i := uint32(0); i < maxEntries; i++ {
		var err error
		completionTime, err = keeper.BeginRedelegation(ctx, val0AccAddr, addrVals[0], addrVals[1], math.LegacyNewDec(1))
		require.NoError(err)
	}

	// an additional redelegation should fail due to max entries
	_, err = keeper.BeginRedelegation(ctx, val0AccAddr, addrVals[0], addrVals[1], math.LegacyNewDec(1))
	require.Error(err)

	// mature redelegations
	ctx = ctx.WithHeaderInfo(coreheader.Info{Time: completionTime})
	_, err = keeper.CompleteRedelegation(ctx, val0AccAddr, addrVals[0], addrVals[1])
	require.NoError(err)

	// redelegation should work again
	_, err = keeper.BeginRedelegation(ctx, val0AccAddr, addrVals[0], addrVals[1], math.LegacyNewDec(1))
	require.NoError(err)
}

func (s *KeeperTestSuite) TestRedelegateSelfDelegation() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, addrVals := createValAddrs(2)

	// create a validator with a self-delegation
	validator := testutil.NewValidator(s.T(), addrVals[0], PKs[0])
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))

	valTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator, issuedShares := validator.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)

	val0AccAddr := sdk.AccAddress(addrVals[0])
	selfDelegation := stakingtypes.NewDelegation(s.addressToString(val0AccAddr), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, selfDelegation))

	// create a second validator
	validator2 := testutil.NewValidator(s.T(), addrVals[1], PKs[1])
	validator2, issuedShares = validator2.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator2 = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator2, true)
	require.Equal(stakingtypes.Bonded, validator2.Status)

	// create a second delegation to validator 1
	delTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator, issuedShares = validator.AddTokensFromDel(delTokens)
	require.Equal(delTokens, issuedShares.RoundInt())
	stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)

	delegation := stakingtypes.NewDelegation(s.addressToString(addrDels[0]), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, delegation))

	_, err := keeper.BeginRedelegation(ctx, val0AccAddr, addrVals[0], addrVals[1], math.LegacyNewDecFromInt(delTokens))
	require.NoError(err)

	// end block
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	s.applyValidatorSetUpdates(ctx, keeper, 2)

	validator, err = keeper.GetValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(valTokens, validator.Tokens)
	require.Equal(stakingtypes.Unbonding, validator.Status)
}

func (s *KeeperTestSuite) TestRedelegateFromUnbondingValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, addrVals := createValAddrs(2)

	// create a validator with a self-delegation
	validator := testutil.NewValidator(s.T(), addrVals[0], PKs[0])
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))

	valTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator, issuedShares := validator.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	val0AccAddr := sdk.AccAddress(addrVals[0].Bytes())
	selfDelegation := stakingtypes.NewDelegation(s.addressToString(val0AccAddr), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, selfDelegation))

	// create a second delegation to this validator
	require.NoError(keeper.DeleteValidatorByPowerIndex(ctx, validator))
	delTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator, issuedShares = validator.AddTokensFromDel(delTokens)
	require.Equal(delTokens, issuedShares.RoundInt())
	stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	delegation := stakingtypes.NewDelegation(s.addressToString(addrDels[1]), s.valAddressToString(addrVals[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, delegation))

	// create a second validator
	validator2 := testutil.NewValidator(s.T(), addrVals[1], PKs[1])
	validator2, issuedShares = validator2.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	_ = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator2, true)

	header := ctx.HeaderInfo()
	blockHeight := int64(10)
	header.Height = blockHeight
	blockTime := time.Unix(333, 0)
	header.Time = blockTime
	ctx = ctx.WithHeaderInfo(header)

	// unbond the all self-delegation to put validator in unbonding state
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	_, amount, err := keeper.Undelegate(ctx, val0AccAddr, addrVals[0], math.LegacyNewDecFromInt(delTokens))
	require.NoError(err)
	require.Equal(amount, delTokens)

	// end block
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	s.applyValidatorSetUpdates(ctx, keeper, 1)

	validator, err = keeper.GetValidator(ctx, addrVals[0])
	require.NoError(err)
	require.Equal(blockHeight, validator.UnbondingHeight)
	params, err := keeper.Params.Get(ctx)
	require.NoError(err)
	require.True(blockTime.Add(params.UnbondingTime).Equal(validator.UnbondingTime))

	// change the context
	header = ctx.HeaderInfo()
	blockHeight2 := int64(20)
	header.Height = blockHeight2
	blockTime2 := time.Unix(444, 0)
	header.Time = blockTime2
	ctx = ctx.WithHeaderInfo(header)

	// unbond some of the other delegation's shares
	redelegateTokens := keeper.TokensFromConsensusPower(ctx, 6)
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	_, err = keeper.BeginRedelegation(ctx, addrDels[1], addrVals[0], addrVals[1], math.LegacyNewDecFromInt(redelegateTokens))
	require.NoError(err)

	// retrieve the unbonding delegation
	ubd, err := keeper.Redelegations.Get(ctx, collections.Join3(addrDels[1].Bytes(), addrVals[0].Bytes(), addrVals[1].Bytes()))
	require.NoError(err)
	require.Len(ubd.Entries, 1)
	require.Equal(blockHeight, ubd.Entries[0].CreationHeight)
	require.True(blockTime.Add(params.UnbondingTime).Equal(ubd.Entries[0].CompletionTime))
}

func (s *KeeperTestSuite) TestRedelegateFromUnbondedValidator() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, valAddrs := createValAddrs(2)

	// create a validator with a self-delegation
	validator := testutil.NewValidator(s.T(), valAddrs[0], PKs[0])
	require.NoError(keeper.SetValidatorByConsAddr(ctx, validator))

	valTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator, issuedShares := validator.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	val0AccAddr := sdk.AccAddress(valAddrs[0].Bytes())
	selfDelegation := stakingtypes.NewDelegation(s.addressToString(val0AccAddr), s.valAddressToString(valAddrs[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, selfDelegation))

	// create a second delegation to this validator
	require.NoError(keeper.DeleteValidatorByPowerIndex(ctx, validator))
	delTokens := keeper.TokensFromConsensusPower(ctx, 10)
	validator, issuedShares = validator.AddTokensFromDel(delTokens)
	require.Equal(delTokens, issuedShares.RoundInt())
	stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	delegation := stakingtypes.NewDelegation(s.addressToString(addrDels[1]), s.valAddressToString(valAddrs[0]), issuedShares)
	require.NoError(keeper.SetDelegation(ctx, delegation))

	// create a second validator
	validator2 := testutil.NewValidator(s.T(), valAddrs[1], PKs[1])
	validator2, issuedShares = validator2.AddTokensFromDel(valTokens)
	require.Equal(valTokens, issuedShares.RoundInt())
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	validator2 = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator2, true)
	require.Equal(stakingtypes.Bonded, validator2.Status)

	ctx = ctx.WithBlockHeight(10)
	ctx = ctx.WithHeaderInfo(coreheader.Info{Height: 10, Time: time.Unix(333, 0)})

	// unbond the all self-delegation to put validator in unbonding state
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	_, amount, err := keeper.Undelegate(ctx, val0AccAddr, valAddrs[0], math.LegacyNewDecFromInt(delTokens))
	require.NoError(err)
	require.Equal(amount, delTokens)

	// end block
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.BondedPoolName, stakingtypes.NotBondedPoolName, gomock.Any())
	s.applyValidatorSetUpdates(ctx, keeper, 1)

	validator, err = keeper.GetValidator(ctx, valAddrs[0])
	require.NoError(err)
	require.Equal(ctx.HeaderInfo().Height, validator.UnbondingHeight)
	params, err := keeper.Params.Get(ctx)
	require.NoError(err)
	require.True(ctx.HeaderInfo().Time.Add(params.UnbondingTime).Equal(validator.UnbondingTime))

	// unbond the validator
	_, err = keeper.UnbondingToUnbonded(ctx, validator)
	require.NoError(err)

	// redelegate some of the delegation's shares
	redelegationTokens := keeper.TokensFromConsensusPower(ctx, 6)
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), stakingtypes.NotBondedPoolName, stakingtypes.BondedPoolName, gomock.Any())
	_, err = keeper.BeginRedelegation(ctx, addrDels[1], valAddrs[0], valAddrs[1], math.LegacyNewDecFromInt(redelegationTokens))
	require.NoError(err)

	// no red should have been found
	red, err := keeper.Redelegations.Get(ctx, collections.Join3(addrDels[0].Bytes(), valAddrs[0].Bytes(), valAddrs[1].Bytes()))
	require.ErrorIs(err, collections.ErrNotFound, "%v", red)
}

func (s *KeeperTestSuite) TestUnbondingDelegationAddEntry() {
	require := s.Require()

	delAddrs, valAddrs := createValAddrs(1)

	delAddr := delAddrs[0]
	valAddr := valAddrs[0]
	creationHeight := int64(10)
	ubd := stakingtypes.NewUnbondingDelegation(
		delAddr,
		valAddr,
		creationHeight,
		time.Unix(0, 0).UTC(),
		math.NewInt(10),
		0,
		address.NewBech32Codec("cosmosvaloper"), address.NewBech32Codec("cosmos"),
	)
	var initialEntries []stakingtypes.UnbondingDelegationEntry
	initialEntries = append(initialEntries, ubd.Entries...)
	require.Len(initialEntries, 1)

	isNew := ubd.AddEntry(creationHeight, time.Unix(0, 0).UTC(), math.NewInt(5), 1)
	require.False(isNew)
	require.Len(ubd.Entries, 1) // entry was merged
	require.NotEqual(initialEntries, ubd.Entries)
	require.Equal(creationHeight, ubd.Entries[0].CreationHeight)
	require.Equal(initialEntries[0].UnbondingId, ubd.Entries[0].UnbondingId) // unbondingID remains unchanged
	require.Equal(ubd.Entries[0].Balance, math.NewInt(15))                   // 10 from previous + 5 from merged

	newCreationHeight := int64(11)
	isNew = ubd.AddEntry(newCreationHeight, time.Unix(1, 0).UTC(), math.NewInt(5), 2)
	require.True(isNew)
	require.Len(ubd.Entries, 2) // entry was appended
	require.NotEqual(initialEntries, ubd.Entries)
	require.Equal(creationHeight, ubd.Entries[0].CreationHeight)
	require.Equal(newCreationHeight, ubd.Entries[1].CreationHeight)
	require.Equal(ubd.Entries[0].Balance, math.NewInt(15))
	require.Equal(ubd.Entries[1].Balance, math.NewInt(5))
	require.NotEqual(ubd.Entries[0].UnbondingId, ubd.Entries[1].UnbondingId) // appended entry has a new unbondingID
}

func (s *KeeperTestSuite) TestSetUnbondingDelegationEntry() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	delAddrs, valAddrs := createValAddrs(1)

	delAddr := delAddrs[0]
	valAddr := valAddrs[0]
	creationHeight := int64(0)
	ubd := stakingtypes.NewUnbondingDelegation(
		delAddr,
		valAddr,
		creationHeight,
		time.Unix(0, 0).UTC(),
		math.NewInt(5),
		0,
		address.NewBech32Codec("cosmosvaloper"), address.NewBech32Codec("cosmos"),
	)

	// set and retrieve a record
	require.NoError(keeper.SetUnbondingDelegation(ctx, ubd))
	resUnbond, err := keeper.GetUnbondingDelegation(ctx, delAddr, valAddr)
	require.NoError(err)
	require.Equal(ubd, resUnbond)

	initialEntries := ubd.Entries
	require.Len(initialEntries, 1)
	require.Equal(initialEntries[0].Balance, math.NewInt(5))
	require.Equal(initialEntries[0].UnbondingId, uint64(0)) // initial unbondingID

	// set unbonding delegation entry for existing creationHeight
	// entries are expected to be merged
	_, err = keeper.SetUnbondingDelegationEntry(
		ctx,
		delAddr,
		valAddr,
		creationHeight,
		time.Unix(0, 0).UTC(),
		math.NewInt(5),
	)
	require.NoError(err)
	resUnbonding, err := keeper.GetUnbondingDelegation(ctx, delAddr, valAddr)
	require.NoError(err)
	require.Len(resUnbonding.Entries, 1)
	require.NotEqual(initialEntries, resUnbonding.Entries)
	require.Equal(creationHeight, resUnbonding.Entries[0].CreationHeight)
	require.Equal(initialEntries[0].UnbondingId, resUnbonding.Entries[0].UnbondingId) // initial unbondingID remains unchanged
	require.Equal(resUnbonding.Entries[0].Balance, math.NewInt(10))                   // 5 from previous entry + 5 from merged entry

	// set unbonding delegation entry for newCreationHeight
	// new entry is expected to be appended to the existing entries
	newCreationHeight := int64(1)
	_, err = keeper.SetUnbondingDelegationEntry(
		ctx,
		delAddr,
		valAddr,
		newCreationHeight,
		time.Unix(1, 0).UTC(),
		math.NewInt(10),
	)
	require.NoError(err)
	resUnbonding, err = keeper.GetUnbondingDelegation(ctx, delAddr, valAddr)
	require.NoError(err)
	require.Len(resUnbonding.Entries, 2)
	require.NotEqual(initialEntries, resUnbonding.Entries)
	require.NotEqual(resUnbonding.Entries[0], resUnbonding.Entries[1])
	require.Equal(creationHeight, resUnbonding.Entries[0].CreationHeight)
	require.Equal(newCreationHeight, resUnbonding.Entries[1].CreationHeight)

	// unbondingID is incremented on every call to SetUnbondingDelegationEntry
	// unbondingID == 1 was skipped because the entry was merged with the existing entry with unbondingID == 0
	// unbondingID comes from a global counter -> gaps in unbondingIDs are OK as long as every unbondingID is unique
	require.Equal(uint64(2), resUnbonding.Entries[1].UnbondingId)
}

func (s *KeeperTestSuite) TestUndelegateWithDustShare() {
	ctx, keeper := s.ctx, s.stakingKeeper
	require := s.Require()

	addrDels, valAddrs := createValAddrs(2)

	s.accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	// construct the validators[0] & slash 1stake
	amt := math.NewInt(100)
	validator := testutil.NewValidator(s.T(), valAddrs[0], PKs[0])
	validator, _ = validator.AddTokensFromDel(amt)
	validator = validator.RemoveTokens(math.NewInt(1))
	validator = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)

	// first add a validators[0] to delegate too
	bond1to1 := stakingtypes.NewDelegation(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[0]), math.LegacyNewDec(100))
	require.NoError(keeper.SetDelegation(ctx, bond1to1))
	resBond, err := keeper.Delegations.Get(ctx, collections.Join(addrDels[0], valAddrs[0]))
	require.NoError(err)
	require.Equal(bond1to1, resBond)

	// second delegators[1] add a validators[0] to delegate
	bond2to1 := stakingtypes.NewDelegation(s.addressToString(addrDels[1]), s.valAddressToString(valAddrs[0]), math.LegacyNewDec(1))
	validator, delegatorShare := validator.AddTokensFromDel(math.NewInt(1))
	bond2to1.Shares = delegatorShare
	_ = stakingkeeper.TestingUpdateValidator(keeper, ctx, validator, true)
	require.NoError(keeper.SetDelegation(ctx, bond2to1))
	resBond, err = keeper.Delegations.Get(ctx, collections.Join(addrDels[1], valAddrs[0]))
	require.NoError(err)
	require.Equal(bond2to1, resBond)

	// check delegation state
	delegations, err := keeper.GetValidatorDelegations(ctx, valAddrs[0])
	require.NoError(err)
	require.Equal(2, len(delegations))

	// undelegate all delegator[0]'s delegate
	_, err = s.msgServer.Undelegate(ctx, stakingtypes.NewMsgUndelegate(s.addressToString(addrDels[0]), s.valAddressToString(valAddrs[0]), sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(99))))
	require.NoError(err)

	// remain only delegator[1]'s delegate
	delegations, err = keeper.GetValidatorDelegations(ctx, valAddrs[0])
	require.NoError(err)
	require.Equal(1, len(delegations))
	require.Equal(delegations[0].DelegatorAddress, s.addressToString(addrDels[1]))
}
