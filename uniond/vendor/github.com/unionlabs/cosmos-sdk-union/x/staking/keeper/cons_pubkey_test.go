package keeper_test

import (
	"time"

	"github.com/golang/mock/gomock"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/header"
	stakingkeeper "cosmossdk.io/x/staking/keeper"
	"cosmossdk.io/x/staking/testutil"
	"cosmossdk.io/x/staking/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

func (s *KeeperTestSuite) TestConsPubKeyRotationHistory() {
	stakingKeeper, ctx := s.stakingKeeper, s.ctx

	_, addrVals := createValAddrs(2)

	// create a validator with a self-delegation
	val := testutil.NewValidator(s.T(), addrVals[0], PKs[0])
	valTokens := stakingKeeper.TokensFromConsensusPower(ctx, 10)
	val, issuedShares := val.AddTokensFromDel(valTokens)
	s.Require().Equal(valTokens, issuedShares.RoundInt())

	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), types.NotBondedPoolName, types.BondedPoolName, gomock.Any())
	_ = stakingkeeper.TestingUpdateValidator(stakingKeeper, ctx, val, true)
	val0AccAddr, err := s.accountKeeper.AddressCodec().BytesToString(addrVals[0])
	s.Require().NoError(err)
	selfDelegation := types.NewDelegation(val0AccAddr, s.valAddressToString(addrVals[0]), issuedShares)

	err = stakingKeeper.SetDelegation(ctx, selfDelegation)
	s.Require().NoError(err)

	validators, err := stakingKeeper.GetAllValidators(ctx)
	s.Require().NoError(err)
	s.Require().Len(validators, 1)

	validator := validators[0]
	valAddr, err := s.stakingKeeper.ValidatorAddressCodec().StringToBytes(validator.OperatorAddress)
	s.Require().NoError(err)

	historyObjects, err := stakingKeeper.GetValidatorConsPubKeyRotationHistory(ctx, valAddr)
	s.Require().NoError(err)
	s.Require().Len(historyObjects, 0)

	newConsPub, err := codectypes.NewAnyWithValue(PKs[1])
	s.Require().NoError(err)

	newConsPub2, err := codectypes.NewAnyWithValue(PKs[2])
	s.Require().NoError(err)

	params, err := stakingKeeper.Params.Get(ctx)
	s.Require().NoError(err)

	height := uint64(ctx.BlockHeight())
	err = stakingKeeper.RotationHistory.Set(ctx, collections.Join(valAddr, height), types.ConsPubKeyRotationHistory{
		OperatorAddress: valAddr,
		OldConsPubkey:   validator.ConsensusPubkey,
		NewConsPubkey:   newConsPub,
		Height:          height,
		Fee:             params.KeyRotationFee,
	})
	s.Require().NoError(err)

	historyObjects, err = stakingKeeper.GetValidatorConsPubKeyRotationHistory(ctx, valAddr)
	s.Require().NoError(err)
	s.Require().Len(historyObjects, 1)

	historyObjects, err = stakingKeeper.GetBlockConsPubKeyRotationHistory(ctx)
	s.Require().NoError(err)
	s.Require().Len(historyObjects, 1)

	err = stakingKeeper.RotationHistory.Set(ctx, collections.Join(valAddr, height+1), types.ConsPubKeyRotationHistory{
		OperatorAddress: valAddr,
		OldConsPubkey:   newConsPub,
		NewConsPubkey:   newConsPub2,
		Height:          height + 1,
		Fee:             params.KeyRotationFee,
	})
	s.Require().NoError(err)

	historyObjects1, err := stakingKeeper.GetValidatorConsPubKeyRotationHistory(ctx, valAddr)
	s.Require().NoError(err)
	s.Require().Len(historyObjects1, 2)

	historyObjects, err = stakingKeeper.GetBlockConsPubKeyRotationHistory(ctx)
	s.Require().NoError(err)

	s.Require().Len(historyObjects, 1)
}

