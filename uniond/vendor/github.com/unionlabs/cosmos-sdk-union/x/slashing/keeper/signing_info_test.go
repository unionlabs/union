package keeper_test

import (
	"time"

	"github.com/golang/mock/gomock"

	"cosmossdk.io/x/slashing/testutil"
	slashingtypes "cosmossdk.io/x/slashing/types"

	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *KeeperTestSuite) TestValidatorSigningInfo() {
	ctx, keeper := s.ctx, s.slashingKeeper
	require := s.Require()

	consStr, err := s.stakingKeeper.ConsensusAddressCodec().BytesToString(consAddr)
	require.NoError(err)

	signingInfo := slashingtypes.NewValidatorSigningInfo(
		consStr,
		ctx.BlockHeight(),
		time.Unix(2, 0),
		false,
		int64(10),
	)

	// set the validator signing information
	require.NoError(keeper.ValidatorSigningInfo.Set(ctx, consAddr, signingInfo))
	require.True(keeper.HasValidatorSigningInfo(ctx, consAddr))
	info, err := keeper.ValidatorSigningInfo.Get(ctx, consAddr)
	require.NoError(err)
	require.Equal(info.StartHeight, ctx.BlockHeight())
	require.Equal(info.JailedUntil, time.Unix(2, 0).UTC())
	require.Equal(info.MissedBlocksCounter, int64(10))

	var signingInfos []slashingtypes.ValidatorSigningInfo

	err = keeper.ValidatorSigningInfo.Walk(ctx, nil, func(consAddr sdk.ConsAddress, info slashingtypes.ValidatorSigningInfo) (stop bool, err error) {
		signingInfos = append(signingInfos, info)
		return false, nil
	})
	require.NoError(err)
	require.Equal(signingInfos[0].Address, signingInfo.Address)

	// test Tombstone
	err = keeper.Tombstone(ctx, consAddr)
	require.NoError(err)
	require.True(keeper.IsTombstoned(ctx, consAddr))

	// test JailUntil
	jailTime := time.Now().Add(time.Hour).UTC()
	require.NoError(keeper.JailUntil(ctx, consAddr, jailTime))
	sInfo, _ := keeper.ValidatorSigningInfo.Get(ctx, consAddr)
	require.Equal(sInfo.JailedUntil, jailTime)
}

func (s *KeeperTestSuite) TestValidatorMissedBlockBitmap_SmallWindow() {
	ctx, keeper := s.ctx, s.slashingKeeper
	require := s.Require()

	for _, window := range []int64{100, 32_000} {
		params := testutil.TestParams()
		params.SignedBlocksWindow = window
		require.NoError(keeper.Params.Set(ctx, params))

		s.stakingKeeper.EXPECT().ValidatorIdentifier(gomock.Any(), consAddr).Return(consAddr, nil).AnyTimes()

		// validator misses all blocks in the window
		var valIdxOffset int64
		for valIdxOffset < params.SignedBlocksWindow {
			idx := valIdxOffset % params.SignedBlocksWindow
			err := keeper.SetMissedBlockBitmapValue(ctx, consAddr, idx, true)
			require.NoError(err)

			missed, err := keeper.GetMissedBlockBitmapValue(ctx, consAddr, idx)
			require.NoError(err)
			require.True(missed)

			valIdxOffset++
		}

		// validator should have missed all blocks
		missedBlocks, err := keeper.GetValidatorMissedBlocks(ctx, consAddr)
		require.NoError(err)
		require.Len(missedBlocks, int(params.SignedBlocksWindow))

		// sign next block, which rolls the missed block bitmap
		idx := valIdxOffset % params.SignedBlocksWindow
		err = keeper.SetMissedBlockBitmapValue(ctx, consAddr, idx, false)
		require.NoError(err)

		missed, err := keeper.GetMissedBlockBitmapValue(ctx, consAddr, idx)
		require.NoError(err)
		require.False(missed)

		// validator should have missed all blocks except the last one
		missedBlocks, err = keeper.GetValidatorMissedBlocks(ctx, consAddr)
		require.NoError(err)
		require.Len(missedBlocks, int(params.SignedBlocksWindow)-1)

		// if the validator rotated its key, there will be different consKeys and a mapping will be added in the state
		consAddr1 := sdk.ConsAddress("addr1_______________")
		s.stakingKeeper.EXPECT().ValidatorIdentifier(gomock.Any(), consAddr1).Return(consAddr, nil).AnyTimes()

		missedBlocks, err = keeper.GetValidatorMissedBlocks(ctx, consAddr1)
		require.NoError(err)
		require.Len(missedBlocks, int(params.SignedBlocksWindow)-1)
	}
}

func (s *KeeperTestSuite) TestPerformConsensusPubKeyUpdate() {
	ctx, slashingKeeper := s.ctx, s.slashingKeeper

	require := s.Require()

	pks := simtestutil.CreateTestPubKeys(500)

	oldConsAddr := sdk.ConsAddress(pks[0].Address())
	newConsAddr := sdk.ConsAddress(pks[1].Address())

	consStrAddr, err := s.stakingKeeper.ConsensusAddressCodec().BytesToString(newConsAddr)
	s.Require().NoError(err)

	newInfo := slashingtypes.NewValidatorSigningInfo(
		consStrAddr,
		int64(4),
		time.Unix(2, 0).UTC(),
		false,
		int64(10),
	)

	err = slashingKeeper.ValidatorSigningInfo.Set(ctx, oldConsAddr, newInfo)
	require.NoError(err)

	s.stakingKeeper.EXPECT().ValidatorIdentifier(gomock.Any(), oldConsAddr).Return(oldConsAddr, nil)
	err = slashingKeeper.SetMissedBlockBitmapValue(ctx, oldConsAddr, 10, true)
	require.NoError(err)

	err = slashingKeeper.Hooks().AfterConsensusPubKeyUpdate(ctx, pks[0], pks[1], sdk.Coin{})
	require.NoError(err)

	// check pubkey relation is set properly
	savedPubKey, err := slashingKeeper.GetPubkey(ctx, newConsAddr.Bytes())
	require.NoError(err)
	require.Equal(savedPubKey, pks[1])

	// check validator's SigningInfo is set properly with new consensus pubkey
	signingInfo, err := slashingKeeper.ValidatorSigningInfo.Get(ctx, newConsAddr)
	require.NoError(err)
	require.Equal(signingInfo, newInfo)

	// missed blocks map corresponds only to the old cons key, as there is an identifier added to get the missed blocks using the new cons key
	missedBlocks, err := slashingKeeper.GetValidatorMissedBlocks(ctx, oldConsAddr)
	require.NoError(err)

	require.Len(missedBlocks, 1)
}
