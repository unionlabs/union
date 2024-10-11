package keeper_test

import (
	"context"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/appmodule"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/mint"
	"cosmossdk.io/x/mint/keeper"
	minttestutil "cosmossdk.io/x/mint/testutil"
	"cosmossdk.io/x/mint/types"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

const govModuleNameStr = "cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn"

type KeeperTestSuite struct {
	suite.Suite

	mintKeeper    keeper.Keeper
	ctx           sdk.Context
	msgServer     types.MsgServer
	stakingKeeper *minttestutil.MockStakingKeeper
	bankKeeper    *minttestutil.MockBankKeeper
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

func (s *KeeperTestSuite) SetupTest() {
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, mint.AppModule{})
	key := storetypes.NewKVStoreKey(types.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	env := runtime.NewEnvironment(storeService, coretesting.NewNopLogger())
	testCtx := testutil.DefaultContextWithDB(s.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	s.ctx = testCtx.Ctx

	// gomock initializations
	ctrl := gomock.NewController(s.T())
	accountKeeper := minttestutil.NewMockAccountKeeper(ctrl)
	bankKeeper := minttestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := minttestutil.NewMockStakingKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress(types.ModuleName).Return(sdk.AccAddress{})

	s.mintKeeper = keeper.NewKeeper(
		encCfg.Codec,
		env,
		stakingKeeper,
		accountKeeper,
		bankKeeper,
		authtypes.FeeCollectorName,
		govModuleNameStr,
	)
	s.stakingKeeper = stakingKeeper
	s.bankKeeper = bankKeeper

	err := s.mintKeeper.Params.Set(s.ctx, types.DefaultParams())
	s.NoError(err)

	s.NoError(s.mintKeeper.Minter.Set(s.ctx, types.DefaultInitialMinter()))
	s.msgServer = keeper.NewMsgServerImpl(s.mintKeeper)
}

func (s *KeeperTestSuite) TestAliasFunctions() {
	stakingTokenSupply := math.NewIntFromUint64(100000000000)
	s.stakingKeeper.EXPECT().StakingTokenSupply(s.ctx).Return(stakingTokenSupply, nil)
	tokenSupply, err := s.mintKeeper.StakingTokenSupply(s.ctx)
	s.NoError(err)
	s.Equal(tokenSupply, stakingTokenSupply)

	bondedRatio := math.LegacyNewDecWithPrec(15, 2)
	s.stakingKeeper.EXPECT().BondedRatio(s.ctx).Return(bondedRatio, nil)
	ratio, err := s.mintKeeper.BondedRatio(s.ctx)
	s.NoError(err)
	s.Equal(ratio, bondedRatio)

	coins := sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(1000000)))
	s.bankKeeper.EXPECT().MintCoins(s.ctx, types.ModuleName, coins).Return(nil)
	s.Equal(s.mintKeeper.MintCoins(s.ctx, sdk.NewCoins()), nil)
	s.Nil(s.mintKeeper.MintCoins(s.ctx, coins))

	fees := sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(1000)))
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(s.ctx, types.ModuleName, authtypes.FeeCollectorName, fees).Return(nil)
	s.Nil(s.mintKeeper.AddCollectedFees(s.ctx, fees))
}

