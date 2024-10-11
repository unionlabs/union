package types_test

import (
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	storetypes "cosmossdk.io/store/types"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authcodec "github.com/cosmos/cosmos-sdk/x/auth/codec"
	"github.com/cosmos/cosmos-sdk/x/auth/keeper"
	authtestutil "github.com/cosmos/cosmos-sdk/x/auth/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
)

var (
	stakeDenom = "stake"
	feeDenom   = "fee"
	emptyCoins = sdk.Coins{}
)

type VestingAccountTestSuite struct {
	suite.Suite

	ctx           sdk.Context
	accountKeeper keeper.AccountKeeper
}

func (s *VestingAccountTestSuite) SetupTest() {
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, vesting.AppModule{})

	key := storetypes.NewKVStoreKey(authtypes.StoreKey)
	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())
	testCtx := testutil.DefaultContextWithDB(s.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	s.ctx = testCtx.Ctx.WithHeaderInfo(header.Info{})

	// gomock initializations
	ctrl := gomock.NewController(&testing.T{})

	maccPerms := map[string][]string{
		"fee_collector":          nil,
		"mint":                   {"minter"},
		"bonded_tokens_pool":     {"burner", "staking"},
		"not_bonded_tokens_pool": {"burner", "staking"},
		"multiPerm":              {"burner", "minter", "staking"},
		"random":                 {"random"},
	}

	s.accountKeeper = keeper.NewAccountKeeper(
		env,
		encCfg.Codec,
		authtypes.ProtoBaseAccount,
		authtestutil.NewMockAccountsModKeeper(ctrl),
		maccPerms,
		authcodec.NewBech32Codec("cosmos"),
		"cosmos",
		authtypes.NewModuleAddress("gov").String(),
	)
}

func TestGetVestedCoinsContVestingAcc(t *testing.T) {
	now := time.Now()
	startTime := now.Add(24 * time.Hour)
	endTime := startTime.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()
	cva, err := types.NewContinuousVestingAccount(bacc, origCoins, startTime.Unix(), endTime.Unix())
	require.NoError(t, err)

	// require no coins vested _before_ the start time of the vesting schedule
	vestedCoins := cva.GetVestedCoins(now)
	require.Nil(t, vestedCoins)

	// require no coins vested _before_ the very beginning of the vesting schedule
	vestedCoins = cva.GetVestedCoins(startTime.Add(-1))
	require.Nil(t, vestedCoins)

	// require all coins vested at the end of the vesting schedule
	vestedCoins = cva.GetVestedCoins(endTime)
	require.Equal(t, origCoins, vestedCoins)

	// require 50% of coins vested
	t50 := time.Duration(0.5 * float64(endTime.Sub(startTime)))
	vestedCoins = cva.GetVestedCoins(startTime.Add(t50))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}, vestedCoins)

	// require 75% of coins vested
	t75 := time.Duration(0.75 * float64(endTime.Sub(startTime)))
	vestedCoins = cva.GetVestedCoins(startTime.Add(t75))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 750), sdk.NewInt64Coin(stakeDenom, 75)}, vestedCoins)

	// require 100% of coins vested
	vestedCoins = cva.GetVestedCoins(endTime)
	require.Equal(t, origCoins, vestedCoins)
}

func TestGetVestingCoinsContVestingAcc(t *testing.T) {
	now := time.Now()
	startTime := now.Add(24 * time.Hour)
	endTime := startTime.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()
	cva, err := types.NewContinuousVestingAccount(bacc, origCoins, startTime.Unix(), endTime.Unix())
	require.NoError(t, err)

	// require all coins vesting before the start time of the vesting schedule
	vestingCoins := cva.GetVestingCoins(now)
	require.Equal(t, origCoins, vestingCoins)

	// require all coins vesting right before the start time of the vesting schedule
	vestingCoins = cva.GetVestingCoins(startTime.Add(-1))
	require.Equal(t, origCoins, vestingCoins)

	// require no coins vesting at the end of the vesting schedule
	vestingCoins = cva.GetVestingCoins(endTime)
	require.Equal(t, emptyCoins, vestingCoins)

	// require 50% of coins vesting in the middle between start and end time
	t50 := time.Duration(0.5 * float64(endTime.Sub(startTime)))
	vestingCoins = cva.GetVestingCoins(startTime.Add(t50))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}, vestingCoins)

	// require 25% of coins vesting after 3/4 of the time between start and end time has passed
	t75 := time.Duration(0.75 * float64(endTime.Sub(startTime)))
	vestingCoins = cva.GetVestingCoins(startTime.Add(t75))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}, vestingCoins)
}

