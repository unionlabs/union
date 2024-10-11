package keeper_test

import (
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/distribution"
	"cosmossdk.io/x/distribution/keeper"
	distrtestutil "cosmossdk.io/x/distribution/testutil"
	disttypes "cosmossdk.io/x/distribution/types"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestCalculateRewardsBasic(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec(sdk.Bech32PrefixValAddr)).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec(sdk.Bech32MainPrefix)).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	// reset fee pool
	require.NoError(t, distrKeeper.FeePool.Set(ctx, disttypes.InitialFeePool()))
	require.NoError(t, distrKeeper.Params.Set(ctx, disttypes.DefaultParams()))

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	operatorAddr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valConsPk0.Address())
	require.NoError(t, err)
	val, err := distrtestutil.CreateValidator(valConsPk0, operatorAddr, math.NewInt(1000))
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDec(0))

	addrStr, err := accountKeeper.AddressCodec().BytesToString(addr)
	require.NoError(t, err)
	valAddrStr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	require.NoError(t, err)

	// delegation mock
	del := stakingtypes.NewDelegation(addrStr, valAddrStr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(3)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del, nil)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// historical count should be 2 (once for validator init, once for delegation init)
	require.Equal(t, 2, getValHistoricalReferenceCount(distrKeeper, ctx))

	// end period
	endingPeriod, _ := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// historical count should be 2 still
	require.Equal(t, 2, getValHistoricalReferenceCount(distrKeeper, ctx))

	// calculate delegation rewards
	rewards, err := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards should be zero
	require.True(t, rewards.IsZero())

	// allocate some rewards
	initial := int64(10)
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial)}}
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// end period
	endingPeriod, _ = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards should be half the tokens
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 2)}}, rewards)

	// commission should be the other half
	valCommission, err := distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 2)}}, valCommission.Commission)
}

func getValHistoricalReferenceCount(k keeper.Keeper, ctx sdk.Context) int {
	count := 0
	err := k.ValidatorHistoricalRewards.Walk(
		ctx, nil, func(key collections.Pair[sdk.ValAddress, uint64], rewards disttypes.ValidatorHistoricalRewards) (stop bool, err error) {
			count += int(rewards.ReferenceCount)
			return false, nil
		},
	)
	if err != nil {
		panic(err)
	}

	return count
}

func TestCalculateRewardsAfterSlash(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec(sdk.Bech32PrefixValAddr)).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec(sdk.Bech32MainPrefix)).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	// reset fee pool
	require.NoError(t, distrKeeper.FeePool.Set(ctx, disttypes.InitialFeePool()))
	require.NoError(t, distrKeeper.Params.Set(ctx, disttypes.DefaultParams()))

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	valPower := int64(100)
	stake := sdk.TokensFromConsensusPower(100, sdk.DefaultPowerReduction)
	operatorAddr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valConsPk0.Address())
	require.NoError(t, err)
	val, err := distrtestutil.CreateValidator(valConsPk0, operatorAddr, stake)
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDec(0))

	addrStr, err := accountKeeper.AddressCodec().BytesToString(addr)
	require.NoError(t, err)
	valAddrStr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	require.NoError(t, err)

	del := stakingtypes.NewDelegation(addrStr, valAddrStr, val.DelegatorShares)

	// set mock calls
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(4)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del, nil)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// end period
	endingPeriod, _ := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards, err := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards should be zero
	require.True(t, rewards.IsZero())

	// start out block height
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	// slash the validator by 50% (simulated with manual calls; we assume the validator is bonded)
	slashedTokens := distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		math.LegacyNewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
		stakingKeeper,
	)
	require.True(t, slashedTokens.IsPositive(), "expected positive slashed tokens, got: %s", slashedTokens)

	// increase block height
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	// allocate some rewards
	initial := sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction)
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDecFromInt(initial)}}
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// end period
	endingPeriod, _ = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards should be half the tokens
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDecFromInt(initial.QuoRaw(2))}}, rewards)

	// commission should be the other half
	valCommission, err := distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDecFromInt(initial.QuoRaw(2))}},
		valCommission.Commission)
}

