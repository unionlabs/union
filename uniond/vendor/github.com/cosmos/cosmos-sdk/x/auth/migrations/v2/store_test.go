package v2_test

import (
	"fmt"
	"testing"
	"time"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth"
	authexported "github.com/cosmos/cosmos-sdk/x/auth/exported"
	"github.com/cosmos/cosmos-sdk/x/auth/keeper"
	v1 "github.com/cosmos/cosmos-sdk/x/auth/migrations/v1"
	v4 "github.com/cosmos/cosmos-sdk/x/auth/migrations/v4"
	authtestutil "github.com/cosmos/cosmos-sdk/x/auth/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting/exported"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	"github.com/cosmos/cosmos-sdk/x/staking"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

type mockSubspace struct {
	ps authtypes.Params
}

func newMockSubspace(ps authtypes.Params) mockSubspace {
	return mockSubspace{ps: ps}
}

func (ms mockSubspace) GetParamSet(ctx sdk.Context, ps authexported.ParamSet) {
	*ps.(*authtypes.Params) = ms.ps
}

func TestMigrateVestingAccounts(t *testing.T) {
	encCfg := moduletestutil.MakeTestEncodingConfig(auth.AppModuleBasic{})
	cdc := encCfg.Codec

	storeKey := sdk.NewKVStoreKey(v1.ModuleName)
	tKey := sdk.NewTransientStoreKey("transient_test")
	ctx := testutil.DefaultContext(storeKey, tKey)
	store := ctx.KVStore(storeKey)

	var (
		accountKeeper keeper.AccountKeeper
		bankKeeper    bankkeeper.Keeper
		stakingKeeper *stakingkeeper.Keeper
	)
	app, err := simtestutil.Setup(
		authtestutil.AppConfig,
		&accountKeeper,
		&bankKeeper,
		&stakingKeeper,
	)
	require.NoError(t, err)

	legacySubspace := newMockSubspace(authtypes.DefaultParams())
	require.NoError(t, v4.Migrate(ctx, store, legacySubspace, cdc))

	ctx = app.BaseApp.NewContext(false, tmproto.Header{Time: time.Now()})
	stakingKeeper.SetParams(ctx, stakingtypes.DefaultParams())

	testCases := []struct {
		name        string
		prepareFunc func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress)
		garbageFunc func(ctx sdk.Context, vesting exported.VestingAccount, accounKeeper keeper.AccountKeeper) error
		tokenAmount int64
		expVested   int64
		expFree     int64
		blockTime   int64
	}{
		{
			"delayed vesting has vested, multiple delegations less than the total account balance",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(200)))
				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().Unix())

				ctx = ctx.WithBlockTime(ctx.BlockTime().AddDate(1, 0, 0))

				err := accountKeeper.SetParams(ctx, authtypes.DefaultParams())
				require.NoError(t, err)

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			0,
			300,
			0,
		},
		{
			"delayed vesting has vested, single delegations which exceed the vested amount",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				require.NoError(t, err)
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(200)))
				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().Unix())

				ctx = ctx.WithBlockTime(ctx.BlockTime().AddDate(1, 0, 0))

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(300), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			0,
			300,
			0,
		},
		{
			"delayed vesting has vested, multiple delegations which exceed the vested amount",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(200)))
				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().Unix())

				ctx = ctx.WithBlockTime(ctx.BlockTime().AddDate(1, 0, 0))

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			0,
			300,
			0,
		},
		{
			"delayed vesting has not vested, single delegations  which exceed the vested amount",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(200)))
				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().AddDate(1, 0, 0).Unix())

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(300), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			200,
			100,
			0,
		},
		{
			"delayed vesting has not vested, multiple delegations which exceed the vested amount",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(200)))
				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().AddDate(1, 0, 0).Unix())

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			200,
			100,
			0,
		},
		{
			"not end time",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(300)))
				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().AddDate(1, 0, 0).Unix())

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(100), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			300,
			0,
			0,
		},
		{
			"delayed vesting has not vested, single delegation greater than the total account balance",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(300)))
				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().AddDate(1, 0, 0).Unix())

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(300), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			300,
			0,
			0,
		},
		{
			"delayed vesting has vested, single delegation greater than the total account balance",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(300)))
				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().Unix())

				ctx = ctx.WithBlockTime(ctx.BlockTime().AddDate(1, 0, 0))

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(300), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			0,
			300,
			0,
		},
		{
			"continuous vesting, start time after blocktime",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				startTime := ctx.BlockTime().AddDate(1, 0, 0).Unix()
				endTime := ctx.BlockTime().AddDate(2, 0, 0).Unix()
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(300)))
				delayedAccount := types.NewContinuousVestingAccount(baseAccount, vestedCoins, startTime, endTime)

				ctx = ctx.WithBlockTime(ctx.BlockTime().AddDate(1, 0, 0))

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(300), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			300,
			0,
			0,
		},
		{
			"continuous vesting, start time passed but not ended",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				startTime := ctx.BlockTime().AddDate(-1, 0, 0).Unix()
				endTime := ctx.BlockTime().AddDate(2, 0, 0).Unix()
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(300)))
				delayedAccount := types.NewContinuousVestingAccount(baseAccount, vestedCoins, startTime, endTime)

				ctx = ctx.WithBlockTime(ctx.BlockTime().AddDate(1, 0, 0))

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(300), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			200,
			100,
			0,
		},
		{
			"continuous vesting, start time and endtime passed",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				startTime := ctx.BlockTime().AddDate(-2, 0, 0).Unix()
				endTime := ctx.BlockTime().AddDate(-1, 0, 0).Unix()
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(300)))
				delayedAccount := types.NewContinuousVestingAccount(baseAccount, vestedCoins, startTime, endTime)

				ctx = ctx.WithBlockTime(ctx.BlockTime().AddDate(1, 0, 0))

				accountKeeper.SetAccount(ctx, delayedAccount)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(300), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			0,
			300,
			0,
		},
		{
			"periodic vesting account, yet to be vested, some rewards delegated",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(100)))

				start := ctx.BlockTime().Unix() + int64(time.Hour/time.Second)

				periods := []types.Period{
					{
						Length: int64((24 * time.Hour) / time.Second),
						Amount: vestedCoins,
					},
				}

				account := types.NewPeriodicVestingAccount(baseAccount, vestedCoins, start, periods)

				accountKeeper.SetAccount(ctx, account)

				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(150), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			300,
			100,
			50,
			0,
		},
		{
			"periodic vesting account, nothing has vested yet",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				/*
					Test case:
					 - periodic vesting account starts at time 1601042400
					 - account balance and original vesting: 3666666670000
					 - nothing has vested, we put the block time slightly after start time
					 - expected vested: original vesting amount
					 - expected free: zero
					 - we're delegating the full original vesting
				*/
				startTime := int64(1601042400)
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(3666666670000)))
				periods := []types.Period{
					{
						Length: 31536000,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(1833333335000))),
					},
					{
						Length: 15638400,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(916666667500))),
					},
					{
						Length: 15897600,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(916666667500))),
					},
				}

				delayedAccount := types.NewPeriodicVestingAccount(baseAccount, vestedCoins, startTime, periods)

				accountKeeper.SetAccount(ctx, delayedAccount)

				// delegation of the original vesting
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(3666666670000), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			3666666670000,
			3666666670000,
			0,
			1601042400 + 1,
		},
		{
			"periodic vesting account, all has vested",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				/*
					Test case:
					 - periodic vesting account starts at time 1601042400
					 - account balance and original vesting: 3666666670000
					 - all has vested, so we set the block time at initial time + sum of all periods times + 1 => 1601042400 + 31536000 + 15897600 + 15897600 + 1
					 - expected vested: zero
					 - expected free: original vesting amount
					 - we're delegating the full original vesting
				*/
				startTime := int64(1601042400)
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(3666666670000)))
				periods := []types.Period{
					{
						Length: 31536000,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(1833333335000))),
					},
					{
						Length: 15638400,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(916666667500))),
					},
					{
						Length: 15897600,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(916666667500))),
					},
				}

				delayedAccount := types.NewPeriodicVestingAccount(baseAccount, vestedCoins, startTime, periods)

				ctx = ctx.WithBlockTime(time.Unix(1601042400+31536000+15897600+15897600+1, 0))

				accountKeeper.SetAccount(ctx, delayedAccount)

				// delegation of the original vesting
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(3666666670000), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			3666666670000,
			0,
			3666666670000,
			1601042400 + 31536000 + 15897600 + 15897600 + 1,
		},
		{
			"periodic vesting account, first period has vested",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				/*
					Test case:
					 - periodic vesting account starts at time 1601042400
					 - account balance and original vesting: 3666666670000
					 - first period have vested, so we set the block time at initial time + time of the first periods + 1 => 1601042400 + 31536000 + 1
					 - expected vested: original vesting - first period amount
					 - expected free: first period amount
					 - we're delegating the full original vesting
				*/
				startTime := int64(1601042400)
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(3666666670000)))
				periods := []types.Period{
					{
						Length: 31536000,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(1833333335000))),
					},
					{
						Length: 15638400,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(916666667500))),
					},
					{
						Length: 15897600,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(916666667500))),
					},
				}

				delayedAccount := types.NewPeriodicVestingAccount(baseAccount, vestedCoins, startTime, periods)

				ctx = ctx.WithBlockTime(time.Unix(1601042400+31536000+1, 0))

				accountKeeper.SetAccount(ctx, delayedAccount)

				// delegation of the original vesting
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(3666666670000), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			3666666670000,
			3666666670000 - 1833333335000,
			1833333335000,
			1601042400 + 31536000 + 1,
		},
		{
			"periodic vesting account, first 2 period has vested",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				/*
					Test case:
					 - periodic vesting account starts at time 1601042400
					 - account balance and original vesting: 3666666670000
					 - first 2 periods have vested, so we set the block time at initial time + time of the two periods + 1 => 1601042400 + 31536000 + 15638400 + 1
					 - expected vested: original vesting - (sum of the first two periods amounts)
					 - expected free: sum of the first two periods
					 - we're delegating the full original vesting
				*/
				startTime := int64(1601042400)
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(3666666670000)))
				periods := []types.Period{
					{
						Length: 31536000,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(1833333335000))),
					},
					{
						Length: 15638400,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(916666667500))),
					},
					{
						Length: 15897600,
						Amount: sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(916666667500))),
					},
				}

				delayedAccount := types.NewPeriodicVestingAccount(baseAccount, vestedCoins, startTime, periods)

				ctx = ctx.WithBlockTime(time.Unix(1601042400+31536000+15638400+1, 0))

				accountKeeper.SetAccount(ctx, delayedAccount)

				// delegation of the original vesting
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(3666666670000), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)
			},
			cleartTrackingFields,
			3666666670000,
			3666666670000 - 1833333335000 - 916666667500,
			1833333335000 + 916666667500,
			1601042400 + 31536000 + 15638400 + 1,
		},
		{
			"vesting account has unbonding delegations in place",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(300)))

				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().AddDate(10, 0, 0).Unix())

				accountKeeper.SetAccount(ctx, delayedAccount)

				// delegation of the original vesting
				_, err = stakingKeeper.Delegate(ctx, delegatorAddr, sdk.NewInt(300), stakingtypes.Unbonded, validator, true)
				require.NoError(t, err)

				ctx = ctx.WithBlockTime(ctx.BlockTime().AddDate(1, 0, 0))

				valAddr, err := sdk.ValAddressFromBech32(validator.OperatorAddress)
				require.NoError(t, err)

				// un-delegation of the original vesting
				_, err = stakingKeeper.Undelegate(ctx, delegatorAddr, valAddr, sdk.NewDecFromInt(sdk.NewInt(300)))
				require.NoError(t, err)
			},
			cleartTrackingFields,
			450,
			300,
			0,
			0,
		},
		{
			"vesting account has never delegated anything",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(300)))

				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().AddDate(10, 0, 0).Unix())

				accountKeeper.SetAccount(ctx, delayedAccount)
			},
			cleartTrackingFields,
			450,
			0,
			0,
			0,
		},
		{
			"vesting account has no delegation but dirty DelegatedFree and DelegatedVesting fields",
			func(ctx sdk.Context, validator stakingtypes.Validator, delegatorAddr sdk.AccAddress) {
				baseAccount := authtypes.NewBaseAccountWithAddress(delegatorAddr)
				vestedCoins := sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(300)))

				delayedAccount := types.NewDelayedVestingAccount(baseAccount, vestedCoins, ctx.BlockTime().AddDate(10, 0, 0).Unix())

				accountKeeper.SetAccount(ctx, delayedAccount)
			},
			dirtyTrackingFields,
			450,
			0,
			0,
			0,
		},
	}

	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			err := accountKeeper.SetParams(ctx, authtypes.DefaultParams())
			require.NoError(t, err)

			addrs := simtestutil.AddTestAddrs(bankKeeper, stakingKeeper, ctx, 1, sdk.NewInt(tc.tokenAmount))
			delegatorAddr := addrs[0]

			_, valAddr := createValidator(t, ctx, bankKeeper, stakingKeeper, tc.tokenAmount*2)
			validator, found := stakingKeeper.GetValidator(ctx, valAddr)
			require.True(t, found)

			tc.prepareFunc(ctx, validator, delegatorAddr)

			if tc.blockTime != 0 {
				ctx = ctx.WithBlockTime(time.Unix(tc.blockTime, 0))
			}

			// We introduce the bug
			savedAccount := accountKeeper.GetAccount(ctx, delegatorAddr)
			vestingAccount, ok := savedAccount.(exported.VestingAccount)
			require.True(t, ok)
			require.NoError(t, tc.garbageFunc(ctx, vestingAccount, accountKeeper))

			m := keeper.NewMigrator(accountKeeper, app.GRPCQueryRouter(), legacySubspace)
			require.NoError(t, m.Migrate1to2(ctx))

			var expVested sdk.Coins
			var expFree sdk.Coins

			if tc.expVested != 0 {
				expVested = sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(tc.expVested)))
			}

			if tc.expFree != 0 {
				expFree = sdk.NewCoins(sdk.NewCoin(stakingKeeper.BondDenom(ctx), sdk.NewInt(tc.expFree)))
			}

			trackingCorrected(
				ctx,
				t,
				accountKeeper,
				savedAccount.GetAddress(),
				expVested,
				expFree,
			)
		})
	}
}