func TestSpendableCoinsContVestingAcc(t *testing.T) {
	now := time.Now()
	startTime := now.Add(24 * time.Hour)
	endTime := startTime.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()
	cva, err := types.NewContinuousVestingAccount(bacc, origCoins, startTime.Unix(), endTime.Unix())
	require.NoError(t, err)

	// require that all original coins are locked before the beginning of the vesting
	// schedule
	lockedCoins := cva.LockedCoins(now)
	require.Equal(t, origCoins, lockedCoins)

	// require that all original coins are locked at the beginning of the vesting
	// schedule
	lockedCoins = cva.LockedCoins(startTime)
	require.Equal(t, origCoins, lockedCoins)

	// require that there exist no locked coins in the end of the vesting schedule
	lockedCoins = cva.LockedCoins(endTime)
	require.Equal(t, sdk.NewCoins(), lockedCoins)

	// require that all vested coins (50%) are spendable
	lockedCoins = cva.LockedCoins(startTime.Add(12 * time.Hour))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}, lockedCoins)

	// require 25% of coins vesting after 3/4 of the time between start and end time has passed
	lockedCoins = cva.LockedCoins(startTime.Add(18 * time.Hour))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}, lockedCoins)
}

func TestTrackDelegationContVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require the ability to delegate all vesting coins
	cva, err := types.NewContinuousVestingAccount(bacc, origCoins, now.Unix(), endTime.Unix())
	require.NoError(t, err)
	cva.TrackDelegation(now, origCoins, origCoins)
	require.Equal(t, origCoins, cva.DelegatedVesting)
	require.Nil(t, cva.DelegatedFree)

	// require the ability to delegate all vested coins
	cva, err = types.NewContinuousVestingAccount(bacc, origCoins, now.Unix(), endTime.Unix())
	require.NoError(t, err)
	cva.TrackDelegation(endTime, origCoins, origCoins)
	require.Nil(t, cva.DelegatedVesting)
	require.Equal(t, origCoins, cva.DelegatedFree)

	// require the ability to delegate all vesting coins (50%) and all vested coins (50%)
	cva, err = types.NewContinuousVestingAccount(bacc, origCoins, now.Unix(), endTime.Unix())
	require.NoError(t, err)
	cva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)}, cva.DelegatedVesting)
	require.Nil(t, cva.DelegatedFree)

	cva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)}, cva.DelegatedVesting)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)}, cva.DelegatedFree)

	// require no modifications when delegation amount is zero or not enough funds
	cva, err = types.NewContinuousVestingAccount(bacc, origCoins, now.Unix(), endTime.Unix())
	require.NoError(t, err)
	require.Panics(t, func() {
		cva.TrackDelegation(endTime, origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 1000000)})
	})
	require.Nil(t, cva.DelegatedVesting)
	require.Nil(t, cva.DelegatedFree)
}

func TestTrackUndelegationContVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require the ability to undelegate all vesting coins
	cva, err := types.NewContinuousVestingAccount(bacc, origCoins, now.Unix(), endTime.Unix())
	require.NoError(t, err)
	cva.TrackDelegation(now, origCoins, origCoins)
	cva.TrackUndelegation(origCoins)
	require.Nil(t, cva.DelegatedFree)
	require.Equal(t, emptyCoins, cva.DelegatedVesting)

	// require the ability to undelegate all vested coins
	cva, err = types.NewContinuousVestingAccount(bacc, origCoins, now.Unix(), endTime.Unix())
	require.NoError(t, err)
	cva.TrackDelegation(endTime, origCoins, origCoins)
	cva.TrackUndelegation(origCoins)
	require.Equal(t, emptyCoins, cva.DelegatedFree)
	require.Nil(t, cva.DelegatedVesting)

	// require no modifications when the undelegation amount is zero
	cva, err = types.NewContinuousVestingAccount(bacc, origCoins, now.Unix(), endTime.Unix())
	require.NoError(t, err)
	require.Panics(t, func() {
		cva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 0)})
	})
	require.Nil(t, cva.DelegatedFree)
	require.Nil(t, cva.DelegatedVesting)

	// vest 50% and delegate to two validators
	cva, err = types.NewContinuousVestingAccount(bacc, origCoins, now.Unix(), endTime.Unix())
	require.NoError(t, err)
	cva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	cva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})

	// undelegate from one validator that got slashed 50%
	cva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)})
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)}, cva.DelegatedFree)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)}, cva.DelegatedVesting)

	// undelegate from the other validator that did not get slashed
	cva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	require.Equal(t, emptyCoins, cva.DelegatedFree)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)}, cva.DelegatedVesting)
}

func TestGetVestedCoinsDelVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require no coins are vested until schedule maturation
	dva, err := types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	vestedCoins := dva.GetVestedCoins(now)
	require.Nil(t, vestedCoins)

	// require all coins be vested at schedule maturation
	vestedCoins = dva.GetVestedCoins(endTime)
	require.Equal(t, origCoins, vestedCoins)
}

func TestGetVestingCoinsDelVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require all coins vesting at the beginning of the schedule
	dva, err := types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	vestingCoins := dva.GetVestingCoins(now)
	require.Equal(t, origCoins, vestingCoins)

	// require no coins vesting at schedule maturation
	vestingCoins = dva.GetVestingCoins(endTime)
	require.Equal(t, emptyCoins, vestingCoins)
}

func TestSpendableCoinsDelVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require that all coins are locked in the beginning of the vesting
	// schedule
	dva, err := types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	lockedCoins := dva.LockedCoins(now)
	require.True(t, lockedCoins.Equal(origCoins))

	// require that all coins are spendable after the maturation of the vesting
	// schedule
	lockedCoins = dva.LockedCoins(endTime)
	require.Equal(t, sdk.NewCoins(), lockedCoins)

	// require that all coins are still vesting after some time
	lockedCoins = dva.LockedCoins(now.Add(12 * time.Hour))
	require.True(t, lockedCoins.Equal(origCoins))

	// delegate some locked coins
	// require that locked is reduced
	delegatedAmount := sdk.NewCoins(sdk.NewInt64Coin(stakeDenom, 50))
	dva.TrackDelegation(now.Add(12*time.Hour), origCoins, delegatedAmount)
	lockedCoins = dva.LockedCoins(now.Add(12 * time.Hour))
	require.True(t, lockedCoins.Equal(origCoins.Sub(delegatedAmount...)))
}

func TestTrackDelegationDelVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require the ability to delegate all vesting coins
	dva, err := types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	dva.TrackDelegation(now, origCoins, origCoins)
	require.Equal(t, origCoins, dva.DelegatedVesting)
	require.Nil(t, dva.DelegatedFree)

	// require the ability to delegate all vested coins
	dva, err = types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	dva.TrackDelegation(endTime, origCoins, origCoins)
	require.Nil(t, dva.DelegatedVesting)
	require.Equal(t, origCoins, dva.DelegatedFree)

	// require the ability to delegate all coins half way through the vesting
	// schedule
	dva, err = types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	dva.TrackDelegation(now.Add(12*time.Hour), origCoins, origCoins)
	require.Equal(t, origCoins, dva.DelegatedVesting)
	require.Nil(t, dva.DelegatedFree)

	// require no modifications when delegation amount is zero or not enough funds
	dva, err = types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	require.Panics(t, func() {
		dva.TrackDelegation(endTime, origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 1000000)})
	})
	require.Nil(t, dva.DelegatedVesting)
	require.Nil(t, dva.DelegatedFree)
}

func TestTrackUndelegationDelVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require the ability to undelegate all vesting coins
	dva, err := types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	dva.TrackDelegation(now, origCoins, origCoins)
	dva.TrackUndelegation(origCoins)
	require.Nil(t, dva.DelegatedFree)
	require.Equal(t, emptyCoins, dva.DelegatedVesting)

	// require the ability to undelegate all vested coins
	dva, err = types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	dva.TrackDelegation(endTime, origCoins, origCoins)
	dva.TrackUndelegation(origCoins)
	require.Equal(t, emptyCoins, dva.DelegatedFree)
	require.Nil(t, dva.DelegatedVesting)

	// require no modifications when the undelegation amount is zero
	dva, err = types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	require.Panics(t, func() {
		dva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 0)})
	})
	require.Nil(t, dva.DelegatedFree)
	require.Nil(t, dva.DelegatedVesting)

	// vest 50% and delegate to two validators
	dva, err = types.NewDelayedVestingAccount(bacc, origCoins, endTime.Unix())
	require.NoError(t, err)
	dva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	dva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})

	// undelegate from one validator that got slashed 50%
	dva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)})

	require.Nil(t, dva.DelegatedFree)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 75)}, dva.DelegatedVesting)

	// undelegate from the other validator that did not get slashed
	dva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	require.Nil(t, dva.DelegatedFree)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)}, dva.DelegatedVesting)
}

func TestGetVestedCoinsPeriodicVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)
	periods := types.Periods{
		types.Period{Length: int64(12 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
	}

	bacc, origCoins := initBaseAccount()
	pva, err := types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)

	// require no coins vested at the beginning of the vesting schedule
	vestedCoins := pva.GetVestedCoins(now)
	require.Nil(t, vestedCoins)

	// require all coins vested at the end of the vesting schedule
	vestedCoins = pva.GetVestedCoins(endTime)
	require.Equal(t, origCoins, vestedCoins)

	// require no coins vested during first vesting period
	vestedCoins = pva.GetVestedCoins(now.Add(6 * time.Hour))
	require.Nil(t, vestedCoins)

	// require 50% of coins vested after period 1
	vestedCoins = pva.GetVestedCoins(now.Add(12 * time.Hour))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}, vestedCoins)

	// require period 2 coins don't vest until period is over
	vestedCoins = pva.GetVestedCoins(now.Add(15 * time.Hour))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}, vestedCoins)

	// require 75% of coins vested after period 2
	vestedCoins = pva.GetVestedCoins(now.Add(18 * time.Hour))
	require.Equal(t,
		sdk.Coins{
			sdk.NewInt64Coin(feeDenom, 750), sdk.NewInt64Coin(stakeDenom, 75),
		}, vestedCoins)

	// require 100% of coins vested
	vestedCoins = pva.GetVestedCoins(now.Add(48 * time.Hour))
	require.Equal(t, origCoins, vestedCoins)
}

func TestOverflowAndNegativeVestedCoinsPeriods(t *testing.T) {
	now := time.Now()
	tests := []struct {
		name    string
		periods []types.Period
		wantErr string
	}{
		{
			"negative .Length",
			types.Periods{
				types.Period{Length: -1, Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}},
				types.Period{Length: 6 * 60 * 60, Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
			},
			"period #0 has a negative length: -1",
		},
		{
			"overflow after .Length additions",
			types.Periods{
				types.Period{Length: 9223372036854775108, Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}},
				types.Period{Length: 6 * 60 * 60, Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
			},
			"vesting start-time cannot be before end-time", // it overflow to a negative number, making start-time > end-time
		},
		{
			"good periods that are not negative nor overflow",
			types.Periods{
				types.Period{Length: now.Unix() - 1000, Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}},
				types.Period{Length: 60, Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
				types.Period{Length: 30, Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
			},
			"",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			bacc, origCoins := initBaseAccount()
			pva, err := types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), tt.periods)
			if tt.wantErr != "" {
				require.ErrorContains(t, err, tt.wantErr)
				return
			}

			require.NoError(t, err)
			if pbva := pva.BaseVestingAccount; pbva.EndTime < 0 {
				t.Fatalf("Unfortunately we still have negative .EndTime :-(: %d", pbva.EndTime)
			}
		})
	}
}

func TestGetVestingCoinsPeriodicVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)
	periods := types.Periods{
		types.Period{Length: int64(12 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
	}

	bacc, origCoins := initBaseAccount()
	pva, err := types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)

	// require all coins vesting at the beginning of the vesting schedule
	vestingCoins := pva.GetVestingCoins(now)
	require.Equal(t, origCoins, vestingCoins)

	// require no coins vesting at the end of the vesting schedule
	vestingCoins = pva.GetVestingCoins(endTime)
	require.Equal(t, emptyCoins, vestingCoins)

	// require 50% of coins vesting
	vestingCoins = pva.GetVestingCoins(now.Add(12 * time.Hour))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}, vestingCoins)

	// require 50% of coins vesting after period 1, but before period 2 completes.
	vestingCoins = pva.GetVestingCoins(now.Add(15 * time.Hour))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}, vestingCoins)

	// require 25% of coins vesting after period 2
	vestingCoins = pva.GetVestingCoins(now.Add(18 * time.Hour))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}, vestingCoins)

	// require 0% of coins vesting after vesting complete
	vestingCoins = pva.GetVestingCoins(now.Add(48 * time.Hour))
	require.Equal(t, emptyCoins, vestingCoins)
}

func TestSpendableCoinsPeriodicVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)
	periods := types.Periods{
		types.Period{Length: int64(12 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
	}

	bacc, origCoins := initBaseAccount()
	pva, err := types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)

	// require that there exist no spendable coins at the beginning of the
	// vesting schedule
	lockedCoins := pva.LockedCoins(now)
	require.Equal(t, origCoins, lockedCoins)

	// require that all original coins are spendable at the end of the vesting
	// schedule
	lockedCoins = pva.LockedCoins(endTime)
	require.Equal(t, sdk.NewCoins(), lockedCoins)

	// require that all still vesting coins (50%) are locked
	lockedCoins = pva.LockedCoins(now.Add(12 * time.Hour))
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}, lockedCoins)
}

func TestTrackDelegationPeriodicVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)
	periods := types.Periods{
		types.Period{Length: int64(12 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
	}

	bacc, origCoins := initBaseAccount()

	// require the ability to delegate all vesting coins
	pva, err := types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	pva.TrackDelegation(now, origCoins, origCoins)
	require.Equal(t, origCoins, pva.DelegatedVesting)
	require.Nil(t, pva.DelegatedFree)

	// require the ability to delegate all vested coins
	pva, err = types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	pva.TrackDelegation(endTime, origCoins, origCoins)
	require.Nil(t, pva.DelegatedVesting)
	require.Equal(t, origCoins, pva.DelegatedFree)

	// delegate half of vesting coins
	pva, err = types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	pva.TrackDelegation(now, origCoins, periods[0].Amount)
	// require that all delegated coins are delegated vesting
	require.Equal(t, pva.DelegatedVesting, periods[0].Amount)
	require.Nil(t, pva.DelegatedFree)

	// delegate 75% of coins, split between vested and vesting
	pva, err = types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	pva.TrackDelegation(now.Add(12*time.Hour), origCoins, periods[0].Amount.Add(periods[1].Amount...))
	// require that the maximum possible amount of vesting coins are chosen for delegation.
	require.Equal(t, pva.DelegatedFree, periods[1].Amount)
	require.Equal(t, pva.DelegatedVesting, periods[0].Amount)

	// require the ability to delegate all vesting coins (50%) and all vested coins (50%)
	pva, err = types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	pva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)}, pva.DelegatedVesting)
	require.Nil(t, pva.DelegatedFree)

	pva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)}, pva.DelegatedVesting)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)}, pva.DelegatedFree)

	// require no modifications when delegation amount is zero or not enough funds
	pva, err = types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	require.Panics(t, func() {
		pva.TrackDelegation(endTime, origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 1000000)})
	})
	require.Nil(t, pva.DelegatedVesting)
	require.Nil(t, pva.DelegatedFree)
}

func TestTrackUndelegationPeriodicVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(24 * time.Hour)
	periods := types.Periods{
		types.Period{Length: int64(12 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 500), sdk.NewInt64Coin(stakeDenom, 50)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
		types.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin(feeDenom, 250), sdk.NewInt64Coin(stakeDenom, 25)}},
	}

	bacc, origCoins := initBaseAccount()

	// require the ability to undelegate all vesting coins at the beginning of vesting
	pva, err := types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	pva.TrackDelegation(now, origCoins, origCoins)
	pva.TrackUndelegation(origCoins)
	require.Nil(t, pva.DelegatedFree)
	require.Equal(t, emptyCoins, pva.DelegatedVesting)

	// require the ability to undelegate all vested coins at the end of vesting
	pva, err = types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	pva.TrackDelegation(endTime, origCoins, origCoins)
	pva.TrackUndelegation(origCoins)
	require.Equal(t, emptyCoins, pva.DelegatedFree)
	require.Nil(t, pva.DelegatedVesting)

	// require the ability to undelegate half of coins
	pva, err = types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	pva.TrackDelegation(endTime, origCoins, periods[0].Amount)
	pva.TrackUndelegation(periods[0].Amount)
	require.Equal(t, emptyCoins, pva.DelegatedFree)
	require.Nil(t, pva.DelegatedVesting)

	// require no modifications when the undelegation amount is zero
	pva, err = types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	require.Panics(t, func() {
		pva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 0)})
	})
	require.Nil(t, pva.DelegatedFree)
	require.Nil(t, pva.DelegatedVesting)

	// vest 50% and delegate to two validators
	pva, err = types.NewPeriodicVestingAccount(bacc, origCoins, now.Unix(), periods)
	require.NoError(t, err)
	pva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	pva.TrackDelegation(now.Add(12*time.Hour), origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})

	// undelegate from one validator that got slashed 50%
	pva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)})
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)}, pva.DelegatedFree)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)}, pva.DelegatedVesting)

	// undelegate from the other validator that did not get slashed
	pva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	require.Equal(t, emptyCoins, pva.DelegatedFree)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)}, pva.DelegatedVesting)
}

func TestGetVestedCoinsPermLockedVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(1000 * 24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require no coins are vested
	plva, err := types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	vestedCoins := plva.GetVestedCoins(now)
	require.Nil(t, vestedCoins)

	// require no coins be vested at end time
	vestedCoins = plva.GetVestedCoins(endTime)
	require.Nil(t, vestedCoins)
}

func TestGetVestingCoinsPermLockedVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(1000 * 24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require all coins vesting at the beginning of the schedule
	plva, err := types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	vestingCoins := plva.GetVestingCoins(now)
	require.Equal(t, origCoins, vestingCoins)

	// require all coins vesting at the end time
	vestingCoins = plva.GetVestingCoins(endTime)
	require.Equal(t, origCoins, vestingCoins)
}

func TestSpendableCoinsPermLockedVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(1000 * 24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require that all coins are locked in the beginning of the vesting
	// schedule
	plva, err := types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	lockedCoins := plva.LockedCoins(now)
	require.True(t, lockedCoins.Equal(origCoins))

	// require that all coins are still locked at end time
	lockedCoins = plva.LockedCoins(endTime)
	require.True(t, lockedCoins.Equal(origCoins))

	// delegate some locked coins
	// require that locked is reduced
	delegatedAmount := sdk.NewCoins(sdk.NewInt64Coin(stakeDenom, 50))
	plva.TrackDelegation(now.Add(12*time.Hour), origCoins, delegatedAmount)
	lockedCoins = plva.LockedCoins(now.Add(12 * time.Hour))
	require.True(t, lockedCoins.Equal(origCoins.Sub(delegatedAmount...)))
}

func TestTrackDelegationPermLockedVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(1000 * 24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require the ability to delegate all vesting coins
	plva, err := types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	plva.TrackDelegation(now, origCoins, origCoins)
	require.Equal(t, origCoins, plva.DelegatedVesting)
	require.Nil(t, plva.DelegatedFree)

	// require the ability to delegate all vested coins at endTime
	plva, err = types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	plva.TrackDelegation(endTime, origCoins, origCoins)
	require.Equal(t, origCoins, plva.DelegatedVesting)
	require.Nil(t, plva.DelegatedFree)

	// require no modifications when delegation amount is zero or not enough funds
	plva, err = types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	require.Panics(t, func() {
		plva.TrackDelegation(endTime, origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 1000000)})
	})
	require.Nil(t, plva.DelegatedVesting)
	require.Nil(t, plva.DelegatedFree)
}