func TestCalculateRewardsAfterManySlashes(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec(sdk.Bech32PrefixValAddr)).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec(sdk.Bech32MainPrefix)).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	// reset fee pool
	require.NoError(t, distrKeeper.FeePool.Set(ctx, disttypes.InitialFeePool()))
	require.NoError(t, distrKeeper.Params.Set(ctx, disttypes.DefaultParams()))

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	valPower := int64(100)
	stake := sdk.TokensFromConsensusPower(valPower, sdk.DefaultPowerReduction)
	operatorAddr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valConsPk0.Address())
	require.NoError(t, err)
	val, err := distrtestutil.CreateValidator(valConsPk0, operatorAddr, stake)
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDec(0))

	addrStr, err := accountKeeper.AddressCodec().BytesToString(addr)
	require.NoError(t, err)
	valAddrStr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	require.NoError(t, err)

	// delegation mocks
	del := stakingtypes.NewDelegation(addrStr, valAddrStr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(4)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del, nil)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// end period
	endingPeriod, _ := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards, err := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards should be zero
	require.True(t, rewards.IsZero())

	// start out block height
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	// slash the validator by 50% (simulated with manual calls; we assume the validator is bonded)
	slashedTokens := distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		math.LegacyNewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
		stakingKeeper,
	)
	require.True(t, slashedTokens.IsPositive(), "expected positive slashed tokens, got: %s", slashedTokens)

	// expect a call for the next slash with the updated validator
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(1)

	// increase block height
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	// allocate some rewards
	initial := sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction)
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDecFromInt(initial)}}
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// slash the validator by 50% again
	slashedTokens = distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower/2,
		math.LegacyNewDecWithPrec(2, 1),
		&val,
		&distrKeeper,
		stakingKeeper,
	)
	require.True(t, slashedTokens.IsPositive(), "expected positive slashed tokens, got: %s", slashedTokens)

	// increase block height
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	// allocate some more rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// end period
	endingPeriod, _ = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards should be half the tokens
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDecFromInt(initial)}}, rewards)

	// commission should be the other half
	valCommission, err := distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDecFromInt(initial)}},
		valCommission.Commission)
}

func TestCalculateRewardsMultiDelegator(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec(sdk.Bech32PrefixValAddr)).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec(sdk.Bech32MainPrefix)).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	// reset fee pool
	require.NoError(t, distrKeeper.FeePool.Set(ctx, disttypes.InitialFeePool()))
	require.NoError(t, distrKeeper.Params.Set(ctx, disttypes.DefaultParams()))

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr0 := sdk.AccAddress(valAddr)
	operatorAddr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valConsPk0.Address())
	require.NoError(t, err)
	val, err := distrtestutil.CreateValidator(valConsPk0, operatorAddr, math.NewInt(100))
	require.NoError(t, err)

	addrStr, err := accountKeeper.AddressCodec().BytesToString(addr0)
	require.NoError(t, err)
	valAddrStr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	require.NoError(t, err)

	val.Commission = stakingtypes.NewCommission(math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDec(0))

	del0 := stakingtypes.NewDelegation(addrStr, valAddrStr, val.DelegatorShares)

	// set mock calls
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(4)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr0, valAddr).Return(del0, nil).Times(1)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr0, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some rewards
	initial := int64(20)
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial)}}
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// second delegation
	addr1 := sdk.AccAddress(valConsAddr1)
	_, del1, err := distrtestutil.Delegate(ctx, distrKeeper, addr1, &val, math.NewInt(100), nil, stakingKeeper, accountKeeper.AddressCodec())
	require.NoError(t, err)

	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr1, valAddr).Return(del1, nil)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(1)

	// call necessary hooks to update a delegation
	err = distrKeeper.Hooks().AfterDelegationModified(ctx, addr1, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some more rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// end period
	endingPeriod, _ := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards, err := distrKeeper.CalculateDelegationRewards(ctx, val, del0, endingPeriod)
	require.NoError(t, err)

	// rewards for del0 should be 3/4 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial * 3 / 4)}}, rewards)

	// calculate delegation rewards for del2
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del1, endingPeriod)
	require.NoError(t, err)

	// rewards for del2 should be 1/4 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial * 1 / 4)}}, rewards)

	// commission should be equal to initial (50% twice)
	valCommission, err := distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial)}}, valCommission.Commission)
}

