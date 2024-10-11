package blockchain

import (
	"bytes"
	"context"
	"reflect"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache/depositcache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	mockExecution "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/voluntaryexits"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func setupBeaconChain(t *testing.T, beaconDB db.Database) *Service {
	ctx := context.Background()
	var web3Service *execution.Service
	var err error
	srv, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		srv.Stop()
	})
	bState, _ := util.DeterministicGenesisState(t, 10)
	pbState, err := state_native.ProtobufBeaconStatePhase0(bState.ToProtoUnsafe())
	require.NoError(t, err)
	mockTrie, err := trie.NewTrie(0)
	require.NoError(t, err)
	err = beaconDB.SaveExecutionChainData(ctx, &ethpb.ETH1ChainData{
		BeaconState: pbState,
		Trie:        mockTrie.ToProto(),
		CurrentEth1Data: &ethpb.LatestETH1Data{
			BlockHash: make([]byte, 32),
		},
		ChainstartData: &ethpb.ChainStartData{
			Eth1Data: &ethpb.Eth1Data{
				DepositRoot:  make([]byte, 32),
				DepositCount: 0,
				BlockHash:    make([]byte, 32),
			},
		},
		DepositContainers: []*ethpb.DepositContainer{},
	})
	require.NoError(t, err)
	web3Service, err = execution.NewService(
		ctx,
		execution.WithDatabase(beaconDB),
		execution.WithHttpEndpoint(endpoint),
		execution.WithDepositContractAddress(common.Address{}),
	)
	require.NoError(t, err, "Unable to set up web3 service")

	attService, err := attestations.NewService(ctx, &attestations.Config{Pool: attestations.NewPool()})
	require.NoError(t, err)

	depositCache, err := depositcache.New()
	require.NoError(t, err)

	fc := doublylinkedtree.New()
	stateGen := stategen.New(beaconDB, fc)
	// Safe a state in stategen to purposes of testing a service stop / shutdown.
	require.NoError(t, stateGen.SaveState(ctx, bytesutil.ToBytes32(bState.FinalizedCheckpoint().Root), bState))

	opts := []Option{
		WithDatabase(beaconDB),
		WithDepositCache(depositCache),
		WithChainStartFetcher(web3Service),
		WithAttestationPool(attestations.NewPool()),
		WithSlashingPool(slashings.NewPool()),
		WithExitPool(voluntaryexits.NewPool()),
		WithP2PBroadcaster(&mockBroadcaster{}),
		WithStateNotifier(&mockBeaconNode{}),
		WithForkChoiceStore(fc),
		WithAttestationService(attService),
		WithStateGen(stateGen),
		WithPayloadIDCache(cache.NewPayloadIDCache()),
		WithClockSynchronizer(startup.NewClockSynchronizer()),
	}

	chainService, err := NewService(ctx, opts...)
	require.NoError(t, err, "Unable to setup chain service")
	chainService.genesisTime = time.Unix(1, 0) // non-zero time

	return chainService
}

func TestChainStartStop_Initialized(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	chainService := setupBeaconChain(t, beaconDB)

	gt := time.Unix(23, 0)
	genesisBlk := util.NewBeaconBlock()
	blkRoot, err := genesisBlk.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, beaconDB, genesisBlk)
	s, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, s.SetGenesisTime(uint64(gt.Unix())))
	require.NoError(t, s.SetSlot(1))
	require.NoError(t, beaconDB.SaveState(ctx, s, blkRoot))
	require.NoError(t, beaconDB.SaveHeadBlockRoot(ctx, blkRoot))
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, blkRoot))
	require.NoError(t, beaconDB.SaveJustifiedCheckpoint(ctx, &ethpb.Checkpoint{Root: blkRoot[:]}))
	require.NoError(t, beaconDB.SaveFinalizedCheckpoint(ctx, &ethpb.Checkpoint{Root: blkRoot[:]}))
	ss := &ethpb.StateSummary{
		Slot: 1,
		Root: blkRoot[:],
	}
	require.NoError(t, beaconDB.SaveStateSummary(ctx, ss))
	chainService.cfg.FinalizedStateAtStartUp = s
	// Test the start function.
	chainService.Start()

	require.NoError(t, chainService.Stop(), "Unable to stop chain service")

	// The context should have been canceled.
	assert.Equal(t, context.Canceled, chainService.ctx.Err(), "Context was not canceled")
	require.LogsContain(t, hook, "data already exists")
}