func (s *KeeperTestSuite) TestDefaultMintFn() {
	s.stakingKeeper.EXPECT().StakingTokenSupply(s.ctx).Return(math.NewIntFromUint64(100000000000), nil).AnyTimes()
	bondedRatio := math.LegacyNewDecWithPrec(15, 2)
	s.stakingKeeper.EXPECT().BondedRatio(s.ctx).Return(bondedRatio, nil).AnyTimes()
	s.bankKeeper.EXPECT().MintCoins(s.ctx, types.ModuleName, sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(792)))).Return(nil)
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(s.ctx, types.ModuleName, authtypes.FeeCollectorName, gomock.Any()).Return(nil)

	minter, err := s.mintKeeper.Minter.Get(s.ctx)
	s.NoError(err)

	err = s.mintKeeper.DefaultMintFn(types.DefaultInflationCalculationFn)(s.ctx, s.mintKeeper.Environment, &minter, "block", 0)
	s.NoError(err)

	// set a maxSupply and call again. totalSupply will be bigger than maxSupply.
	params, err := s.mintKeeper.Params.Get(s.ctx)
	s.NoError(err)
	params.MaxSupply = math.NewInt(10000000000)
	err = s.mintKeeper.Params.Set(s.ctx, params)
	s.NoError(err)

	err = s.mintKeeper.DefaultMintFn(types.DefaultInflationCalculationFn)(s.ctx, s.mintKeeper.Environment, &minter, "block", 0)
	s.NoError(err)

	// modify max supply to be almost reached
	// modify blocksPerYear to mint 2053 coins per block
	// we tried to mint 2053stake, but we only get to mint 2000stake
	params, err = s.mintKeeper.Params.Get(s.ctx)
	s.NoError(err)
	params.MaxSupply = math.NewInt(100000000000 + 2000)
	params.BlocksPerYear = 2434275
	err = s.mintKeeper.Params.Set(s.ctx, params)
	s.NoError(err)

	s.bankKeeper.EXPECT().MintCoins(s.ctx, types.ModuleName, sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(2000)))).Return(nil)
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(s.ctx, types.ModuleName, authtypes.FeeCollectorName, gomock.Any()).Return(nil)

	err = s.mintKeeper.DefaultMintFn(types.DefaultInflationCalculationFn)(s.ctx, s.mintKeeper.Environment, &minter, "block", 0)
	s.NoError(err)
}

func (s *KeeperTestSuite) TestBeginBlocker() {
	s.stakingKeeper.EXPECT().StakingTokenSupply(s.ctx).Return(math.NewIntFromUint64(100000000000), nil).AnyTimes()
	bondedRatio := math.LegacyNewDecWithPrec(15, 2)
	s.stakingKeeper.EXPECT().BondedRatio(s.ctx).Return(bondedRatio, nil).AnyTimes()
	s.bankKeeper.EXPECT().MintCoins(s.ctx, types.ModuleName, sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(792)))).Return(nil)
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(s.ctx, types.ModuleName, authtypes.FeeCollectorName, gomock.Any()).Return(nil)

	// get minter (it should get modified afterwards)
	minter, err := s.mintKeeper.Minter.Get(s.ctx)
	s.NoError(err)

	err = s.mintKeeper.BeginBlocker(s.ctx, s.mintKeeper.DefaultMintFn(types.DefaultInflationCalculationFn))
	s.NoError(err)

	// get minter again and compare
	newMinter, err := s.mintKeeper.Minter.Get(s.ctx)
	s.NoError(err)
	s.NotEqual(minter, newMinter)

	// now use a mintfn that doesn't do anything
	err = s.mintKeeper.BeginBlocker(s.ctx, func(ctx context.Context, env appmodule.Environment, minter *types.Minter, epochId string, epochNumber int64) error {
		return nil
	})
	s.NoError(err)

	// get minter again and compare
	unchangedMinter, err := s.mintKeeper.Minter.Get(s.ctx)
	s.NoError(err)
	s.Equal(newMinter, unchangedMinter)
}

func (s *KeeperTestSuite) TestMigrator() {
	m := keeper.NewMigrator(s.mintKeeper)
	s.NoError(m.Migrate1to2(s.ctx)) // just to get the coverage up

	// set max supply to one and migrate (should get it to zero)
	params, err := s.mintKeeper.Params.Get(s.ctx)
	s.NoError(err)
	params.MaxSupply = math.OneInt()
	s.NoError(s.mintKeeper.Params.Set(s.ctx, params))

	s.NoError(m.Migrate2to3(s.ctx))

	newParams, err := s.mintKeeper.Params.Get(s.ctx)
	s.NoError(err)
	s.Equal(math.ZeroInt(), newParams.MaxSupply)
}
