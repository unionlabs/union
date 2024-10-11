package types_test

import (
	"context"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/comet"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/mock"
	"github.com/cosmos/cosmos-sdk/types"
)

type dummyCtxKey struct{}

type contextTestSuite struct {
	suite.Suite
}

func TestContextTestSuite(t *testing.T) {
	suite.Run(t, new(contextTestSuite))
}

func (s *contextTestSuite) TestCacheContext() {
	key := storetypes.NewKVStoreKey(s.T().Name() + "_TestCacheContext")
	k1 := []byte("hello")
	v1 := []byte("world")
	k2 := []byte("key")
	v2 := []byte("value")

	ctx := testutil.DefaultContext(key, storetypes.NewTransientStoreKey("transient_"+s.T().Name()))
	store := ctx.KVStore(key)
	store.Set(k1, v1)
	s.Require().Equal(v1, store.Get(k1))
	s.Require().Nil(store.Get(k2))

	cctx, write := ctx.CacheContext()
	cstore := cctx.KVStore(key)
	s.Require().Equal(v1, cstore.Get(k1))
	s.Require().Nil(cstore.Get(k2))

	// emit some events
	cctx.EventManager().EmitEvent(types.NewEvent("foo", types.NewAttribute("key", "value")))
	cctx.EventManager().EmitEvent(types.NewEvent("bar", types.NewAttribute("key", "value")))

	cstore.Set(k2, v2)
	s.Require().Equal(v2, cstore.Get(k2))
	s.Require().Nil(store.Get(k2))

	write()

	s.Require().Equal(v2, store.Get(k2))
	s.Require().Len(ctx.EventManager().Events(), 2)
}

func (s *contextTestSuite) TestLogContext() {
	key := storetypes.NewKVStoreKey(s.T().Name())
	ctx := testutil.DefaultContext(key, storetypes.NewTransientStoreKey("transient_"+s.T().Name()))
	ctrl := gomock.NewController(s.T())
	s.T().Cleanup(ctrl.Finish)

	logger := mock.NewMockLogger(ctrl)
	logger.EXPECT().Debug("debug")
	logger.EXPECT().Info("info")
	logger.EXPECT().Error("error")

	ctx = ctx.WithLogger(logger)
	ctx.Logger().Debug("debug")
	ctx.Logger().Info("info")
	ctx.Logger().Error("error")
}

// Testing saving/loading sdk type values to/from the context
func (s *contextTestSuite) TestContextWithCustom() {
	var ctx types.Context
	s.Require().True(ctx.IsZero())

	ctrl := gomock.NewController(s.T())
	s.T().Cleanup(ctrl.Finish)

	height := int64(1)
	chainid := "chainid"
	ischeck := true
	txbytes := []byte("txbytes")
	logger := mock.NewMockLogger(ctrl)
	voteinfos := []abci.VoteInfo{{}}
	meter := storetypes.NewGasMeter(10000)
	blockGasMeter := storetypes.NewGasMeter(20000)
	minGasPrices := types.DecCoins{types.NewInt64DecCoin("feetoken", 1)}
	headerHash := []byte("headerHash")
	zeroGasCfg := storetypes.GasConfig{}

	ctx = types.NewContext(nil, ischeck, logger)
	s.Require().Equal(cmtproto.Header{}, ctx.BlockHeader())

	ctx = ctx.
		WithBlockHeight(height).
		WithChainID(chainid).
		WithTxBytes(txbytes).
		WithVoteInfos(voteinfos).
		WithGasMeter(meter).
		WithMinGasPrices(minGasPrices).
		WithBlockGasMeter(blockGasMeter).
		WithHeaderHash(headerHash).
		WithKVGasConfig(zeroGasCfg).
		WithTransientKVGasConfig(zeroGasCfg)

	s.Require().Equal(height, ctx.BlockHeight())
	s.Require().Equal(chainid, ctx.ChainID())
	s.Require().Equal(ischeck, ctx.IsCheckTx())
	s.Require().Equal(txbytes, ctx.TxBytes())
	s.Require().Equal(logger, ctx.Logger())
	s.Require().Equal(voteinfos, ctx.VoteInfos())
	s.Require().Equal(meter, ctx.GasMeter())
	s.Require().Equal(minGasPrices, ctx.MinGasPrices())
	s.Require().Equal(blockGasMeter, ctx.BlockGasMeter())
	s.Require().Equal(headerHash, ctx.HeaderHash())
	s.Require().False(ctx.WithIsCheckTx(false).IsCheckTx())
	s.Require().Equal(zeroGasCfg, ctx.KVGasConfig())
	s.Require().Equal(zeroGasCfg, ctx.TransientKVGasConfig())

	// test IsReCheckTx
	s.Require().False(ctx.IsReCheckTx())
	ctx = ctx.WithIsCheckTx(false)
	ctx = ctx.WithIsReCheckTx(true)
	s.Require().True(ctx.IsCheckTx())
	s.Require().True(ctx.IsReCheckTx())

	// test consensus param
	s.Require().Equal(cmtproto.ConsensusParams{}, ctx.ConsensusParams())
	cp := cmtproto.ConsensusParams{}
	s.Require().Equal(cp, ctx.WithConsensusParams(cp).ConsensusParams())

	// test inner context
	newContext := context.WithValue(ctx.Context(), dummyCtxKey{}, "value")
	s.Require().NotEqual(ctx.Context(), ctx.WithContext(newContext).Context())
}

