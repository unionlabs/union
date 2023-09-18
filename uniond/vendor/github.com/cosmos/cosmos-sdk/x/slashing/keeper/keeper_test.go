package keeper_test

import (
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtime "github.com/cometbft/cometbft/types/time"

	"github.com/cosmos/cosmos-sdk/baseapp"
	sdktestutil "github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	slashingkeeper "github.com/cosmos/cosmos-sdk/x/slashing/keeper"
	slashingtestutil "github.com/cosmos/cosmos-sdk/x/slashing/testutil"
	slashingtypes "github.com/cosmos/cosmos-sdk/x/slashing/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

var consAddr = sdk.ConsAddress(sdk.AccAddress([]byte("addr1_______________")))

type KeeperTestSuite struct {
	suite.Suite

	ctx            sdk.Context
	stakingKeeper  *slashingtestutil.MockStakingKeeper
	slashingKeeper slashingkeeper.Keeper
	queryClient    slashingtypes.QueryClient
	msgServer      slashingtypes.MsgServer
}

func (s *KeeperTestSuite) SetupTest() {
	key := sdk.NewKVStoreKey(slashingtypes.StoreKey)
	testCtx := sdktestutil.DefaultContextWithDB(s.T(), key, sdk.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Time: tmtime.Now()})
	encCfg := moduletestutil.MakeTestEncodingConfig()

	// gomock initializations
	ctrl := gomock.NewController(s.T())
	s.stakingKeeper = slashingtestutil.NewMockStakingKeeper(ctrl)

	s.ctx = ctx
	s.slashingKeeper = slashingkeeper.NewKeeper(
		encCfg.Codec,
		encCfg.Amino,
		key,
		s.stakingKeeper,
		authtypes.NewModuleAddress(govtypes.ModuleName).String(),
	)
	// set test params
	s.slashingKeeper.SetParams(ctx, slashingtestutil.TestParams())

	slashingtypes.RegisterInterfaces(encCfg.InterfaceRegistry)
	queryHelper := baseapp.NewQueryServerTestHelper(ctx, encCfg.InterfaceRegistry)
	slashingtypes.RegisterQueryServer(queryHelper, s.slashingKeeper)

	s.queryClient = slashingtypes.NewQueryClient(queryHelper)
	s.msgServer = slashingkeeper.NewMsgServerImpl(s.slashingKeeper)
}

func (s *KeeperTestSuite) TestPubkey() {
	ctx, keeper := s.ctx, s.slashingKeeper
	require := s.Require()

	_, pubKey, addr := testdata.KeyTestPubAddr()
	require.NoError(keeper.AddPubkey(ctx, pubKey))

	expectedPubKey, err := keeper.GetPubkey(ctx, addr.Bytes())
	require.NoError(err)
	require.Equal(pubKey, expectedPubKey)
}

func (s *KeeperTestSuite) TestJailAndSlash() {
	s.stakingKeeper.EXPECT().SlashWithInfractionReason(s.ctx,
		consAddr,
		s.ctx.BlockHeight(),
		sdk.TokensToConsensusPower(sdk.NewInt(1), sdk.DefaultPowerReduction),
		s.slashingKeeper.SlashFractionDoubleSign(s.ctx),
		stakingtypes.Infraction_INFRACTION_UNSPECIFIED,
	).Return(sdk.NewInt(0))

	s.slashingKeeper.Slash(
		s.ctx,
		consAddr,
		s.slashingKeeper.SlashFractionDoubleSign(s.ctx),
		sdk.TokensToConsensusPower(sdk.NewInt(1), sdk.DefaultPowerReduction),
		s.ctx.BlockHeight(),
	)

	s.stakingKeeper.EXPECT().Jail(s.ctx, consAddr).Return()
	s.slashingKeeper.Jail(s.ctx, consAddr)
}

func (s *KeeperTestSuite) TestJailAndSlashWithInfractionReason() {
	s.stakingKeeper.EXPECT().SlashWithInfractionReason(s.ctx,
		consAddr,
		s.ctx.BlockHeight(),
		sdk.TokensToConsensusPower(sdk.NewInt(1), sdk.DefaultPowerReduction),
		s.slashingKeeper.SlashFractionDoubleSign(s.ctx),
		stakingtypes.Infraction_INFRACTION_DOUBLE_SIGN,
	).Return(sdk.NewInt(0))

	s.slashingKeeper.SlashWithInfractionReason(
		s.ctx,
		consAddr,
		s.slashingKeeper.SlashFractionDoubleSign(s.ctx),
		sdk.TokensToConsensusPower(sdk.NewInt(1), sdk.DefaultPowerReduction),
		s.ctx.BlockHeight(),
		stakingtypes.Infraction_INFRACTION_DOUBLE_SIGN,
	)

	s.stakingKeeper.EXPECT().Jail(s.ctx, consAddr).Return()
	s.slashingKeeper.Jail(s.ctx, consAddr)
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}