func TestWithdrawDelegationRewardsBasic(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec(sdk.Bech32PrefixValAddr)).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec(sdk.Bech32MainPrefix)).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	// reset fee pool
	require.NoError(t, distrKeeper.FeePool.Set(ctx, disttypes.InitialFeePool()))
	require.NoError(t, distrKeeper.Params.Set(ctx, disttypes.DefaultParams()))

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	operatorAddr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valConsPk0.Address())
	require.NoError(t, err)
	val, err := distrtestutil.CreateValidator(valConsPk0, operatorAddr, math.NewInt(100))
	require.NoError(t, err)

	val.Commission = stakingtypes.NewCommission(math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDec(0))

	addrStr, err := accountKeeper.AddressCodec().BytesToString(addr)
	require.NoError(t, err)
	valAddrStr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	require.NoError(t, err)

	// delegation mock
	del := stakingtypes.NewDelegation(addrStr, valAddrStr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(5)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del, nil).Times(3)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some rewards
	initial := sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction)
	tokens := sdk.DecCoins{sdk.NewDecCoin(sdk.DefaultBondDenom, initial)}

	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// historical count should be 2 (initial + latest for delegation)
	require.Equal(t, 2, getValHistoricalReferenceCount(distrKeeper, ctx))

	// withdraw rewards (the bank keeper should be called with the right amount of tokens to transfer)
	expRewards := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, initial.QuoRaw(2))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expRewards)
	_, err = distrKeeper.WithdrawDelegationRewards(ctx, sdk.AccAddress(valAddr), valAddr)
	require.Nil(t, err)

	// historical count should still be 2 (added one record, cleared one)
	require.Equal(t, 2, getValHistoricalReferenceCount(distrKeeper, ctx))

	// withdraw commission (the bank keeper should be called with the right amount of tokens to transfer)
	expCommission := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, initial.QuoRaw(2))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expCommission)
	_, err = distrKeeper.WithdrawValidatorCommission(ctx, valAddr)
	require.Nil(t, err)
}

func TestCalculateRewardsAfterManySlashesInSameBlock(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec(sdk.Bech32PrefixValAddr)).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec(sdk.Bech32MainPrefix)).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	// reset fee pool
	require.NoError(t, distrKeeper.FeePool.Set(ctx, disttypes.InitialFeePool()))
	require.NoError(t, distrKeeper.Params.Set(ctx, disttypes.DefaultParams()))

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	operatorAddr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valConsPk0.Address())
	require.NoError(t, err)
	val, err := distrtestutil.CreateValidator(valConsPk0, operatorAddr, math.NewInt(100))
	require.NoError(t, err)

	val.Commission = stakingtypes.NewCommission(math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDec(0))

	addrStr, err := accountKeeper.AddressCodec().BytesToString(addr)
	require.NoError(t, err)
	valAddrStr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	require.NoError(t, err)

	// delegation mock
	del := stakingtypes.NewDelegation(addrStr, valAddrStr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(5)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del, nil)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// end period
	endingPeriod, _ := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards, err := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards should be zero
	require.True(t, rewards.IsZero())

	// start out block height
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	// allocate some rewards
	initial := math.LegacyNewDecFromInt(sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction))
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	valPower := int64(100)
	// slash the validator by 50% (simulated with manual calls; we assume the validator is bonded)
	distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		math.LegacyNewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
		stakingKeeper,
	)

	// slash the validator by 50% again
	// stakingKeeper.Slash(ctx, valConsAddr0, ctx.BlockHeight(), valPower/2, math.LegacyNewDecWithPrec(5, 1))
	distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower/2,
		math.LegacyNewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
		stakingKeeper,
	)

	// increase block height
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	// allocate some more rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// end period
	endingPeriod, _ = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards should be half the tokens
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}, rewards)

	// commission should be the other half
	valCommission, err := distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}, valCommission.Commission)
}

