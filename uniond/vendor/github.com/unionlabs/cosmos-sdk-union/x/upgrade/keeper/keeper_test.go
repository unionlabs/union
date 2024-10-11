package keeper_test

import (
	"context"
	"path/filepath"
	"testing"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmttypes "github.com/cometbft/cometbft/types"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/upgrade"
	"cosmossdk.io/x/upgrade/keeper"
	upgradetestutil "cosmossdk.io/x/upgrade/testutil"
	"cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

type KeeperTestSuite struct {
	suite.Suite

	key              *storetypes.KVStoreKey
	baseApp          *baseapp.BaseApp
	upgradeKeeper    *keeper.Keeper
	homeDir          string
	ctx              sdk.Context
	msgSrvr          types.MsgServer
	addrs            []sdk.AccAddress
	encodedAddrs     []string
	encodedAuthority string
	encCfg           moduletestutil.TestEncodingConfig
}

func (s *KeeperTestSuite) SetupTest() {
	s.encCfg = moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, upgrade.AppModule{})
	key := storetypes.NewKVStoreKey(types.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	env := runtime.NewEnvironment(storeService, coretesting.NewNopLogger())
	s.key = key
	testCtx := testutil.DefaultContextWithDB(s.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	s.ctx = testCtx.Ctx.WithHeaderInfo(header.Info{Height: 10})

	s.baseApp = baseapp.NewBaseApp(
		"upgrade",
		log.NewNopLogger(),
		testCtx.DB,
		s.encCfg.TxConfig.TxDecoder(),
	)
	s.baseApp.SetParamStore(&paramStore{params: cmttypes.DefaultConsensusParams().ToProto()})
	s.baseApp.SetVersionModifier(newMockedVersionModifier(0))

	appVersion, err := s.baseApp.AppVersion(context.Background())
	s.Require().NoError(err)
	s.Require().Equal(uint64(0), appVersion)

	skipUpgradeHeights := make(map[int64]bool)

	homeDir := filepath.Join(s.T().TempDir(), "x_upgrade_keeper_test")
	ac := addresscodec.NewBech32Codec("cosmos")
	authority, err := ac.BytesToString(authtypes.NewModuleAddress(types.GovModuleName))
	s.Require().NoError(err)
	s.encodedAuthority = authority

	ctrl := gomock.NewController(s.T())
	s.upgradeKeeper = keeper.NewKeeper(env, skipUpgradeHeights, s.encCfg.Codec, homeDir, s.baseApp, authority, upgradetestutil.NewMockConsensusKeeper(ctrl))

	s.T().Log("home dir:", homeDir)
	s.homeDir = homeDir

	s.msgSrvr = keeper.NewMsgServerImpl(s.upgradeKeeper)
	s.addrs = simtestutil.CreateIncrementalAccounts(1)
	encodedAddr, err := ac.BytesToString(s.addrs[0].Bytes())
	s.Require().NoError(err)
	s.encodedAddrs = []string{encodedAddr}
}

func (s *KeeperTestSuite) TestReadUpgradeInfoFromDisk() {
	// require no error when the upgrade info file does not exist
	_, err := s.upgradeKeeper.ReadUpgradeInfoFromDisk()
	s.Require().NoError(err)

	expected := types.Plan{
		Name:   "test_upgrade",
		Height: 100,
	}

	// create an upgrade info file
	s.Require().NoError(s.upgradeKeeper.DumpUpgradeInfoToDisk(101, expected))

	ui, err := s.upgradeKeeper.ReadUpgradeInfoFromDisk()
	s.Require().NoError(err)
	expected.Height = 101
	s.Require().Equal(expected, ui)

	// create invalid upgrade plan (with empty name)
	expected.Name = ""
	s.Require().NoError(s.upgradeKeeper.DumpUpgradeInfoToDisk(101, expected))
	_, err = s.upgradeKeeper.ReadUpgradeInfoFromDisk()
	s.Require().ErrorContains(err, "name cannot be empty: invalid request")
}

func (s *KeeperTestSuite) TestScheduleUpgrade() {
	cases := []struct {
		name    string
		plan    types.Plan
		setup   func()
		expPass bool
	}{
		{
			name: "successful height schedule",
			plan: types.Plan{
				Name:   "all-good",
				Info:   "some text here",
				Height: 123450000,
			},
			setup:   func() {},
			expPass: true,
		},
		{
			name: "successful overwrite",
			plan: types.Plan{
				Name:   "all-good",
				Info:   "some text here",
				Height: 123450000,
			},
			setup: func() {
				err := s.upgradeKeeper.ScheduleUpgrade(s.ctx, types.Plan{
					Name:   "alt-good",
					Info:   "new text here",
					Height: 543210000,
				})
				s.Require().NoError(err)
			},
			expPass: true,
		},
		{
			name: "unsuccessful schedule: invalid plan",
			plan: types.Plan{
				Height: 123450000,
			},
			setup:   func() {},
			expPass: false,
		},
		{
			name: "unsuccessful height schedule: due date in past",
			plan: types.Plan{
				Name:   "all-good",
				Info:   "some text here",
				Height: 1,
			},
			setup:   func() {},
			expPass: false,
		},
		{
			name: "unsuccessful schedule: schedule already executed",
			plan: types.Plan{
				Name:   "all-good",
				Info:   "some text here",
				Height: 123450000,
			},
			setup: func() {
				s.upgradeKeeper.SetUpgradeHandler("all-good", func(ctx context.Context, plan types.Plan, vm appmodule.VersionMap) (appmodule.VersionMap, error) {
					return vm, nil
				})
				err := s.upgradeKeeper.ApplyUpgrade(s.ctx, types.Plan{
					Name:   "all-good",
					Info:   "some text here",
					Height: 123450000,
				})
				s.Require().NoError(err)
			},
			expPass: false,
		},
	}

	for _, tc := range cases {
		s.Run(tc.name, func() {
			// reset suite
			s.SetupTest()

			// setup test case
			tc.setup()

			err := s.upgradeKeeper.ScheduleUpgrade(s.ctx, tc.plan)

			if tc.expPass {
				s.Require().NoError(err, "valid test case failed")
			} else {
				s.Require().Error(err, "invalid test case passed")
			}
		})
	}
}

func (s *KeeperTestSuite) TestSetUpgradedClient() {
	cs := []byte("IBC client state")

	cases := []struct {
		name   string
		height int64
		setup  func()
		exists bool
	}{
		{
			name:   "no upgraded client exists",
			height: 10,
			setup:  func() {},
			exists: false,
		},
		{
			name:   "success",
			height: 10,
			setup: func() {
				err := s.upgradeKeeper.SetUpgradedClient(s.ctx, 10, cs)
				s.Require().NoError(err)
			},
			exists: true,
		},
	}

	for _, tc := range cases {
		// reset suite
		s.SetupTest()

		// setup test case
		tc.setup()

		gotCs, err := s.upgradeKeeper.GetUpgradedClient(s.ctx, tc.height)

		if tc.exists {
			s.Require().Equal(cs, gotCs, "valid case: %s did not retrieve correct client state", tc.name)
			s.Require().NoError(err, "valid case: %s did not retrieve client state", tc.name)
		} else {
			s.Require().Nil(gotCs, "invalid case: %s retrieved valid client state", tc.name)
			s.Require().Error(err, "invalid case: %s retrieved valid client state", tc.name)
		}
	}
}

func (s *KeeperTestSuite) TestIsSkipHeight() {
	var skipOne int64 = 9
	ok := s.upgradeKeeper.IsSkipHeight(11)
	s.Require().False(ok)
	skip := map[int64]bool{skipOne: true}
	storeService := runtime.NewKVStoreService(s.key)
	env := runtime.NewEnvironment(storeService, coretesting.NewNopLogger())
	ctrl := gomock.NewController(s.T())
	upgradeKeeper := keeper.NewKeeper(env, skip, s.encCfg.Codec, s.T().TempDir(), s.baseApp, s.encodedAuthority, upgradetestutil.NewMockConsensusKeeper(ctrl))
	s.Require().True(upgradeKeeper.IsSkipHeight(9))
	s.Require().False(upgradeKeeper.IsSkipHeight(10))
}

func (s *KeeperTestSuite) TestUpgradedConsensusState() {
	cs := []byte("IBC consensus state")
	s.Require().NoError(s.upgradeKeeper.SetUpgradedConsensusState(s.ctx, 10, cs))
	bz, err := s.upgradeKeeper.GetUpgradedConsensusState(s.ctx, 10)
	s.Require().Equal(cs, bz)
	s.Require().NoError(err)
}

func (s *KeeperTestSuite) TestDowngradeVerified() {
	s.upgradeKeeper.SetDowngradeVerified(true)
	ok := s.upgradeKeeper.DowngradeVerified()
	s.Require().True(ok)
}

// Test that the protocol version successfully increments after an
// upgrade and is successfully set on BaseApp's appVersion.
func (s *KeeperTestSuite) TestIncrementProtocolVersion() {
	oldProtocolVersion, err := s.baseApp.AppVersion(context.Background())
	s.Require().NoError(err)
	res := s.upgradeKeeper.HasHandler("dummy")
	s.Require().False(res)
	dummyPlan := types.Plan{
		Name:   "dummy",
		Info:   "some text here",
		Height: 100,
	}

	err = s.upgradeKeeper.ApplyUpgrade(s.ctx, dummyPlan)
	s.Require().EqualError(err, "ApplyUpgrade should never be called without first checking HasHandler")

	s.upgradeKeeper.SetUpgradeHandler("dummy", func(_ context.Context, _ types.Plan, vm appmodule.VersionMap) (appmodule.VersionMap, error) {
		return vm, nil
	})
	err = s.upgradeKeeper.ApplyUpgrade(s.ctx, dummyPlan)
	s.Require().NoError(err)
	upgradedProtocolVersion, err := s.baseApp.AppVersion(s.ctx)
	s.Require().NoError(err)

	s.Require().Equal(oldProtocolVersion+1, upgradedProtocolVersion)
}

// Tests that the underlying state of x/upgrade is set correctly after
// an upgrade.
func (s *KeeperTestSuite) TestMigrations() {
	initialVM := appmodule.VersionMap{"bank": uint64(1)}
	err := s.upgradeKeeper.SetModuleVersionMap(s.ctx, initialVM)
	s.Require().NoError(err)
	vmBefore, err := s.upgradeKeeper.GetModuleVersionMap(s.ctx)
	s.Require().NoError(err)

	s.upgradeKeeper.SetUpgradeHandler("dummy", func(_ context.Context, _ types.Plan, vm appmodule.VersionMap) (appmodule.VersionMap, error) {
		// simulate upgrading the bank module
		vm["bank"]++
		return vm, nil
	})
	dummyPlan := types.Plan{
		Name:   "dummy",
		Info:   "some text here",
		Height: 123450000,
	}

	s.Require().NoError(s.upgradeKeeper.ApplyUpgrade(s.ctx, dummyPlan))
	vm, err := s.upgradeKeeper.GetModuleVersionMap(s.ctx)
	s.Require().Equal(vmBefore["bank"]+1, vm["bank"])
	s.Require().NoError(err)
}

func (s *KeeperTestSuite) TestLastCompletedUpgrade() {
	keeper := s.upgradeKeeper
	require := s.Require()

	s.T().Log("verify empty name if applied upgrades are empty")
	name, height, err := keeper.GetLastCompletedUpgrade(s.ctx)
	require.Equal("", name)
	require.Equal(int64(0), height)
	require.NoError(err)

	keeper.SetUpgradeHandler("test0", func(_ context.Context, _ types.Plan, vm appmodule.VersionMap) (appmodule.VersionMap, error) {
		return vm, nil
	})

	require.True(keeper.HasHandler("test0"))
	err = keeper.ApplyUpgrade(s.ctx, types.Plan{
		Name:   "test0",
		Height: 10,
	})
	require.NoError(err)

	s.T().Log("verify valid upgrade name and height")
	name, height, err = keeper.GetLastCompletedUpgrade(s.ctx)
	require.Equal("test0", name)
	require.Equal(int64(10), height)
	require.NoError(err)

	keeper.SetUpgradeHandler("test1", func(_ context.Context, _ types.Plan, vm appmodule.VersionMap) (appmodule.VersionMap, error) {
		return vm, nil
	})

	newCtx := s.ctx.WithHeaderInfo(header.Info{Height: 15})
	err = keeper.ApplyUpgrade(newCtx, types.Plan{
		Name:   "test1",
		Height: 15,
	})
	require.NoError(err)

	s.T().Log("verify valid upgrade name and height with multiple upgrades")
	name, height, err = keeper.GetLastCompletedUpgrade(newCtx)
	require.Equal("test1", name)
	require.Equal(int64(15), height)
	require.NoError(err)
}

// This test ensures that `GetLastDoneUpgrade` always returns the last upgrade according to the block height
// it was executed at, rather than using an ordering based on upgrade names.
func (s *KeeperTestSuite) TestLastCompletedUpgradeOrdering() {
	keeper := s.upgradeKeeper
	require := s.Require()

	// apply first upgrade
	keeper.SetUpgradeHandler("test-v0.9", func(_ context.Context, _ types.Plan, vm appmodule.VersionMap) (appmodule.VersionMap, error) {
		return vm, nil
	})

	err := keeper.ApplyUpgrade(s.ctx, types.Plan{
		Name:   "test-v0.9",
		Height: 10,
	})
	require.NoError(err)

	name, height, err := keeper.GetLastCompletedUpgrade(s.ctx)
	require.Equal("test-v0.9", name)
	require.Equal(int64(10), height)
	require.NoError(err)

	// apply second upgrade
	keeper.SetUpgradeHandler("test-v0.10", func(_ context.Context, _ types.Plan, vm appmodule.VersionMap) (appmodule.VersionMap, error) {
		return vm, nil
	})

	newCtx := s.ctx.WithHeaderInfo(header.Info{Height: 15})
	err = keeper.ApplyUpgrade(newCtx, types.Plan{
		Name:   "test-v0.10",
		Height: 15,
	})
	require.NoError(err)

	name, height, err = keeper.GetLastCompletedUpgrade(newCtx)
	require.Equal("test-v0.10", name)
	require.Equal(int64(15), height)
	require.NoError(err)
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

type paramStore struct {
	params cmtproto.ConsensusParams
}

var _ baseapp.ParamStore = (*paramStore)(nil)

func (ps *paramStore) Set(_ context.Context, value cmtproto.ConsensusParams) error {
	ps.params = value
	return nil
}

func (ps paramStore) Has(_ context.Context) (bool, error) {
	return true, nil
}

func (ps paramStore) Get(_ context.Context) (cmtproto.ConsensusParams, error) {
	return ps.params, nil
}