// Testing saving/loading of header fields to/from the context
func (s *contextTestSuite) TestContextHeader() {
	var ctx types.Context

	height := int64(5)
	addr := secp256k1.GenPrivKey().PubKey().Address()
	proposer := types.ConsAddress(addr)

	ctx = types.NewContext(nil, false, nil)

	ctx = ctx.
		WithBlockHeight(height).
		WithProposer(proposer)
	s.Require().Equal(height, ctx.BlockHeight())
	s.Require().Equal(height, ctx.BlockHeader().Height)
	s.Require().Equal(proposer.Bytes(), ctx.BlockHeader().ProposerAddress)
}

func (s *contextTestSuite) TestContextHeaderClone() {
	cases := map[string]struct {
		h cmtproto.Header
	}{
		"empty": {
			h: cmtproto.Header{},
		},
		"height": {
			h: cmtproto.Header{
				Height: 77,
			},
		},
		"time": {
			h: cmtproto.Header{
				Time: time.Unix(12345677, 12345),
			},
		},
		"zero time": {
			h: cmtproto.Header{
				Time: time.Unix(0, 0),
			},
		},
		"many items": {
			h: cmtproto.Header{
				Height:  823,
				Time:    time.Unix(9999999999, 0),
				ChainID: "silly-demo",
			},
		},
		"many items with hash": {
			h: cmtproto.Header{
				Height:        823,
				Time:          time.Unix(9999999999, 0),
				ChainID:       "silly-demo",
				AppHash:       []byte{5, 34, 11, 3, 23},
				ConsensusHash: []byte{11, 3, 23, 87, 3, 1},
			},
		},
	}

	for name, tc := range cases {
		s.T().Run(name, func(t *testing.T) {
			ctx := types.NewContext(nil, false, nil).WithBlockHeader(tc.h)
			s.Require().Equal(tc.h.Height, ctx.BlockHeight())
			s.Require().Equal(tc.h.Time.UTC(), ctx.BlockHeader().Time)

			// update only changes one field
			var newHeight int64 = 17
			ctx = ctx.WithBlockHeight(newHeight)
			s.Require().Equal(newHeight, ctx.BlockHeight())
			s.Require().Equal(tc.h.Time.UTC(), ctx.BlockHeader().Time)
		})
	}
}