func TestCalculateRewardsMultiDelegatorMultiSlash(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec(sdk.Bech32PrefixValAddr)).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec(sdk.Bech32MainPrefix)).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	// reset fee pool
	require.NoError(t, distrKeeper.FeePool.Set(ctx, disttypes.InitialFeePool()))
	require.NoError(t, distrKeeper.Params.Set(ctx, disttypes.DefaultParams()))

	valPower := int64(100)

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	operatorAddr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valConsPk0.Address())
	require.NoError(t, err)
	val, err := distrtestutil.CreateValidator(valConsPk0, operatorAddr, sdk.TokensFromConsensusPower(valPower, sdk.DefaultPowerReduction))
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDec(0))

	addrStr, err := accountKeeper.AddressCodec().BytesToString(addr)
	require.NoError(t, err)
	valAddrStr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	require.NoError(t, err)

	// validator and delegation mocks
	del := stakingtypes.NewDelegation(addrStr, valAddrStr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(3)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del, nil)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(2)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some rewards
	initial := math.LegacyNewDecFromInt(sdk.TokensFromConsensusPower(30, sdk.DefaultPowerReduction))
	tokens := sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// slash the validator
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		math.LegacyNewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
		stakingKeeper,
	)
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	// update validator mock
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(1)

	// second delegation
	_, del2, err := distrtestutil.Delegate(
		ctx,
		distrKeeper,
		sdk.AccAddress(valConsAddr1),
		&val,
		sdk.TokensFromConsensusPower(100, sdk.DefaultPowerReduction),
		nil,
		stakingKeeper,
		accountKeeper.AddressCodec(),
	)
	require.NoError(t, err)

	// new delegation mock and update validator mock
	stakingKeeper.EXPECT().Delegation(gomock.Any(), sdk.AccAddress(valConsAddr1), valAddr).Return(del2, nil)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(1)

	// call necessary hooks to update a delegation
	err = distrKeeper.Hooks().AfterDelegationModified(ctx, sdk.AccAddress(valConsAddr1), valAddr)
	require.NoError(t, err)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some more rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// slash the validator again
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	distrtestutil.SlashValidator(
		ctx,
		valConsAddr0,
		ctx.BlockHeight(),
		valPower,
		math.LegacyNewDecWithPrec(5, 1),
		&val,
		&distrKeeper,
		stakingKeeper,
	)
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 3})

	// end period
	endingPeriod, _ := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards, err := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards for del1 should be 2/3 initial (half initial first period, 1/6 initial second period)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial.QuoInt64(2).Add(initial.QuoInt64(6))}}, rewards)

	// calculate delegation rewards for del2
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del2, endingPeriod)
	require.NoError(t, err)

	// rewards for del2 should be initial / 3
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial.QuoInt64(3)}}, rewards)

	// commission should be equal to initial (twice 50% commission, unaffected by slashing)
	valCommission, err := distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: initial}}, valCommission.Commission)
}

func TestCalculateRewardsMultiDelegatorMultWithdraw(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec(sdk.Bech32PrefixValAddr)).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec(sdk.Bech32MainPrefix)).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	// reset fee pool
	require.NoError(t, distrKeeper.FeePool.Set(ctx, disttypes.InitialFeePool()))
	require.NoError(t, distrKeeper.Params.Set(ctx, disttypes.DefaultParams()))

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	operatorAddr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valConsPk0.Address())
	require.NoError(t, err)
	val, err := distrtestutil.CreateValidator(valConsPk0, operatorAddr, math.NewInt(100))
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDec(0))

	addrStr, err := accountKeeper.AddressCodec().BytesToString(addr)
	require.NoError(t, err)
	valAddrStr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	require.NoError(t, err)

	// validator and delegation mocks
	del := stakingtypes.NewDelegation(addrStr, valAddrStr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(3)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del, nil).Times(5)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(2)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some rewards
	initial := int64(20)
	tokens := sdk.DecCoins{sdk.NewDecCoin(sdk.DefaultBondDenom, math.NewInt(initial))}
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// historical count should be 2 (validator init, delegation init)
	require.Equal(t, 2, getValHistoricalReferenceCount(distrKeeper, ctx))

	// second delegation
	_, del2, err := distrtestutil.Delegate(
		ctx,
		distrKeeper,
		sdk.AccAddress(valConsAddr1),
		&val,
		math.NewInt(100),
		nil,
		stakingKeeper,
		accountKeeper.AddressCodec(),
	)
	require.NoError(t, err)

	// new delegation mock and update validator mock
	stakingKeeper.EXPECT().Delegation(gomock.Any(), sdk.AccAddress(valConsAddr1), valAddr).Return(del2, nil).Times(3)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(6)

	// call necessary hooks to update a delegation
	err = distrKeeper.Hooks().AfterDelegationModified(ctx, sdk.AccAddress(valConsAddr1), valAddr)
	require.NoError(t, err)

	// historical count should be 3 (second delegation init)
	require.Equal(t, 3, getValHistoricalReferenceCount(distrKeeper, ctx))

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some more rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

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
	require.Equal(t, 3, getValHistoricalReferenceCount(distrKeeper, ctx))

	// validator withdraws commission
	expCommission := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(initial))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expCommission)
	_, err = distrKeeper.WithdrawValidatorCommission(ctx, valAddr)
	require.NoError(t, err)

	// end period
	endingPeriod, _ := distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards, err := distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards for del1 should be zero
	require.True(t, rewards.IsZero())

	// calculate delegation rewards for del2
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del2, endingPeriod)
	require.NoError(t, err)

	// rewards for del2 should be zero
	require.True(t, rewards.IsZero())

	// commission should be zero
	valCommission, err := distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	require.True(t, valCommission.Commission.IsZero())

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some more rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// first delegator withdraws again
	expCommission = sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(initial*1/4))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expCommission)
	_, err = distrKeeper.WithdrawDelegationRewards(ctx, addr, valAddr)
	require.NoError(t, err)

	// end period
	endingPeriod, _ = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards for del1 should be zero
	require.True(t, rewards.IsZero())

	// calculate delegation rewards for del2
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del2, endingPeriod)
	require.NoError(t, err)

	// rewards for del2 should be 1/4 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 4)}}, rewards)

	// commission should be half initial
	valCommission, err = distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 2)}}, valCommission.Commission)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some more rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// withdraw commission
	expCommission = sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(initial))}
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(ctx, disttypes.ModuleName, addr, expCommission)
	_, err = distrKeeper.WithdrawValidatorCommission(ctx, valAddr)
	require.NoError(t, err)

	// end period
	endingPeriod, _ = distrKeeper.IncrementValidatorPeriod(ctx, val)

	// calculate delegation rewards for del1
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	require.NoError(t, err)

	// rewards for del1 should be 1/4 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 4)}}, rewards)

	// calculate delegation rewards for del2
	rewards, err = distrKeeper.CalculateDelegationRewards(ctx, val, del2, endingPeriod)
	require.NoError(t, err)

	// rewards for del2 should be 1/2 initial
	require.Equal(t, sdk.DecCoins{{Denom: sdk.DefaultBondDenom, Amount: math.LegacyNewDec(initial / 2)}}, rewards)

	// commission should be zero
	valCommission, err = distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	require.True(t, valCommission.Commission.IsZero())
}

