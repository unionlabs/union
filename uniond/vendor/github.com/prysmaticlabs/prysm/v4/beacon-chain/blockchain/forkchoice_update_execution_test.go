package blockchain

import (
	"context"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	mockExecution "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestService_isNewHead(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	service := setupBeaconChain(t, beaconDB)
	require.Equal(t, true, service.isNewHead([32]byte{}))

	service.head = &head{root: [32]byte{1}}
	require.Equal(t, true, service.isNewHead([32]byte{2}))
	require.Equal(t, false, service.isNewHead([32]byte{1}))

	// Nil head should use origin root
	service.head = nil
	service.originBlockRoot = [32]byte{3}
	require.Equal(t, true, service.isNewHead([32]byte{2}))
	require.Equal(t, false, service.isNewHead([32]byte{3}))
}

func TestService_getHeadStateAndBlock(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	service := setupBeaconChain(t, beaconDB)
	_, _, err := service.getStateAndBlock(context.Background(), [32]byte{})
	require.ErrorContains(t, "block does not exist", err)

	blk, err := blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlock(&ethpb.SignedBeaconBlock{Signature: []byte{1}}))
	require.NoError(t, err)
	require.NoError(t, service.cfg.BeaconDB.SaveBlock(context.Background(), blk))

	st, _ := util.DeterministicGenesisState(t, 1)
	r, err := blk.Block().HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, service.cfg.BeaconDB.SaveState(context.Background(), st, r))

	gotState, err := service.cfg.BeaconDB.State(context.Background(), r)
	require.NoError(t, err)
	require.DeepEqual(t, st.ToProto(), gotState.ToProto())

	gotBlk, err := service.cfg.BeaconDB.Block(context.Background(), r)
	require.NoError(t, err)
	require.DeepEqual(t, blk, gotBlk)
}

func TestService_forkchoiceUpdateWithExecution_exceptionalCases(t *testing.T) {
	ctx := context.Background()
	opts := testServiceOptsWithDB(t)

	service, err := NewService(ctx, opts...)
	require.NoError(t, err)
	service.cfg.PayloadIDCache = cache.NewPayloadIDCache()
	service.cfg.TrackedValidatorsCache = cache.NewTrackedValidatorsCache()

	b := util.NewBeaconBlock()
	b.Block.Slot = 2
	wsb, err := blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	r1, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, service.saveInitSyncBlock(ctx, r1, wsb))
	st, _ := util.DeterministicGenesisState(t, 1)
	service.head = &head{
		root:  r1,
		block: wsb,
		state: st,
	}
	service.cfg.PayloadIDCache.Set(2, [32]byte{2}, [8]byte{1})
	b = util.NewBeaconBlock()
	b.Block.Slot = 3
	util.SaveBlock(t, ctx, service.cfg.BeaconDB, b)
	r1, err = b.Block.HashTreeRoot()
	require.NoError(t, err)
	st, _ = util.DeterministicGenesisState(t, 1)
	service.head = &head{
		root:  r1,
		block: wsb,
		state: st,
	}
	service.cfg.PayloadIDCache.Set(2, [32]byte{2}, [8]byte{1})
	args := &fcuConfig{
		headState:     st,
		headRoot:      r1,
		headBlock:     wsb,
		proposingSlot: service.CurrentSlot() + 1,
	}
	require.NoError(t, service.forkchoiceUpdateWithExecution(ctx, args))

	payloadID, has := service.cfg.PayloadIDCache.PayloadID(2, [32]byte{2})
	require.Equal(t, true, has)
	require.Equal(t, primitives.PayloadID{1}, payloadID)
}

func TestService_forkchoiceUpdateWithExecution_SameHeadRootNewProposer(t *testing.T) {
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx, beaconDB, fcs := tr.ctx, tr.db, tr.fcs

	altairBlk := util.SaveBlock(t, ctx, beaconDB, util.NewBeaconBlockAltair())
	altairBlkRoot, err := altairBlk.Block().HashTreeRoot()
	require.NoError(t, err)
	bellatrixBlk := util.SaveBlock(t, ctx, beaconDB, util.NewBeaconBlockBellatrix())
	bellatrixBlkRoot, err := bellatrixBlk.Block().HashTreeRoot()
	require.NoError(t, err)
	st, _ := util.DeterministicGenesisState(t, 10)
	service.head = &head{
		state: st,
	}

	ojc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	ofc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	state, blkRoot, err := prepareForkchoiceState(ctx, 0, [32]byte{}, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 1, altairBlkRoot, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 2, bellatrixBlkRoot, altairBlkRoot, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))

	service.cfg.ExecutionEngineCaller = &mockExecution.EngineClient{}
	require.NoError(t, beaconDB.SaveState(ctx, st, bellatrixBlkRoot))
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, bellatrixBlkRoot))
	sb, err := blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlockBellatrix(&ethpb.SignedBeaconBlockBellatrix{}))
	require.NoError(t, err)
	require.NoError(t, beaconDB.SaveBlock(ctx, sb))
	r, err := sb.Block().HashTreeRoot()
	require.NoError(t, err)

	// Set head to be the same but proposing next slot
	service.head.root = r
	service.head.block = sb
	service.head.state = st
	service.cfg.PayloadIDCache.Set(service.CurrentSlot()+1, [32]byte{} /* root */, [8]byte{})
	args := &fcuConfig{
		headState:     st,
		headBlock:     sb,
		headRoot:      r,
		proposingSlot: service.CurrentSlot() + 1,
	}
	require.NoError(t, service.forkchoiceUpdateWithExecution(ctx, args))
}

func TestShouldOverrideFCU(t *testing.T) {
	hook := logTest.NewGlobal()
	service, tr := minimalTestService(t)
	ctx, fcs := tr.ctx, tr.fcs

	service.SetGenesisTime(time.Now().Add(-time.Duration(2*params.BeaconConfig().SecondsPerSlot) * time.Second))
	headRoot := [32]byte{'b'}
	parentRoot := [32]byte{'a'}
	ojc := &ethpb.Checkpoint{}
	st, root, err := prepareForkchoiceState(ctx, 1, parentRoot, [32]byte{}, [32]byte{}, ojc, ojc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, st, root))
	st, root, err = prepareForkchoiceState(ctx, 2, headRoot, parentRoot, [32]byte{}, ojc, ojc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, st, root))

	require.Equal(t, primitives.Slot(2), service.CurrentSlot())
	require.Equal(t, true, service.shouldOverrideFCU(headRoot, 2))
	require.LogsDoNotContain(t, hook, "12 seconds")
	require.Equal(t, false, service.shouldOverrideFCU(parentRoot, 2))
	require.LogsContain(t, hook, "12 seconds")

	head, err := fcs.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, headRoot, head)

	fcs.SetGenesisTime(uint64(time.Now().Unix()) - 29)
	require.Equal(t, true, service.shouldOverrideFCU(parentRoot, 3))
	require.LogsDoNotContain(t, hook, "10 seconds")
	fcs.SetGenesisTime(uint64(time.Now().Unix()) - 24)
	service.SetGenesisTime(time.Now().Add(-time.Duration(2*params.BeaconConfig().SecondsPerSlot+10) * time.Second))
	require.Equal(t, false, service.shouldOverrideFCU(parentRoot, 3))
	require.LogsContain(t, hook, "10 seconds")
}