func TestTrackUndelegationPermLockedVestingAcc(t *testing.T) {
	now := time.Now()
	endTime := now.Add(1000 * 24 * time.Hour)

	bacc, origCoins := initBaseAccount()

	// require the ability to undelegate all vesting coins
	plva, err := types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	plva.TrackDelegation(now, origCoins, origCoins)
	plva.TrackUndelegation(origCoins)
	require.Nil(t, plva.DelegatedFree)
	require.Equal(t, emptyCoins, plva.DelegatedVesting)

	// require the ability to undelegate all vesting coins at endTime
	plva, err = types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	plva.TrackDelegation(endTime, origCoins, origCoins)
	plva.TrackUndelegation(origCoins)
	require.Nil(t, plva.DelegatedFree)
	require.Equal(t, emptyCoins, plva.DelegatedVesting)

	// require no modifications when the undelegation amount is zero
	plva, err = types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	require.Panics(t, func() {
		plva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 0)})
	})
	require.Nil(t, plva.DelegatedFree)
	require.Nil(t, plva.DelegatedVesting)

	// delegate to two validators
	plva, err = types.NewPermanentLockedAccount(bacc, origCoins)
	require.NoError(t, err)
	plva.TrackDelegation(now, origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	plva.TrackDelegation(now, origCoins, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})

	// undelegate from one validator that got slashed 50%
	plva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)})

	require.Nil(t, plva.DelegatedFree)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 75)}, plva.DelegatedVesting)

	// undelegate from the other validator that did not get slashed
	plva.TrackUndelegation(sdk.Coins{sdk.NewInt64Coin(stakeDenom, 50)})
	require.Nil(t, plva.DelegatedFree)
	require.Equal(t, sdk.Coins{sdk.NewInt64Coin(stakeDenom, 25)}, plva.DelegatedVesting)
}

func TestGenesisAccountValidate(t *testing.T) {
	pubkey := secp256k1.GenPrivKey().PubKey()
	addr := sdk.AccAddress(pubkey.Address())
	baseAcc := authtypes.NewBaseAccount(addr, pubkey, 0, 0)
	initialVesting := sdk.NewCoins(sdk.NewInt64Coin(sdk.DefaultBondDenom, 50))
	baseVestingWithCoins, err := types.NewBaseVestingAccount(baseAcc, initialVesting, 100)
	require.NoError(t, err)
	tests := []struct {
		name   string
		acc    authtypes.GenesisAccount
		expErr bool
	}{
		{
			"valid base account",
			baseAcc,
			false,
		},
		{
			"invalid base valid account",
			authtypes.NewBaseAccount(addr, secp256k1.GenPrivKey().PubKey(), 0, 0),
			true,
		},
		{
			"valid base vesting account",
			baseVestingWithCoins,
			false,
		},
		{
			"valid continuous vesting account",
			func() authtypes.GenesisAccount {
				acc, _ := types.NewContinuousVestingAccount(baseAcc, initialVesting, 100, 200)
				return acc
			}(),
			false,
		},
		{
			"invalid vesting times",
			func() authtypes.GenesisAccount {
				acc, _ := types.NewContinuousVestingAccount(baseAcc, initialVesting, 1654668078, 1554668078)
				return acc
			}(),
			true,
		},
		{
			"valid periodic vesting account",
			func() authtypes.GenesisAccount {
				acc, _ := types.NewPeriodicVestingAccount(baseAcc, initialVesting, 0, types.Periods{types.Period{Length: int64(100), Amount: sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 50)}}})
				return acc
			}(),
			false,
		},
		{
			"invalid vesting period lengths",
			types.NewPeriodicVestingAccountRaw(
				baseVestingWithCoins,
				0, types.Periods{types.Period{Length: int64(50), Amount: sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 50)}}}),
			true,
		},
		{
			"invalid vesting period amounts",
			types.NewPeriodicVestingAccountRaw(
				baseVestingWithCoins,
				0, types.Periods{types.Period{Length: int64(100), Amount: sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 25)}}}),
			true,
		},
		{
			"valid permanent locked vesting account",
			func() authtypes.GenesisAccount {
				acc, _ := types.NewPermanentLockedAccount(baseAcc, initialVesting)
				return acc
			}(),
			false,
		},
		{
			"invalid positive end time for permanently locked vest account",
			&types.PermanentLockedAccount{BaseVestingAccount: baseVestingWithCoins},
			true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			require.Equal(t, tt.expErr, tt.acc.Validate() != nil)
		})
	}
}

func initBaseAccount() (*authtypes.BaseAccount, sdk.Coins) {
	_, _, addr := testdata.KeyTestPubAddr()
	origCoins := sdk.Coins{sdk.NewInt64Coin(feeDenom, 1000), sdk.NewInt64Coin(stakeDenom, 100)}
	bacc := authtypes.NewBaseAccountWithAddress(addr)

	return bacc, origCoins
}

func TestVestingAccountTestSuite(t *testing.T) {
	suite.Run(t, new(VestingAccountTestSuite))
}
