package keeper_test

import (
	"testing"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/distribution"
	"github.com/cosmos/cosmos-sdk/x/distribution/keeper"
	distrtestutil "github.com/cosmos/cosmos-sdk/x/distribution/testutil"
	disttypes "github.com/cosmos/cosmos-sdk/x/distribution/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

func TestCalculateRewardsBasic(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := sdk.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		key,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		"fee_collector",
		authtypes.NewModuleAddress("gov").String(),
	)

	// reset fee pool
	distrKeeper.SetFeePool(ctx, disttypes.InitialFeePool())
	distrKeeper.SetParams(ctx, disttypes.DefaultParams())

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	val, err := distrtestutil.CreateValidator(valConsPk0, sdk.NewInt(1000))
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(sdk.NewDecWithPrec(5, 1), sdk.NewDecWithPrec(5, 1), math.LegacyNewDec(0))

	// delegation mock
	del := stakingtypes.NewDelegation(addr, valAddr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(3)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// historical count should be 2 (once for validator init, once for delegation init)
	require.Equal(t, uint64(2), distrKeeper.GetValidatorHistoricalReferenceCount(ctx))

	// end period
	endingPeriod := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// historical count should be 2 still
	require.Equal(t, uint64(2), distrKeeper.GetValidatorHistoricalReferenceCount(ctx))

	// calculate delegation rewards
	rewards := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards should be zero
	require.True(t, rewards.IsZero())

	// allocate some rewards
	initial := int64(10)
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial)}}
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// end period
	endingPeriod = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards should be half the tokens
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 2)}}, rewards)

	// commission should be the other half
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 2)}}, distrKeeper.GetValidatorAccumulatedCommission(ctx, valAddr).Commission)
}

func TestCalculateRewardsAfterSlash(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := sdk.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		key,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		"fee_collector",
		authtypes.NewModuleAddress("gov").String(),
	)

	// reset fee pool
	distrKeeper.SetFeePool(ctx, disttypes.InitialFeePool())
	distrKeeper.SetParams(ctx, disttypes.DefaultParams())

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	valPower := int64(100)
	stake := sdk.TokensFromConsensusPower(100, sdk.DefaultPowerReduction)
	val, err := distrtestutil.CreateValidator(valConsPk0, stake)
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(sdk.NewDecWithPrec(5, 1), sdk.NewDecWithPrec(5, 1), math.LegacyNewDec(0))

	del := stakingtypes.NewDelegation(addr, valAddr, val.DelegatorShares)

	// set mock calls
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(4)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// end period
	endingPeriod := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards should be zero
	require.True(t, rewards.IsZero())

	// start out block height
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)

	// slash the validator by 50% (simulated with manual calls; we assume the validator is bonded)
	slashedTokens := distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		sdk.NewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
	)
	require.True(t, slashedTokens.IsPositive(), "expected positive slashed tokens, got: %s", slashedTokens)

	// increase block height
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)

	// allocate some rewards
	initial := sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction)
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: sdk.NewDecFromInt(initial)}}
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// end period
	endingPeriod = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards should be half the tokens
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: sdk.NewDecFromInt(initial.QuoRaw(2))}}, rewards)

	// commission should be the other half
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: sdk.NewDecFromInt(initial.QuoRaw(2))}},
		distrKeeper.GetValidatorAccumulatedCommission(ctx, valAddr).Commission)
}

func TestCalculateRewardsAfterManySlashes(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := sdk.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		key,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		"fee_collector",
		authtypes.NewModuleAddress("gov").String(),
	)

	// reset fee pool
	distrKeeper.SetFeePool(ctx, disttypes.InitialFeePool())
	distrKeeper.SetParams(ctx, disttypes.DefaultParams())

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	valPower := int64(100)
	stake := sdk.TokensFromConsensusPower(valPower, sdk.DefaultPowerReduction)
	val, err := distrtestutil.CreateValidator(valConsPk0, stake)
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(sdk.NewDecWithPrec(5, 1), sdk.NewDecWithPrec(5, 1), math.LegacyNewDec(0))

	// delegation mocks
	del := stakingtypes.NewDelegation(addr, valAddr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(4)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// end period
	endingPeriod := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards should be zero
	require.True(t, rewards.IsZero())

	// start out block height
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)

	// slash the validator by 50% (simulated with manual calls; we assume the validator is bonded)
	slashedTokens := distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		sdk.NewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
	)
	require.True(t, slashedTokens.IsPositive(), "expected positive slashed tokens, got: %s", slashedTokens)

	// expect a call for the next slash with the updated validator
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(1)

	// increase block height
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)

	// allocate some rewards
	initial := sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction)
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: sdk.NewDecFromInt(initial)}}
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// slash the validator by 50% again
	slashedTokens = distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower/2,
		sdk.NewDecWithPrec(2, 1),
		&val,
		&distrKeeper,
	)
	require.True(t, slashedTokens.IsPositive(), "expected positive slashed tokens, got: %s", slashedTokens)

	// increase block height
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)

	// allocate some more rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// end period
	endingPeriod = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards should be half the tokens
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: sdk.NewDecFromInt(initial)}}, rewards)

	// commission should be the other half
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: sdk.NewDecFromInt(initial)}},
		distrKeeper.GetValidatorAccumulatedCommission(ctx, valAddr).Commission)
}