func (s *KeeperTestSuite) TestValidatorIdentifier() {
	stakingKeeper, ctx, accountKeeper, bankKeeper := s.stakingKeeper, s.ctx, s.accountKeeper, s.bankKeeper

	msgServer := stakingkeeper.NewMsgServerImpl(stakingKeeper)
	s.setValidators(6)
	validators, err := stakingKeeper.GetAllValidators(ctx)
	s.Require().NoError(err)
	s.Require().Len(validators, 6)

	initialConsAddr, err := validators[3].GetConsAddr()
	s.Require().NoError(err)

	oldPk, err := stakingKeeper.ValidatorIdentifier(ctx, initialConsAddr)
	s.Require().NoError(err)
	s.Require().Nil(oldPk)

	bondedPool := authtypes.NewEmptyModuleAccount(types.BondedPoolName)
	accountKeeper.EXPECT().GetModuleAccount(gomock.Any(), types.BondedPoolName).Return(bondedPool).AnyTimes()
	bankKeeper.EXPECT().GetBalance(gomock.Any(), bondedPool.GetAddress(), sdk.DefaultBondDenom).Return(sdk.NewInt64Coin(sdk.DefaultBondDenom, 1000000)).AnyTimes()

	val, err := stakingKeeper.ValidatorAddressCodec().StringToBytes(validators[3].GetOperator())
	s.Require().NoError(err)
	bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), sdk.AccAddress(val), gomock.Any(), gomock.Any()).Return(nil).AnyTimes()

	req, err := types.NewMsgRotateConsPubKey(validators[3].GetOperator(), PKs[495])
	s.Require().NoError(err)
	_, err = msgServer.RotateConsPubKey(ctx, req)
	s.Require().NoError(err)
	_, err = stakingKeeper.BlockValidatorUpdates(ctx)
	s.Require().NoError(err)
	params, err := stakingKeeper.Params.Get(ctx)
	s.Require().NoError(err)

	oldPk1, err := stakingKeeper.ValidatorIdentifier(ctx, sdk.ConsAddress(PKs[495].Address()))
	s.Require().NoError(err)
	s.Require().Equal(oldPk1.Bytes(), initialConsAddr)

	ctx = ctx.WithHeaderInfo(header.Info{Time: ctx.BlockTime().Add(params.UnbondingTime).Add(time.Hour)})
	_, err = stakingKeeper.BlockValidatorUpdates(ctx)
	s.Require().NoError(err)

	req, err = types.NewMsgRotateConsPubKey(validators[3].GetOperator(), PKs[494])
	s.Require().NoError(err)
	_, err = msgServer.RotateConsPubKey(ctx, req)
	s.Require().NoError(err)
	_, err = stakingKeeper.BlockValidatorUpdates(ctx)
	s.Require().NoError(err)

	ctx = ctx.WithHeaderInfo(header.Info{Time: ctx.BlockTime().Add(params.UnbondingTime)})

	oldPk2, err := stakingKeeper.ValidatorIdentifier(ctx, sdk.ConsAddress(PKs[494].Address()))
	s.Require().NoError(err)
	_, err = stakingKeeper.BlockValidatorUpdates(ctx)
	s.Require().NoError(err)

	s.Require().Equal(oldPk2.Bytes(), initialConsAddr)
}

func (s *KeeperTestSuite) setValidators(n int) {
	stakingKeeper, ctx := s.stakingKeeper, s.ctx

	_, addrVals := createValAddrs(n)

	for i := 0; i < n; i++ {
		addr, err := s.stakingKeeper.ValidatorAddressCodec().BytesToString(addrVals[i])
		s.Require().NoError(err)

		val := testutil.NewValidator(s.T(), addrVals[i], PKs[i])
		valTokens := stakingKeeper.TokensFromConsensusPower(ctx, 10)
		val, issuedShares := val.AddTokensFromDel(valTokens)
		s.Require().Equal(valTokens, issuedShares.RoundInt())

		s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), types.NotBondedPoolName, types.BondedPoolName, gomock.Any())
		_ = stakingkeeper.TestingUpdateValidator(stakingKeeper, ctx, val, true)
		accAddr, err := s.accountKeeper.AddressCodec().BytesToString(addrVals[i])
		s.Require().NoError(err)
		selfDelegation := types.NewDelegation(accAddr, addr, issuedShares)
		err = stakingKeeper.SetDelegation(ctx, selfDelegation)
		s.Require().NoError(err)

		err = stakingKeeper.SetValidatorByConsAddr(ctx, val)
		s.Require().NoError(err)

	}

	validators, err := stakingKeeper.GetAllValidators(ctx)
	s.Require().NoError(err)
	s.Require().Len(validators, n)
}