func TestChainStartStop_GenesisZeroHashes(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	chainService := setupBeaconChain(t, beaconDB)

	gt := time.Unix(23, 0)
	genesisBlk := util.NewBeaconBlock()
	blkRoot, err := genesisBlk.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb := util.SaveBlock(t, ctx, beaconDB, genesisBlk)
	s, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, s.SetGenesisTime(uint64(gt.Unix())))
	require.NoError(t, beaconDB.SaveState(ctx, s, blkRoot))
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, blkRoot))
	require.NoError(t, beaconDB.SaveBlock(ctx, wsb))
	require.NoError(t, beaconDB.SaveJustifiedCheckpoint(ctx, &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}))
	require.NoError(t, beaconDB.SaveFinalizedCheckpoint(ctx, &ethpb.Checkpoint{Root: blkRoot[:]}))
	chainService.cfg.FinalizedStateAtStartUp = s
	// Test the start function.
	chainService.Start()

	require.NoError(t, chainService.Stop(), "Unable to stop chain service")

	// The context should have been canceled.
	assert.Equal(t, context.Canceled, chainService.ctx.Err(), "Context was not canceled")
	require.LogsContain(t, hook, "data already exists")
}

func TestChainService_InitializeBeaconChain(t *testing.T) {
	helpers.ClearCache()
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()

	bc := setupBeaconChain(t, beaconDB)
	var err error

	// Set up 10 deposits pre chain start for validators to register
	count := uint64(10)
	deposits, _, err := util.DeterministicDepositsAndKeys(count)
	require.NoError(t, err)
	dt, _, err := util.DepositTrieFromDeposits(deposits)
	require.NoError(t, err)
	hashTreeRoot, err := dt.HashTreeRoot()
	require.NoError(t, err)
	genState, err := transition.EmptyGenesisState()
	require.NoError(t, err)
	err = genState.SetEth1Data(&ethpb.Eth1Data{
		DepositRoot:  hashTreeRoot[:],
		DepositCount: uint64(len(deposits)),
		BlockHash:    make([]byte, 32),
	})
	require.NoError(t, err)
	genState, err = blocks.ProcessPreGenesisDeposits(ctx, genState, deposits)
	require.NoError(t, err)

	_, err = bc.initializeBeaconChain(ctx, time.Unix(0, 0), genState, &ethpb.Eth1Data{DepositRoot: hashTreeRoot[:], BlockHash: make([]byte, 32)})
	require.NoError(t, err)

	_, err = bc.HeadState(ctx)
	assert.NoError(t, err)
	headBlk, err := bc.HeadBlock(ctx)
	require.NoError(t, err)
	if headBlk == nil {
		t.Error("Head state can't be nil after initialize beacon chain")
	}
	r, err := bc.HeadRoot(ctx)
	require.NoError(t, err)
	if bytesutil.ToBytes32(r) == params.BeaconConfig().ZeroHash {
		t.Error("Canonical root for slot 0 can't be zeros after initialize beacon chain")
	}
}

func TestChainService_CorrectGenesisRoots(t *testing.T) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	chainService := setupBeaconChain(t, beaconDB)

	gt := time.Unix(23, 0)
	genesisBlk := util.NewBeaconBlock()
	blkRoot, err := genesisBlk.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, beaconDB, genesisBlk)
	s, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, s.SetGenesisTime(uint64(gt.Unix())))
	require.NoError(t, s.SetSlot(0))
	require.NoError(t, beaconDB.SaveState(ctx, s, blkRoot))
	require.NoError(t, beaconDB.SaveHeadBlockRoot(ctx, blkRoot))
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, blkRoot))
	require.NoError(t, beaconDB.SaveFinalizedCheckpoint(ctx, &ethpb.Checkpoint{Root: blkRoot[:]}))
	chainService.cfg.FinalizedStateAtStartUp = s
	// Test the start function.
	chainService.Start()

	cp := chainService.FinalizedCheckpt()
	require.DeepEqual(t, blkRoot[:], cp.Root, "Finalize Checkpoint root is incorrect")
	cp = chainService.CurrentJustifiedCheckpt()
	require.NoError(t, err)
	require.DeepEqual(t, params.BeaconConfig().ZeroHash[:], cp.Root, "Justified Checkpoint root is incorrect")

	require.NoError(t, chainService.Stop(), "Unable to stop chain service")

}