func TestCalculateRewardsMultiDelegator(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := sdk.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		key,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		"fee_collector",
		authtypes.NewModuleAddress("gov").String(),
	)

	// reset fee pool
	distrKeeper.SetFeePool(ctx, disttypes.InitialFeePool())
	distrKeeper.SetParams(ctx, disttypes.DefaultParams())

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr0 := sdk.AccAddress(valAddr)
	val, err := distrtestutil.CreateValidator(valConsPk0, math.NewInt(100))
	require.NoError(t, err)

	val.Commission = stakingtypes.NewCommission(sdk.NewDecWithPrec(5, 1), sdk.NewDecWithPrec(5, 1), math.LegacyNewDec(0))

	del0 := stakingtypes.NewDelegation(addr0, valAddr, val.DelegatorShares)

	// set mock calls
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(4)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr0, valAddr).Return(del0).Times(1)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr0, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some rewards
	initial := int64(20)
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial)}}
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// second delegation
	addr1 := sdk.AccAddress(valConsAddr1)
	_, del1, err := distrtestutil.Delegate(ctx, distrKeeper, addr1, &val, sdk.NewInt(100), nil)
	require.NoError(t, err)

	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr1, valAddr).Return(del1)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(1)

	// call necessary hooks to update a delegation
	err = distrKeeper.Hooks().AfterDelegationModified(ctx, addr1, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some more rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// end period
	endingPeriod := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards := distrKeeper.CalculateDelegationRewards(ctx, val, del0, endingPeriod)

	// rewards for del0 should be 3/4 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial * 3 / 4)}}, rewards)

	// calculate delegation rewards for del2
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del1, endingPeriod)

	// rewards for del2 should be 1/4 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial * 1 / 4)}}, rewards)

	// commission should be equal to initial (50% twice)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial)}}, distrKeeper.GetValidatorAccumulatedCommission(ctx, valAddr).Commission)
}

func TestWithdrawDelegationRewardsBasic(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := sdk.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		key,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		"fee_collector",
		authtypes.NewModuleAddress("gov").String(),
	)

	// reset fee pool
	distrKeeper.SetFeePool(ctx, disttypes.InitialFeePool())
	distrKeeper.SetParams(ctx, disttypes.DefaultParams())

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	val, err := distrtestutil.CreateValidator(valConsPk0, math.NewInt(100))
	require.NoError(t, err)

	val.Commission = stakingtypes.NewCommission(sdk.NewDecWithPrec(5, 1), sdk.NewDecWithPrec(5, 1), math.LegacyNewDec(0))

	// delegation mock
	del := stakingtypes.NewDelegation(addr, valAddr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(5)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del).Times(3)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some rewards
	initial := sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction)
	tokens := sdk.DecCoins{sdk.NewDecCoin(sdk.DefaultBondDenom, initial)}

	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// historical count should be 2 (initial + latest for delegation)
	require.Equal(t, uint64(2), distrKeeper.GetValidatorHistoricalReferenceCount(ctx))

	// withdraw rewards (the bank keeper should be called with the right amount of tokens to transfer)
	expRewards := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, initial.QuoRaw(2))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expRewards)
	_, err = distrKeeper.WithdrawDelegationRewards(ctx, sdk.AccAddress(valAddr), valAddr)
	require.Nil(t, err)

	// historical count should still be 2 (added one record, cleared one)
	require.Equal(t, uint64(2), distrKeeper.GetValidatorHistoricalReferenceCount(ctx))

	// withdraw commission (the bank keeper should be called with the right amount of tokens to transfer)
	expCommission := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, initial.QuoRaw(2))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expCommission)
	_, err = distrKeeper.WithdrawValidatorCommission(ctx, valAddr)
	require.Nil(t, err)
}