func trackingCorrected(ctx sdk.Context, t *testing.T, ak keeper.AccountKeeper, addr sdk.AccAddress, expDelVesting sdk.Coins, expDelFree sdk.Coins) {
	t.Helper()
	baseAccount := ak.GetAccount(ctx, addr)
	vDA, ok := baseAccount.(exported.VestingAccount)
	require.True(t, ok)

	vestedOk := expDelVesting.IsEqual(vDA.GetDelegatedVesting())
	freeOk := expDelFree.IsEqual(vDA.GetDelegatedFree())
	require.True(t, vestedOk, vDA.GetDelegatedVesting().String())
	require.True(t, freeOk, vDA.GetDelegatedFree().String())
}

func cleartTrackingFields(ctx sdk.Context, vesting exported.VestingAccount, accountKeeper keeper.AccountKeeper) error {
	switch t := vesting.(type) {
	case *types.DelayedVestingAccount:
		t.DelegatedFree = nil
		t.DelegatedVesting = nil
		accountKeeper.SetAccount(ctx, t)
	case *types.ContinuousVestingAccount:
		t.DelegatedFree = nil
		t.DelegatedVesting = nil
		accountKeeper.SetAccount(ctx, t)
	case *types.PeriodicVestingAccount:
		t.DelegatedFree = nil
		t.DelegatedVesting = nil
		accountKeeper.SetAccount(ctx, t)
	default:
		return fmt.Errorf("expected vesting account, found %t", t)
	}

	return nil
}