func TestChainService_InitializeChainInfo(t *testing.T) {
	genesis := util.NewBeaconBlock()
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err)

	finalizedSlot := params.BeaconConfig().SlotsPerEpoch*2 + 1
	headBlock := util.NewBeaconBlock()
	headBlock.Block.Slot = finalizedSlot
	headBlock.Block.ParentRoot = bytesutil.PadTo(genesisRoot[:], 32)
	headState, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, headState.SetSlot(finalizedSlot))
	require.NoError(t, headState.SetGenesisValidatorsRoot(params.BeaconConfig().ZeroHash[:]))
	headRoot, err := headBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	c, tr := minimalTestService(t, WithFinalizedStateAtStartUp(headState))
	ctx, beaconDB, stateGen := tr.ctx, tr.db, tr.sg

	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, genesisRoot))
	util.SaveBlock(t, ctx, beaconDB, genesis)
	require.NoError(t, beaconDB.SaveState(ctx, headState, headRoot))
	require.NoError(t, beaconDB.SaveState(ctx, headState, genesisRoot))
	util.SaveBlock(t, ctx, beaconDB, headBlock)
	require.NoError(t, beaconDB.SaveFinalizedCheckpoint(ctx, &ethpb.Checkpoint{Epoch: slots.ToEpoch(finalizedSlot), Root: headRoot[:]}))
	require.NoError(t, stateGen.SaveState(ctx, headRoot, headState))

	require.NoError(t, c.StartFromSavedState(headState))
	headBlk, err := c.HeadBlock(ctx)
	require.NoError(t, err)
	pb, err := headBlk.Proto()
	require.NoError(t, err)
	assert.DeepEqual(t, headBlock, pb, "Head block incorrect")
	s, err := c.HeadState(ctx)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, headState.ToProtoUnsafe(), s.ToProtoUnsafe(), "Head state incorrect")
	assert.Equal(t, c.HeadSlot(), headBlock.Block.Slot, "Head slot incorrect")
	r, err := c.HeadRoot(context.Background())
	require.NoError(t, err)
	if !bytes.Equal(headRoot[:], r) {
		t.Error("head slot incorrect")
	}
	assert.Equal(t, genesisRoot, c.originBlockRoot, "Genesis block root incorrect")
}

func TestChainService_InitializeChainInfo_SetHeadAtGenesis(t *testing.T) {
	genesis := util.NewBeaconBlock()
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err)

	finalizedSlot := params.BeaconConfig().SlotsPerEpoch*2 + 1
	headBlock := util.NewBeaconBlock()
	headBlock.Block.Slot = finalizedSlot
	headBlock.Block.ParentRoot = bytesutil.PadTo(genesisRoot[:], 32)
	headState, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, headState.SetSlot(finalizedSlot))
	require.NoError(t, headState.SetGenesisValidatorsRoot(params.BeaconConfig().ZeroHash[:]))
	headRoot, err := headBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	c, tr := minimalTestService(t, WithFinalizedStateAtStartUp(headState))
	ctx, beaconDB := tr.ctx, tr.db

	util.SaveBlock(t, ctx, beaconDB, genesis)
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, genesisRoot))
	require.NoError(t, beaconDB.SaveState(ctx, headState, genesisRoot))
	require.NoError(t, beaconDB.SaveState(ctx, headState, headRoot))
	util.SaveBlock(t, ctx, beaconDB, headBlock)
	ss := &ethpb.StateSummary{
		Slot: finalizedSlot,
		Root: headRoot[:],
	}
	require.NoError(t, beaconDB.SaveStateSummary(ctx, ss))
	require.NoError(t, beaconDB.SaveFinalizedCheckpoint(ctx, &ethpb.Checkpoint{Root: headRoot[:], Epoch: slots.ToEpoch(finalizedSlot)}))

	require.NoError(t, c.StartFromSavedState(headState))
	s, err := c.HeadState(ctx)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, headState.ToProtoUnsafe(), s.ToProtoUnsafe(), "Head state incorrect")
	assert.Equal(t, genesisRoot, c.originBlockRoot, "Genesis block root incorrect")
	pb, err := c.head.block.Proto()
	require.NoError(t, err)
	assert.DeepEqual(t, headBlock, pb)
}