func Test100PercentCommissionReward(t *testing.T) {
	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(disttypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec(sdk.Bech32PrefixValAddr)).AnyTimes()
	stakingKeeper.EXPECT().BondDenom(gomock.Any()).Return("stake", nil).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec(sdk.Bech32MainPrefix)).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	// reset fee pool
	require.NoError(t, distrKeeper.FeePool.Set(ctx, disttypes.InitialFeePool()))
	require.NoError(t, distrKeeper.Params.Set(ctx, disttypes.DefaultParams()))

	// create validator with 50% commission
	valAddr := sdk.ValAddress(valConsAddr0)
	addr := sdk.AccAddress(valAddr)
	operatorAddr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valConsPk0.Address())
	require.NoError(t, err)
	val, err := distrtestutil.CreateValidator(valConsPk0, operatorAddr, math.NewInt(100))
	require.NoError(t, err)
	val.Commission = stakingtypes.NewCommission(math.LegacyNewDecWithPrec(10, 1), math.LegacyNewDecWithPrec(10, 1), math.LegacyNewDec(0))

	addrStr, err := accountKeeper.AddressCodec().BytesToString(addr)
	require.NoError(t, err)
	valAddrStr, err := stakingKeeper.ValidatorAddressCodec().BytesToString(valAddr)
	require.NoError(t, err)

	// validator and delegation mocks
	del := stakingtypes.NewDelegation(addrStr, valAddrStr, val.DelegatorShares)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(3)
	stakingKeeper.EXPECT().Delegation(gomock.Any(), addr, valAddr).Return(del, nil).Times(3)

	// run the necessary hooks manually (given that we are not running an actual staking module)
	err = distrtestutil.CallCreateValidatorHooks(ctx, distrKeeper, addr, valAddr)
	require.NoError(t, err)
	stakingKeeper.EXPECT().Validator(gomock.Any(), valAddr).Return(val, nil).Times(2)

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some rewards
	initial := int64(20)
	tokens := sdk.DecCoins{sdk.NewDecCoin(sdk.DefaultBondDenom, math.NewInt(initial))}
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some more rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	// next block
	ctx = ctx.WithHeaderInfo(header.Info{Height: ctx.HeaderInfo().Height + 1})

	// allocate some more rewards
	require.NoError(t, distrKeeper.AllocateTokensToValidator(ctx, val, tokens))

	rewards, err := distrKeeper.WithdrawDelegationRewards(ctx, addr, valAddr)
	require.NoError(t, err)

	zeroRewards := sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, math.ZeroInt())}
	require.True(t, rewards.Equal(zeroRewards))

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