func (s *contextTestSuite) TestUnwrapSDKContext() {
	sdkCtx := types.NewContext(nil, false, nil)
	ctx := types.WrapSDKContext(sdkCtx)
	sdkCtx2 := types.UnwrapSDKContext(ctx)
	s.Require().Equal(sdkCtx, sdkCtx2)

	ctx = context.Background()
	s.Require().Panics(func() { types.UnwrapSDKContext(ctx) })

	// test unwrapping when we've used context.WithValue
	ctx = context.WithValue(sdkCtx, dummyCtxKey{}, "bar")
	sdkCtx2 = types.UnwrapSDKContext(ctx)
	s.Require().Equal(sdkCtx, sdkCtx2)
}

func (s *contextTestSuite) TestTryUnwrapSDKContext() {
	sdkCtx := types.NewContext(nil, false, nil)
	ctx := types.WrapSDKContext(sdkCtx)
	unwrappedCtx, ok := types.TryUnwrapSDKContext(ctx)
	s.Require().True(ok)
	s.Require().Equal(sdkCtx, unwrappedCtx)

	// test case where context doesn't have sdk.Context
	ctxWithoutSDK := context.Background()
	unwrappedCtx, ok = types.TryUnwrapSDKContext(ctxWithoutSDK)
	s.Require().False(ok)
	s.Require().Equal(types.Context{}, unwrappedCtx)

	// test try unwrapping when we've used context.WithValue
	ctx = context.WithValue(sdkCtx, dummyCtxKey{}, "bar")
	unwrappedCtx, ok = types.TryUnwrapSDKContext(ctx)
	s.Require().True(ok)
	s.Require().Equal(sdkCtx, unwrappedCtx)
}

func (s *contextTestSuite) TestToSDKEvidence() {
	misbehaviors := []abci.Misbehavior{
		{
			Type:             abci.MisbehaviorType(1),
			Height:           100,
			Time:             time.Now(),
			TotalVotingPower: 10,
			Validator: abci.Validator{
				Address: []byte("address1"),
				Power:   5,
			},
		},
	}

	expEvidences := []comet.Evidence{
		{
			Type:             comet.MisbehaviorType(1),
			Height:           100,
			Time:             misbehaviors[0].Time,
			TotalVotingPower: 10,
			Validator: comet.Validator{
				Address: []byte("address1"),
				Power:   5,
			},
		},
	}

	// test ToSDKEvidence method
	evidence := types.ToSDKEvidence(misbehaviors)
	s.Require().Len(evidence, len(misbehaviors))
	s.Require().Equal(expEvidences, evidence)
}

func (s *contextTestSuite) TestToSDKCommitInfo() {
	commitInfo := abci.CommitInfo{
		Round: 1,
		Votes: []abci.VoteInfo{
			{
				Validator: abci.Validator{
					Address: []byte("address1"),
					Power:   5,
				},
				BlockIdFlag: cmtproto.BlockIDFlagCommit,
			},
		},
	}

	expCommit := comet.CommitInfo{
		Round: 1,
		Votes: []comet.VoteInfo{
			{
				Validator: comet.Validator{
					Address: []byte("address1"),
					Power:   5,
				},
				BlockIDFlag: comet.BlockIDFlagCommit,
			},
		},
	}

	// test ToSDKCommitInfo method
	commit := types.ToSDKCommitInfo(commitInfo)
	s.Require().Equal(expCommit, commit)
}

func (s *contextTestSuite) TestToSDKExtendedCommitInfo() {
	extendedCommitInfo := abci.ExtendedCommitInfo{
		Round: 1,
		Votes: []abci.ExtendedVoteInfo{
			{
				Validator: abci.Validator{
					Address: []byte("address1"),
					Power:   5,
				},
				BlockIdFlag: cmtproto.BlockIDFlagCommit,
			},
		},
	}

	expCommitInfo := comet.CommitInfo{
		Round: 1,
		Votes: []comet.VoteInfo{
			{
				Validator: comet.Validator{
					Address: []byte("address1"),
					Power:   5,
				},
				BlockIDFlag: comet.BlockIDFlagCommit,
			},
		},
	}

	// test ToSDKExtendedCommitInfo
	commitInfo := types.ToSDKExtendedCommitInfo(extendedCommitInfo)
	s.Require().Equal(expCommitInfo, commitInfo)
}