func TestCalculateRewardsAfterManySlashesInSameBlock(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := sdk.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		key,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		"fee_collector",
		authtypes.NewModuleAddress("gov").String(),
	)

	// reset fee pool
	distrKeeper.SetFeePool(ctx, disttypes.InitialFeePool())
	distrKeeper.SetParams(ctx, disttypes.DefaultParams())

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	val, err := distrtestutil.CreateValidator(valConsPk0, math.NewInt(100))
	require.NoError(t, err)

	val.Commission = stakingtypes.NewCommission(sdk.NewDecWithPrec(5, 1), sdk.NewDecWithPrec(5, 1), math.LegacyNewDec(0))

	// delegation mock
	del := stakingtypes.NewDelegation(addr, valAddr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(5)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// end period
	endingPeriod := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards should be zero
	require.True(t, rewards.IsZero())

	// start out block height
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)

	// allocate some rewards
	initial := sdk.NewDecFromInt(sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction))
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	valPower := int64(100)
	// slash the validator by 50% (simulated with manual calls; we assume the validator is bonded)
	distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		sdk.NewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
	)

	// slash the validator by 50% again
	// stakingKeeper.Slash(ctx, valConsAddr0, ctx.BlockHeight(), valPower/2, sdk.NewDecWithPrec(5, 1))
	distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower/2,
		sdk.NewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
	)

	// increase block height
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)

	// allocate some more rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// end period
	endingPeriod = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards should be half the tokens
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}, rewards)

	// commission should be the other half
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}, distrKeeper.GetValidatorAccumulatedCommission(ctx, valAddr).Commission)
}

func TestCalculateRewardsMultiDelegatorMultiSlash(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := sdk.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		key,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		"fee_collector",
		authtypes.NewModuleAddress("gov").String(),
	)

	// reset fee pool
	distrKeeper.SetFeePool(ctx, disttypes.InitialFeePool())
	distrKeeper.SetParams(ctx, disttypes.DefaultParams())

	valPower := int64(100)

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	val, err := distrtestutil.CreateValidator(valConsPk0, sdk.TokensFromConsensusPower(valPower, sdk.DefaultPowerReduction))
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(sdk.NewDecWithPrec(5, 1), sdk.NewDecWithPrec(5, 1), math.LegacyNewDec(0))

	// validator and delegation mocks
	del := stakingtypes.NewDelegation(addr, valAddr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(3)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(2)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some rewards
	initial := sdk.NewDecFromInt(sdk.TokensFromConsensusPower(30, sdk.DefaultPowerReduction))
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// slash the validator
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)
	distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		sdk.NewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
	)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)

	// update validator mock
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(1)

	// second delegation
	_, del2, err := distrtestutil.Delegate(
		ctx,
		distrKeeper,
		sdk.AccAddress(valConsAddr1),
		&val,
		sdk.TokensFromConsensusPower(100, sdk.DefaultPowerReduction),
		nil,
	)
	require.NoError(t, err)

	// new delegation mock and update validator mock
	stakingKeeper.EXPECT().Delegation(gomock.Any(), sdk.AccAddress(valConsAddr1), valAddr).Return(del2)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(1)

	// call necessary hooks to update a delegation
	err = distrKeeper.Hooks().AfterDelegationModified(ctx, sdk.AccAddress(valConsAddr1), valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some more rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// slash the validator again
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)
	distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		sdk.NewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
	)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 3)

	// end period
	endingPeriod := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards for del1 should be 2/3 initial (half initial first period, 1/6 initial second period)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial.QuoInt64(2).Add(initial.QuoInt64(6))}}, rewards)

	// calculate delegation rewards for del2
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del2, endingPeriod)

	// rewards for del2 should be initial / 3
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial.QuoInt64(3)}}, rewards)

	// commission should be equal to initial (twice 50% commission, unaffected by slashing)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}, distrKeeper.GetValidatorAccumulatedCommission(ctx, valAddr).Commission)
}

