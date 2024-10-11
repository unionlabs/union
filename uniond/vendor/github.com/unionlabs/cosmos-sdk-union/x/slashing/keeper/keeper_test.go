package keeper_test

import (
	"encoding/binary"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	st "cosmossdk.io/api/cosmos/staking/v1beta1"
	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	sdkmath "cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	slashingkeeper "cosmossdk.io/x/slashing/keeper"
	slashingtestutil "cosmossdk.io/x/slashing/testutil"
	slashingtypes "cosmossdk.io/x/slashing/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdktestutil "github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	addresstypes "github.com/cosmos/cosmos-sdk/types/address"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var consAddr = sdk.ConsAddress("addr1_______________")

type KeeperTestSuite struct {
	suite.Suite

	ctx            sdk.Context
	stakingKeeper  *slashingtestutil.MockStakingKeeper
	slashingKeeper slashingkeeper.Keeper
	queryClient    slashingtypes.QueryClient
	msgServer      slashingtypes.MsgServer
	key            *storetypes.KVStoreKey
}

func (s *KeeperTestSuite) SetupTest() {
	key := storetypes.NewKVStoreKey(slashingtypes.StoreKey)
	s.key = key
	storeService := runtime.NewKVStoreService(key)
	env := runtime.NewEnvironment(storeService, coretesting.NewNopLogger())
	testCtx := sdktestutil.DefaultContextWithDB(s.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now().Round(0).UTC()})
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})

	// gomock initializations
	ctrl := gomock.NewController(s.T())
	s.stakingKeeper = slashingtestutil.NewMockStakingKeeper(ctrl)
	s.stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec("cosmosvaloper")).AnyTimes()
	s.stakingKeeper.EXPECT().ConsensusAddressCodec().Return(address.NewBech32Codec("cosmosvalcons")).AnyTimes()

	authStr, err := address.NewBech32Codec("cosmos").BytesToString(authtypes.NewModuleAddress(slashingtypes.GovModuleName))
	s.Require().NoError(err)

	s.ctx = ctx
	s.slashingKeeper = slashingkeeper.NewKeeper(
		env,
		encCfg.Codec,
		encCfg.Amino,
		s.stakingKeeper,
		authStr,
	)
	// set test params
	err = s.slashingKeeper.Params.Set(ctx, slashingtestutil.TestParams())
	s.Require().NoError(err)
	slashingtypes.RegisterInterfaces(encCfg.InterfaceRegistry)
	queryHelper := baseapp.NewQueryServerTestHelper(ctx, encCfg.InterfaceRegistry)
	slashingtypes.RegisterQueryServer(queryHelper, slashingkeeper.NewQuerier(s.slashingKeeper))

	s.queryClient = slashingtypes.NewQueryClient(queryHelper)
	s.msgServer = slashingkeeper.NewMsgServerImpl(s.slashingKeeper)
}

func (s *KeeperTestSuite) TestPubkey() {
	ctx, keeper := s.ctx, s.slashingKeeper
	require := s.Require()

	_, pubKey, addr := testdata.KeyTestPubAddr()
	require.NoError(keeper.AddrPubkeyRelation.Set(ctx, pubKey.Address(), pubKey))

	expectedPubKey, err := keeper.GetPubkey(ctx, addr.Bytes())
	require.NoError(err)
	require.Equal(pubKey, expectedPubKey)
}

func (s *KeeperTestSuite) TestJailAndSlash() {
	slashFractionDoubleSign, err := s.slashingKeeper.SlashFractionDoubleSign(s.ctx)
	s.Require().NoError(err)

	s.stakingKeeper.EXPECT().SlashWithInfractionReason(s.ctx,
		consAddr,
		s.ctx.BlockHeight(),
		sdk.TokensToConsensusPower(sdkmath.NewInt(1), sdk.DefaultPowerReduction),
		slashFractionDoubleSign,
		st.Infraction_INFRACTION_UNSPECIFIED,
	).Return(sdkmath.NewInt(0), nil)

	err = s.slashingKeeper.Slash(
		s.ctx,
		consAddr,
		slashFractionDoubleSign,
		sdk.TokensToConsensusPower(sdkmath.NewInt(1), sdk.DefaultPowerReduction),
		s.ctx.BlockHeight(),
	)
	s.Require().NoError(err)
	s.stakingKeeper.EXPECT().Jail(s.ctx, consAddr).Return(nil)
	s.Require().NoError(s.slashingKeeper.Jail(s.ctx, consAddr))
}

func (s *KeeperTestSuite) TestJailAndSlashWithInfractionReason() {
	slashFractionDoubleSign, err := s.slashingKeeper.SlashFractionDoubleSign(s.ctx)
	s.Require().NoError(err)

	s.stakingKeeper.EXPECT().SlashWithInfractionReason(s.ctx,
		consAddr,
		s.ctx.BlockHeight(),
		sdk.TokensToConsensusPower(sdkmath.NewInt(1), sdk.DefaultPowerReduction),
		slashFractionDoubleSign,
		st.Infraction_INFRACTION_DOUBLE_SIGN,
	).Return(sdkmath.NewInt(0), nil)

	err = s.slashingKeeper.SlashWithInfractionReason(
		s.ctx,
		consAddr,
		slashFractionDoubleSign,
		sdk.TokensToConsensusPower(sdkmath.NewInt(1), sdk.DefaultPowerReduction),
		s.ctx.BlockHeight(),
		st.Infraction_INFRACTION_DOUBLE_SIGN,
	)
	s.Require().NoError(err)
	s.stakingKeeper.EXPECT().Jail(s.ctx, consAddr).Return(nil)
	s.Require().NoError(s.slashingKeeper.Jail(s.ctx, consAddr))
}

// ValidatorMissedBlockBitmapKey returns the key for a validator's missed block
// bitmap chunk.
func validatorMissedBlockBitmapKey(v sdk.ConsAddress, chunkIndex int64) []byte {
	bz := make([]byte, 8)
	binary.LittleEndian.PutUint64(bz, uint64(chunkIndex))

	validatorMissedBlockBitmapKeyPrefix := []byte{0x02} // Prefix for missed block bitmap
	return append(append(validatorMissedBlockBitmapKeyPrefix, addresstypes.MustLengthPrefix(v.Bytes())...), bz...)
}

func (s *KeeperTestSuite) TestValidatorMissedBlockBMMigrationToColls() {
	s.SetupTest()

	consAddr := sdk.ConsAddress("addr1_______________")
	index := int64(0)
	err := sdktestutil.DiffCollectionsMigration(
		s.ctx,
		s.key,
		100,
		func(i int64) {
			s.ctx.KVStore(s.key).Set(validatorMissedBlockBitmapKey(consAddr, index), []byte{})
		},
		"7ad1f994d45ec9495ae5f990a3fba100c2cc70167a154c33fb43882dc004eafd",
	)
	s.Require().NoError(err)

	err = sdktestutil.DiffCollectionsMigration(
		s.ctx,
		s.key,
		100,
		func(i int64) {
			err := s.slashingKeeper.SetMissedBlockBitmapChunk(s.ctx, consAddr, index, []byte{})
			s.Require().NoError(err)
		},
		"7ad1f994d45ec9495ae5f990a3fba100c2cc70167a154c33fb43882dc004eafd",
	)
	s.Require().NoError(err)
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}