func TestChainService_SaveHeadNoDB(t *testing.T) {
	beaconDB := testDB.SetupDB(t)
	ctx := context.Background()
	fc := doublylinkedtree.New()
	s := &Service{
		cfg: &config{BeaconDB: beaconDB, StateGen: stategen.New(beaconDB, fc), ForkChoiceStore: fc},
	}
	blk := util.NewBeaconBlock()
	blk.Block.Slot = 1
	r, err := blk.HashTreeRoot()
	require.NoError(t, err)
	newState, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, s.cfg.StateGen.SaveState(ctx, r, newState))
	wsb, err := consensusblocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	require.NoError(t, s.saveHeadNoDB(ctx, wsb, r, newState, false))

	newB, err := s.cfg.BeaconDB.HeadBlock(ctx)
	require.NoError(t, err)
	if reflect.DeepEqual(newB, blk) {
		t.Error("head block should not be equal")
	}
}

func TestHasBlock_ForkChoiceAndDB_DoublyLinkedTree(t *testing.T) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	s := &Service{
		cfg: &config{ForkChoiceStore: doublylinkedtree.New(), BeaconDB: beaconDB},
	}
	b := util.NewBeaconBlock()
	r, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	beaconState, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, s.cfg.ForkChoiceStore.InsertNode(ctx, beaconState, r))

	assert.Equal(t, false, s.hasBlock(ctx, [32]byte{}), "Should not have block")
	assert.Equal(t, true, s.hasBlock(ctx, r), "Should have block")
}

func TestServiceStop_SaveCachedBlocks(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	beaconDB := testDB.SetupDB(t)
	s := &Service{
		cfg:            &config{BeaconDB: beaconDB, StateGen: stategen.New(beaconDB, doublylinkedtree.New())},
		ctx:            ctx,
		cancel:         cancel,
		initSyncBlocks: make(map[[32]byte]interfaces.ReadOnlySignedBeaconBlock),
	}
	bb := util.NewBeaconBlock()
	r, err := bb.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err := consensusblocks.NewSignedBeaconBlock(bb)
	require.NoError(t, err)
	require.NoError(t, s.saveInitSyncBlock(ctx, r, wsb))
	require.NoError(t, s.Stop())
	require.Equal(t, true, s.cfg.BeaconDB.HasBlock(ctx, r))
}

func TestProcessChainStartTime_ReceivedFeed(t *testing.T) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := setupBeaconChain(t, beaconDB)
	mgs := &MockClockSetter{}
	service.clockSetter = mgs
	gt := time.Now()
	service.onExecutionChainStart(context.Background(), gt)
	gs, err := beaconDB.GenesisState(ctx)
	require.NoError(t, err)
	require.NotEqual(t, nil, gs)
	require.Equal(t, 32, len(gs.GenesisValidatorsRoot()))
	var zero [32]byte
	require.DeepNotEqual(t, gs.GenesisValidatorsRoot(), zero[:])
	require.Equal(t, gt, mgs.G.GenesisTime())
	require.Equal(t, bytesutil.ToBytes32(gs.GenesisValidatorsRoot()), mgs.G.GenesisValidatorsRoot())
}

func BenchmarkHasBlockDB(b *testing.B) {
	beaconDB := testDB.SetupDB(b)
	ctx := context.Background()
	s := &Service{
		cfg: &config{BeaconDB: beaconDB},
	}
	blk := util.NewBeaconBlock()
	wsb, err := consensusblocks.NewSignedBeaconBlock(blk)
	require.NoError(b, err)
	require.NoError(b, s.cfg.BeaconDB.SaveBlock(ctx, wsb))
	r, err := blk.Block.HashTreeRoot()
	require.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		require.Equal(b, true, s.cfg.BeaconDB.HasBlock(ctx, r), "Block is not in DB")
	}
}