func TestCalculateRewardsMultiDelegatorMultWithdraw(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := sdk.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		key,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		"fee_collector",
		authtypes.NewModuleAddress("gov").String(),
	)

	// reset fee pool
	distrKeeper.SetFeePool(ctx, disttypes.InitialFeePool())
	distrKeeper.SetParams(ctx, disttypes.DefaultParams())

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	val, err := distrtestutil.CreateValidator(valConsPk0, sdk.NewInt(100))
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(sdk.NewDecWithPrec(5, 1), sdk.NewDecWithPrec(5, 1), math.LegacyNewDec(0))

	// validator and delegation mocks
	del := stakingtypes.NewDelegation(addr, valAddr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(3)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del).Times(5)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(2)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some rewards
	initial := int64(20)
	tokens := sdk.DecCoins{sdk.NewDecCoin(sdk.DefaultBondDenom, math.NewInt(initial))}
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// historical count should be 2 (validator init, delegation init)
	require.Equal(t, uint64(2), distrKeeper.GetValidatorHistoricalReferenceCount(ctx))

	// second delegation
	_, del2, err := distrtestutil.Delegate(
		ctx,
		distrKeeper,
		sdk.AccAddress(valConsAddr1),
		&val,
		sdk.NewInt(100),
		nil,
	)
	require.NoError(t, err)

	// new delegation mock and update validator mock
	stakingKeeper.EXPECT().Delegation(gomock.Any(), sdk.AccAddress(valConsAddr1), valAddr).Return(del2).Times(3)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(6)

	// call necessary hooks to update a delegation
	err = distrKeeper.Hooks().AfterDelegationModified(ctx, sdk.AccAddress(valConsAddr1), valAddr)
	require.NoError(t, err)

	// historical count should be 3 (second delegation init)
	require.Equal(t, uint64(3), distrKeeper.GetValidatorHistoricalReferenceCount(ctx))

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some more rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// first delegator withdraws
	expRewards := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(initial*3/4))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expRewards)
	_, err = distrKeeper.WithdrawDelegationRewards(ctx, addr, valAddr)
	require.NoError(t, err)

	// second delegator withdraws
	expRewards = sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(initial*1/4))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, sdk.AccAddress(valConsAddr1), expRewards)
	_, err = distrKeeper.WithdrawDelegationRewards(ctx, sdk.AccAddress(valConsAddr1), valAddr)
	require.NoError(t, err)

	// historical count should be 3 (validator init + two delegations)
	require.Equal(t, uint64(3), distrKeeper.GetValidatorHistoricalReferenceCount(ctx))

	// validator withdraws commission
	expCommission := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(initial))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expCommission)
	_, err = distrKeeper.WithdrawValidatorCommission(ctx, valAddr)
	require.NoError(t, err)

	// end period
	endingPeriod := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards for del1 should be zero
	require.True(t, rewards.IsZero())

	// calculate delegation rewards for del2
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del2, endingPeriod)

	// rewards for del2 should be zero
	require.True(t, rewards.IsZero())

	// commission should be zero
	require.True(t, distrKeeper.GetValidatorAccumulatedCommission(ctx, valAddr).Commission.IsZero())

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some more rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// first delegator withdraws again
	expCommission = sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(initial*1/4))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expCommission)
	_, err = distrKeeper.WithdrawDelegationRewards(ctx, addr, valAddr)
	require.NoError(t, err)

	// end period
	endingPeriod = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards for del1 should be zero
	require.True(t, rewards.IsZero())

	// calculate delegation rewards for del2
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del2, endingPeriod)

	// rewards for del2 should be 1/4 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 4)}}, rewards)

	// commission should be half initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 2)}}, distrKeeper.GetValidatorAccumulatedCommission(ctx, valAddr).Commission)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some more rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// withdraw commission
	expCommission = sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(initial))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expCommission)
	_, err = distrKeeper.WithdrawValidatorCommission(ctx, valAddr)
	require.NoError(t, err)

	// end period
	endingPeriod = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)

	// rewards for del1 should be 1/4 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 4)}}, rewards)

	// calculate delegation rewards for del2
	rewards = distrKeeper.CalculateDelegationRewards(ctx, val, del2, endingPeriod)

	// rewards for del2 should be 1/2 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 2)}}, rewards)

	// commission should be zero
	require.True(t, distrKeeper.GetValidatorAccumulatedCommission(ctx, valAddr).Commission.IsZero())
}

func Test100PercentCommissionReward(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := sdk.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		key,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		"fee_collector",
		authtypes.NewModuleAddress("gov").String(),
	)

	// reset fee pool
	distrKeeper.SetFeePool(ctx, disttypes.InitialFeePool())
	distrKeeper.SetParams(ctx, disttypes.DefaultParams())

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	val, err := distrtestutil.CreateValidator(valConsPk0, sdk.NewInt(100))
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(sdk.NewDecWithPrec(10, 1), sdk.NewDecWithPrec(10, 1), math.LegacyNewDec(0))

	// validator and delegation mocks
	del := stakingtypes.NewDelegation(addr, valAddr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(3)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del).Times(3)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val).Times(2)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some rewards
	initial := int64(20)
	tokens := sdk.DecCoins{sdk.NewDecCoin(sdk.DefaultBondDenom, math.NewInt(initial))}
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some more rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	// next block
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// allocate some more rewards
	distrKeeper.AllocateTokensToValidator(ctx, val, tokens)

	rewards, err := distrKeeper.WithdrawDelegationRewards(ctx, sdk.AccAddress(addr), valAddr)
	require.NoError(t, err)

	zeroRewards := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.ZeroInt())}
	require.True(t, rewards.IsEqual(zeroRewards))

	events := ctx.EventManager().Events()
	lastEvent := events[len(events)-1]

	var hasValue bool
	for _, attr := range lastEvent.Attributes {
		if attr.Key == "amount" && attr.Value == "0stake" {
			hasValue = true
		}
	}
	require.True(t, hasValue)
}