func dirtyTrackingFields(ctx sdk.Context, vesting exported.VestingAccount, accountKeeper keeper.AccountKeeper) error {
	dirt := sdk.NewCoins(sdk.NewInt64Coin("stake", 42))

	switch t := vesting.(type) {
	case *types.DelayedVestingAccount:
		t.DelegatedFree = dirt
		t.DelegatedVesting = dirt
		accountKeeper.SetAccount(ctx, t)
	case *types.ContinuousVestingAccount:
		t.DelegatedFree = dirt
		t.DelegatedVesting = dirt
		accountKeeper.SetAccount(ctx, t)
	case *types.PeriodicVestingAccount:
		t.DelegatedFree = dirt
		t.DelegatedVesting = dirt
		accountKeeper.SetAccount(ctx, t)
	default:
		return fmt.Errorf("expected vesting account, found %t", t)
	}

	return nil
}

func createValidator(t *testing.T, ctx sdk.Context, bankKeeper bankkeeper.Keeper, stakingKeeper *stakingkeeper.Keeper, powers int64) (sdk.AccAddress, sdk.ValAddress) {
	valTokens := sdk.TokensFromConsensusPower(powers, sdk.DefaultPowerReduction)
	addrs := simtestutil.AddTestAddrsIncremental(bankKeeper, stakingKeeper, ctx, 1, valTokens)
	valAddrs := simtestutil.ConvertAddrsToValAddrs(addrs)
	pks := simtestutil.CreateTestPubKeys(1)

	val1, err := stakingtypes.NewValidator(valAddrs[0], pks[0], stakingtypes.Description{})
	require.NoError(t, err)

	stakingKeeper.SetValidator(ctx, val1)
	require.NoError(t, stakingKeeper.SetValidatorByConsAddr(ctx, val1))
	stakingKeeper.SetNewValidatorByPowerIndex(ctx, val1)

	_, err = stakingKeeper.Delegate(ctx, addrs[0], valTokens, stakingtypes.Unbonded, val1, true)
	require.NoError(t, err)

	_ = staking.EndBlocker(ctx, stakingKeeper)

	return addrs[0], valAddrs[0]
}