func BenchmarkHasBlockForkChoiceStore_DoublyLinkedTree(b *testing.B) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(b)
	s := &Service{
		cfg: &config{ForkChoiceStore: doublylinkedtree.New(), BeaconDB: beaconDB},
	}
	blk := util.NewBeaconBlock()
	r, err := blk.Block.HashTreeRoot()
	require.NoError(b, err)
	beaconState, err := util.NewBeaconState()
	require.NoError(b, err)
	require.NoError(b, s.cfg.ForkChoiceStore.InsertNode(ctx, beaconState, r))

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		require.Equal(b, true, s.cfg.ForkChoiceStore.HasNode(r), "Block is not in fork choice store")
	}
}

func TestChainService_EverythingOptimistic(t *testing.T) {
	resetFn := features.InitWithReset(&features.Flags{
		EnableStartOptimistic: true,
	})
	defer resetFn()

	genesis := util.NewBeaconBlock()
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err)
	finalizedSlot := params.BeaconConfig().SlotsPerEpoch*2 + 1
	headBlock := util.NewBeaconBlock()
	headBlock.Block.Slot = finalizedSlot
	headBlock.Block.ParentRoot = bytesutil.PadTo(genesisRoot[:], 32)
	headState, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, headState.SetSlot(finalizedSlot))
	require.NoError(t, headState.SetGenesisValidatorsRoot(params.BeaconConfig().ZeroHash[:]))
	headRoot, err := headBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	c, tr := minimalTestService(t, WithFinalizedStateAtStartUp(headState))
	ctx, beaconDB, stateGen := tr.ctx, tr.db, tr.sg

	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, genesisRoot))
	util.SaveBlock(t, ctx, beaconDB, genesis)
	require.NoError(t, beaconDB.SaveState(ctx, headState, headRoot))
	require.NoError(t, beaconDB.SaveState(ctx, headState, genesisRoot))
	util.SaveBlock(t, ctx, beaconDB, headBlock)
	require.NoError(t, beaconDB.SaveFinalizedCheckpoint(ctx, &ethpb.Checkpoint{Epoch: slots.ToEpoch(finalizedSlot), Root: headRoot[:]}))

	require.NoError(t, err)
	require.NoError(t, stateGen.SaveState(ctx, headRoot, headState))
	require.NoError(t, beaconDB.SaveLastValidatedCheckpoint(ctx, &ethpb.Checkpoint{Epoch: slots.ToEpoch(finalizedSlot), Root: headRoot[:]}))
	require.NoError(t, c.StartFromSavedState(headState))
	require.Equal(t, true, c.cfg.ForkChoiceStore.HasNode(headRoot))
	op, err := c.cfg.ForkChoiceStore.IsOptimistic(headRoot)
	require.NoError(t, err)
	require.Equal(t, true, op)
}

// MockClockSetter satisfies the ClockSetter interface for testing the conditions where blockchain.Service should
// call SetGenesis.
type MockClockSetter struct {
	G   *startup.Clock
	Err error
}

var _ startup.ClockSetter = &MockClockSetter{}

// SetClock satisfies the ClockSetter interface.
// The value is written to an exported field 'G' so that it can be accessed in tests.
func (s *MockClockSetter) SetClock(g *startup.Clock) error {
	s.G = g
	return s.Err
}

func TestNotifyIndex(t *testing.T) {
	// Initialize a blobNotifierMap
	bn := &blobNotifierMap{
		seenIndex: make(map[[32]byte][fieldparams.MaxBlobsPerBlock]bool),
		notifiers: make(map[[32]byte]chan uint64),
	}

	// Sample root and index
	var root [32]byte
	copy(root[:], "exampleRoot")

	// Test notifying a new index
	bn.notifyIndex(root, 1)
	if !bn.seenIndex[root][1] {
		t.Errorf("Index was not marked as seen")
	}

	// Test that a new channel is created
	if _, ok := bn.notifiers[root]; !ok {
		t.Errorf("Notifier channel was not created")
	}

	// Test notifying an already seen index
	bn.notifyIndex(root, 1)
	if len(bn.notifiers[root]) > 1 {
		t.Errorf("Notifier channel should not receive multiple messages for the same index")
	}

	// Test notifying a new index again
	bn.notifyIndex(root, 2)
	if !bn.seenIndex[root][2] {
		t.Errorf("Index was not marked as seen")
	}

	// Test that the notifier channel receives the index
	select {
	case idx := <-bn.notifiers[root]:
		if idx != 1 {
			t.Errorf("Received index on channel is incorrect")
		}
	default:
		t.Errorf("Notifier channel did not receive the index")
	}
}
