package initialsync

import (
	"context"
	"fmt"
	"sync"
	"testing"
	"time"

	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	p2pm "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2pt "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	p2pTypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestBlocksFetcher_nonSkippedSlotAfter(t *testing.T) {
	peersGen := func(size int) []*peerData {
		blocks := append(makeSequence(1, 64), makeSequence(500, 640)...)
		blocks = append(blocks, makeSequence(51200, 51264)...)
		blocks = append(blocks, 55000)
		blocks = append(blocks, makeSequence(57000, 57256)...)
		var peersData []*peerData
		for i := 0; i < size; i++ {
			peersData = append(peersData, &peerData{
				blocks:         blocks,
				finalizedEpoch: 1800,
				headSlot:       57000,
			})
		}
		return peersData
	}
	chainConfig := struct {
		peers []*peerData
	}{
		peers: peersGen(5),
	}

	mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, chainConfig.peers)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	fetcher := newBlocksFetcher(
		ctx,
		&blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
		},
	)
	fetcher.rateLimiter = leakybucket.NewCollector(6400, 6400, 1*time.Second, false)
	seekSlots := map[primitives.Slot]primitives.Slot{
		0:     1,
		10:    11,
		31:    32,
		32:    33,
		63:    64,
		64:    500,
		160:   500,
		352:   500,
		480:   500,
		512:   513,
		639:   640,
		640:   51200,
		6640:  51200,
		51200: 51201,
	}
	for seekSlot, expectedSlot := range seekSlots {
		t.Run(fmt.Sprintf("range: %d (%d-%d)", expectedSlot-seekSlot, seekSlot, expectedSlot), func(t *testing.T) {
			slot, err := fetcher.nonSkippedSlotAfter(ctx, seekSlot)
			assert.NoError(t, err)
			assert.Equal(t, expectedSlot, slot, "Unexpected slot")
		})
	}

	t.Run("test isolated non-skipped slot", func(t *testing.T) {
		seekSlot := primitives.Slot(51264)
		expectedSlot := primitives.Slot(55000)

		var wg sync.WaitGroup
		wg.Add(1)

		var i int
		go func() {
			for {
				i++
				slot, err := fetcher.nonSkippedSlotAfter(ctx, seekSlot)
				assert.NoError(t, err)
				if slot == expectedSlot {
					wg.Done()
					break
				}
			}
		}()
		if util.WaitTimeout(&wg, 5*time.Second) {
			t.Errorf("Isolated non-skipped slot not found in %d iterations: %v", i, expectedSlot)
		} else {
			log.Debugf("Isolated non-skipped slot found in %d iterations", i)
		}
	})

	t.Run("no peers with higher target epoch available", func(t *testing.T) {
		peers := []*peerData{
			{finalizedEpoch: 3, headSlot: 160},
			{finalizedEpoch: 3, headSlot: 160},
			{finalizedEpoch: 3, headSlot: 160},
			{finalizedEpoch: 8, headSlot: 320},
			{finalizedEpoch: 8, headSlot: 320},
			{finalizedEpoch: 10, headSlot: 320},
			{finalizedEpoch: 10, headSlot: 640},
		}
		p2p := p2pt.NewTestP2P(t)
		connectPeers(t, p2p, peers, p2p.Peers())
		fetcher := newBlocksFetcher(
			ctx,
			&blocksFetcherConfig{
				chain: mc,
				p2p:   p2p,
			},
		)
		mc.FinalizedCheckPoint = &ethpb.Checkpoint{
			Epoch: 10,
		}
		require.NoError(t, mc.State.SetSlot(12*params.BeaconConfig().SlotsPerEpoch))

		fetcher.mode = modeStopOnFinalizedEpoch
		slot, err := fetcher.nonSkippedSlotAfter(ctx, 160)
		assert.ErrorContains(t, errSlotIsTooHigh.Error(), err)
		assert.Equal(t, primitives.Slot(0), slot)

		fetcher.mode = modeNonConstrained
		require.NoError(t, mc.State.SetSlot(20*params.BeaconConfig().SlotsPerEpoch))
		slot, err = fetcher.nonSkippedSlotAfter(ctx, 160)
		assert.ErrorContains(t, errSlotIsTooHigh.Error(), err)
		assert.Equal(t, primitives.Slot(0), slot)
	})
}

func TestBlocksFetcher_findFork(t *testing.T) {
	// Chain graph:
	// A - B - C - D - E
	//      \
	//       - C'- D'- E'- F'- G'
	// Allow fetcher to proceed till E, then connect peer having alternative branch.
	// Test that G' slot can be reached i.e. fetcher can track back and explore alternative paths.
	beaconDB := dbtest.SetupDB(t)
	p2p := p2pt.NewTestP2P(t)

	// Chain contains blocks from 8 epochs (from 0 to 7, 256 is the start slot of epoch8).
	chain1 := extendBlockSequence(t, []*ethpb.SignedBeaconBlock{}, 250)
	finalizedSlot := primitives.Slot(63)
	finalizedEpoch := slots.ToEpoch(finalizedSlot)

	genesisBlock := chain1[0]
	util.SaveBlock(t, context.Background(), beaconDB, genesisBlock)
	genesisRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	mc := &mock.ChainService{
		State: st,
		Root:  genesisRoot[:],
		DB:    beaconDB,
		FinalizedCheckPoint: &ethpb.Checkpoint{
			Epoch: finalizedEpoch,
			Root:  []byte(fmt.Sprintf("finalized_root %d", finalizedEpoch)),
		},
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{},
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(
		ctx,
		&blocksFetcherConfig{
			chain: mc,
			clock: startup.NewClock(mc.Genesis, mc.ValidatorsRoot),
			p2p:   p2p,
			db:    beaconDB,
		},
	)
	fetcher.rateLimiter = leakybucket.NewCollector(6400, 6400, 1*time.Second, false)

	// Consume all chain1 blocks from many peers (alternative fork will be featured by a single peer,
	// and should still be enough to explore alternative paths).
	peers := make([]peer.ID, 0)
	for i := 0; i < 5; i++ {
		peers = append(peers, connectPeerHavingBlocks(t, p2p, chain1, finalizedSlot, p2p.Peers()))
	}

	blockBatchLimit := uint64(flags.Get().BlockBatchLimit) * 2
	pidInd := 0
	for i := uint64(1); i < uint64(len(chain1)); i += blockBatchLimit {
		req := &ethpb.BeaconBlocksByRangeRequest{
			StartSlot: primitives.Slot(i),
			Step:      1,
			Count:     blockBatchLimit,
		}
		blocks, err := fetcher.requestBlocks(ctx, req, peers[pidInd%len(peers)])
		require.NoError(t, err)
		for _, blk := range blocks {
			require.NoError(t, beaconDB.SaveBlock(ctx, blk))
			require.NoError(t, st.SetSlot(blk.Block().Slot()))
		}
		pidInd++
	}

	// Assert that all the blocks from chain1 are known.
	for _, blk := range chain1 {
		blkRoot, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, true, beaconDB.HasBlock(ctx, blkRoot) || mc.HasBlock(ctx, blkRoot))
	}
	assert.Equal(t, primitives.Slot(250), mc.HeadSlot())

	// Assert no blocks on further requests, disallowing to progress.
	req := &ethpb.BeaconBlocksByRangeRequest{
		StartSlot: 251,
		Step:      1,
		Count:     blockBatchLimit,
	}
	blks, err := fetcher.requestBlocks(ctx, req, peers[pidInd%len(peers)])
	require.NoError(t, err)
	assert.Equal(t, 0, len(blks))

	// If no peers with unexplored paths exist, error should be returned.
	fork, err := fetcher.findFork(ctx, 251)
	require.ErrorContains(t, errNoPeersAvailable.Error(), err)
	require.Equal(t, (*forkData)(nil), fork)

	// Add peer that has blocks after 250, but those blocks are orphaned i.e. they do not have common
	// ancestor with what we already have. So, error is expected.
	chain1a := extendBlockSequence(t, []*ethpb.SignedBeaconBlock{}, 265)
	connectPeerHavingBlocks(t, p2p, chain1a, finalizedSlot, p2p.Peers())
	fork, err = fetcher.findFork(ctx, 251)
	require.ErrorContains(t, errNoPeersWithAltBlocks.Error(), err)
	require.Equal(t, (*forkData)(nil), fork)

	// Add peer which has blocks after 250. It is not on another fork, but algorithm
	// is smart enough to link back to common ancestor, w/o discriminating between forks. This is
	// by design: fork exploration is undertaken when FSMs are stuck, so any progress is good.
	chain1b := extendBlockSequence(t, chain1, 64)
	forkSlot1b := primitives.Slot(len(chain1))
	curForkMoreBlocksPeer := connectPeerHavingBlocks(t, p2p, chain1b, finalizedSlot, p2p.Peers())
	fork, err = fetcher.findFork(ctx, 251)
	require.NoError(t, err)

	reqEnd := testForkStartSlot(t, 251) + primitives.Slot(findForkReqRangeSize())
	require.Equal(t, primitives.Slot(len(chain1)), fork.bwb[0].Block.Block().Slot())
	require.Equal(t, int(reqEnd-forkSlot1b), len(fork.bwb))
	require.Equal(t, curForkMoreBlocksPeer, fork.peer)
	// Save all chain1b blocks (so that they do not interfere with alternative fork)
	for _, blk := range chain1b {
		util.SaveBlock(t, ctx, beaconDB, blk)
		require.NoError(t, st.SetSlot(blk.Block.Slot))
	}
	forkSlot := primitives.Slot(129)
	chain2 := extendBlockSequence(t, chain1[:forkSlot], 165)
	// Assert that forked blocks from chain2 are unknown.
	assert.Equal(t, 294, len(chain2))
	for _, blk := range chain2[forkSlot:] {
		blkRoot, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, false, beaconDB.HasBlock(ctx, blkRoot) || mc.HasBlock(ctx, blkRoot))
	}

	// Search for alternative paths (add single peer having alternative path).
	alternativePeer := connectPeerHavingBlocks(t, p2p, chain2, finalizedSlot, p2p.Peers())
	fork, err = fetcher.findFork(ctx, 251)
	require.NoError(t, err)
	assert.Equal(t, alternativePeer, fork.peer)
	assert.Equal(t, 65, len(fork.bwb))
	ind := forkSlot
	for _, blk := range fork.bwb {
		require.Equal(t, blk.Block.Block().Slot(), chain2[ind].Block.Slot)
		ind++
	}

	// Process returned blocks and then attempt to extend chain (ensuring that parent block exists).
	for _, blk := range fork.bwb {
		require.NoError(t, beaconDB.SaveBlock(ctx, blk.Block))
		require.NoError(t, st.SetSlot(blk.Block.Block().Slot()))
	}
	assert.Equal(t, forkSlot.Add(uint64(len(fork.bwb)-1)), mc.HeadSlot())
	for i := forkSlot.Add(uint64(len(fork.bwb))); i < primitives.Slot(len(chain2)); i++ {
		blk := chain2[i]
		require.Equal(t, blk.Block.Slot, i, "incorrect block selected for slot %d", i)
		// Only save is parent block exists.
		parentRoot := bytesutil.ToBytes32(blk.Block.ParentRoot)
		if beaconDB.HasBlock(ctx, parentRoot) || mc.HasBlock(ctx, parentRoot) {
			util.SaveBlock(t, ctx, beaconDB, blk)
			require.NoError(t, st.SetSlot(blk.Block.Slot))
		}
	}

	// Assert that all the blocks from chain2 are known.
	for _, blk := range chain2 {
		blkRoot, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, true, beaconDB.HasBlock(ctx, blkRoot) || mc.HasBlock(ctx, blkRoot), "slot %d", blk.Block.Slot)
	}
}

func testForkStartSlot(t *testing.T, slot primitives.Slot) primitives.Slot {
	// When we call find fork, the way we find the first common ancestor is:
	// - start at the first slot of the given epoch (epochStart below)
	// - look back an additional epoch, which findForkWithPeer comments say is done to ensure
	//   there will be more overlap, since this routine kicks off when we're losing sight of the canonical chain
	// - add one: this happens in nonSkippedSlotAfterWithPeersTarget and isn't really explained why this method takes
	//   the slot before the beginning of the search range and not the actual search range.
	// Anyway, since findFork returns the last common ancestor between the known chain and the forked chain,
	// we need to figure out how much the two overlap so we can ignore the common blocks when looking at the
	// size of the response.
	epochStart, err := slots.EpochStart(slots.ToEpoch(slot))
	require.NoError(t, err)
	return 1 + (epochStart - params.BeaconConfig().SlotsPerEpoch)
}

func TestBlocksFetcher_findForkWithPeer(t *testing.T) {
	beaconDB := dbtest.SetupDB(t)
	p1 := p2pt.NewTestP2P(t)

	knownBlocks := extendBlockSequence(t, []*ethpb.SignedBeaconBlock{}, 128)
	genesisBlock := knownBlocks[0]
	util.SaveBlock(t, context.Background(), beaconDB, genesisBlock)
	genesisRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	mc := &mock.ChainService{
		State:          st,
		Root:           genesisRoot[:],
		DB:             beaconDB,
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{},
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(
		ctx,
		&blocksFetcherConfig{
			chain: mc,
			clock: startup.NewClock(mc.Genesis, mc.ValidatorsRoot),
			p2p:   p1,
			db:    beaconDB,
		},
	)
	fetcher.rateLimiter = leakybucket.NewCollector(6400, 6400, 1*time.Second, false)

	for _, blk := range knownBlocks {
		util.SaveBlock(t, ctx, beaconDB, blk)
		require.NoError(t, st.SetSlot(blk.Block.Slot))
	}

	t.Run("slot is too early", func(t *testing.T) {
		p2 := p2pt.NewTestP2P(t)
		_, err := fetcher.findForkWithPeer(ctx, p2.PeerID(), 0)
		assert.ErrorContains(t, "slot is too low to backtrack", err)
	})

	t.Run("no peer status", func(t *testing.T) {
		p2 := p2pt.NewTestP2P(t)
		_, err := fetcher.findForkWithPeer(ctx, p2.PeerID(), 64)
		assert.ErrorContains(t, "cannot obtain peer's status", err)
	})

	t.Run("no non-skipped blocks found", func(t *testing.T) {
		p2 := p2pt.NewTestP2P(t)
		p1.Connect(p2)
		defer func() {
			assert.NoError(t, p1.Disconnect(p2.PeerID()))
		}()
		p1.Peers().SetChainState(p2.PeerID(), &ethpb.Status{
			HeadRoot: nil,
			HeadSlot: 0,
		})
		_, err := fetcher.findForkWithPeer(ctx, p2.PeerID(), 64)
		assert.ErrorContains(t, "cannot locate non-empty slot for a peer", err)
	})

	t.Run("no diverging blocks", func(t *testing.T) {
		p2 := connectPeerHavingBlocks(t, p1, knownBlocks, 64, p1.Peers())
		time.Sleep(100 * time.Millisecond)
		defer func() {
			assert.NoError(t, p1.Disconnect(p2))
		}()
		_, err := fetcher.findForkWithPeer(ctx, p2, 64)
		assert.ErrorContains(t, "no alternative blocks exist within scanned range", err)
	})

	t.Run("first block is diverging - backtrack successfully", func(t *testing.T) {
		forkedSlot := primitives.Slot(24)
		altBlocks := extendBlockSequence(t, knownBlocks[:forkedSlot], 128)
		p2 := connectPeerHavingBlocks(t, p1, altBlocks, 128, p1.Peers())
		time.Sleep(100 * time.Millisecond)
		defer func() {
			assert.NoError(t, p1.Disconnect(p2))
		}()
		fork, err := fetcher.findForkWithPeer(ctx, p2, 64)
		require.NoError(t, err)
		require.Equal(t, 10, len(fork.bwb))
		assert.Equal(t, forkedSlot, fork.bwb[0].Block.Block().Slot(), "Expected slot %d to be ancestor", forkedSlot)
	})

	t.Run("first block is diverging - no common ancestor", func(t *testing.T) {
		altBlocks := extendBlockSequence(t, []*ethpb.SignedBeaconBlock{}, 128)
		p2 := connectPeerHavingBlocks(t, p1, altBlocks, 128, p1.Peers())
		time.Sleep(100 * time.Millisecond)
		defer func() {
			assert.NoError(t, p1.Disconnect(p2))
		}()
		_, err := fetcher.findForkWithPeer(ctx, p2, 64)
		require.ErrorContains(t, "failed to find common ancestor", err)
	})

	t.Run("mid block is diverging - no backtrack is necessary", func(t *testing.T) {
		forkedSlot := primitives.Slot(60)
		altBlocks := extendBlockSequence(t, knownBlocks[:forkedSlot], 128)
		p2 := connectPeerHavingBlocks(t, p1, altBlocks, 128, p1.Peers())
		time.Sleep(100 * time.Millisecond)
		defer func() {
			assert.NoError(t, p1.Disconnect(p2))
		}()
		fork, err := fetcher.findForkWithPeer(ctx, p2, 64)
		require.NoError(t, err)

		reqEnd := testForkStartSlot(t, 64) + primitives.Slot(findForkReqRangeSize())
		expectedLen := reqEnd - forkedSlot
		// there are 4 blocks that are different before the beginning
		require.Equal(t, int(expectedLen), len(fork.bwb))
		assert.Equal(t, primitives.Slot(60), fork.bwb[0].Block.Block().Slot())
	})
}

func TestTestForkStartSlot(t *testing.T) {
	require.Equal(t, primitives.Slot(33), testForkStartSlot(t, 64))
	require.Equal(t, primitives.Slot(193), testForkStartSlot(t, 251))
}

func consumeBlockRootRequest(t *testing.T, p *p2pt.TestP2P) func(network.Stream) {
	return func(stream network.Stream) {
		defer func() {
			_err := stream.Close()
			_ = _err
		}()

		req := new(p2pTypes.BeaconBlockByRootsReq)
		assert.NoError(t, p.Encoding().DecodeWithMaxLength(stream, req))
	}
}

func TestBlocksFetcher_findAncestor(t *testing.T) {
	beaconDB := dbtest.SetupDB(t)
	p2p := p2pt.NewTestP2P(t)

	knownBlocks := extendBlockSequence(t, []*ethpb.SignedBeaconBlock{}, 128)
	finalizedSlot := primitives.Slot(63)
	finalizedEpoch := slots.ToEpoch(finalizedSlot)

	genesisBlock := knownBlocks[0]
	util.SaveBlock(t, context.Background(), beaconDB, genesisBlock)
	genesisRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	mc := &mock.ChainService{
		State: st,
		Root:  genesisRoot[:],
		DB:    beaconDB,
		FinalizedCheckPoint: &ethpb.Checkpoint{
			Epoch: finalizedEpoch,
			Root:  []byte(fmt.Sprintf("finalized_root %d", finalizedEpoch)),
		},
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{},
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(
		ctx,
		&blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
			db:    beaconDB,
		},
	)
	fetcher.rateLimiter = leakybucket.NewCollector(6400, 6400, 1*time.Second, false)
	pcl := fmt.Sprintf("%s/ssz_snappy", p2pm.RPCBlocksByRootTopicV1)

	t.Run("error on request", func(t *testing.T) {
		p2 := p2pt.NewTestP2P(t)
		p2p.Connect(p2)

		wsb, err := blocks.NewSignedBeaconBlock(knownBlocks[4])
		require.NoError(t, err)
		_, err = fetcher.findAncestor(ctx, p2.PeerID(), wsb)
		assert.ErrorContains(t, "protocols not supported", err)
	})

	t.Run("no blocks", func(t *testing.T) {
		p2 := p2pt.NewTestP2P(t)
		p2p.Connect(p2)

		p2.SetStreamHandler(pcl, consumeBlockRootRequest(t, p2))

		wsb, err := blocks.NewSignedBeaconBlock(knownBlocks[4])
		require.NoError(t, err)

		fork, err := fetcher.findAncestor(ctx, p2.PeerID(), wsb)
		assert.ErrorContains(t, "no common ancestor found", err)
		assert.Equal(t, (*forkData)(nil), fork)
	})
}

func TestBlocksFetcher_currentHeadAndTargetEpochs(t *testing.T) {
	tests := []struct {
		name               string
		syncMode           syncMode
		peers              []*peerData
		ourFinalizedEpoch  primitives.Epoch
		ourHeadSlot        primitives.Slot
		expectedHeadEpoch  primitives.Epoch
		targetEpoch        primitives.Epoch
		targetEpochSupport int
	}{
		{
			name:               "ignore lower epoch peers in best finalized",
			syncMode:           modeStopOnFinalizedEpoch,
			ourHeadSlot:        5 * params.BeaconConfig().SlotsPerEpoch,
			expectedHeadEpoch:  4,
			ourFinalizedEpoch:  4,
			targetEpoch:        10,
			targetEpochSupport: 3,
			peers: []*peerData{
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 8, headSlot: 320},
				{finalizedEpoch: 8, headSlot: 320},
				{finalizedEpoch: 10, headSlot: 320},
				{finalizedEpoch: 10, headSlot: 320},
				{finalizedEpoch: 10, headSlot: 320},
			},
		},
		{
			name:               "resolve ties in best finalized",
			syncMode:           modeStopOnFinalizedEpoch,
			ourHeadSlot:        5 * params.BeaconConfig().SlotsPerEpoch,
			expectedHeadEpoch:  4,
			ourFinalizedEpoch:  4,
			targetEpoch:        10,
			targetEpochSupport: 3,
			peers: []*peerData{
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 8, headSlot: 320},
				{finalizedEpoch: 8, headSlot: 320},
				{finalizedEpoch: 8, headSlot: 320},
				{finalizedEpoch: 10, headSlot: 320},
				{finalizedEpoch: 10, headSlot: 320},
				{finalizedEpoch: 10, headSlot: 320},
			},
		},
		{
			name:               "best non-finalized",
			syncMode:           modeNonConstrained,
			ourHeadSlot:        5 * params.BeaconConfig().SlotsPerEpoch,
			expectedHeadEpoch:  5,
			ourFinalizedEpoch:  4,
			targetEpoch:        20,
			targetEpochSupport: 1,
			peers: []*peerData{
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 3, headSlot: 160},
				{finalizedEpoch: 8, headSlot: 320},
				{finalizedEpoch: 8, headSlot: 320},
				{finalizedEpoch: 10, headSlot: 320},
				{finalizedEpoch: 10, headSlot: 320},
				{finalizedEpoch: 10, headSlot: 320},
				{finalizedEpoch: 15, headSlot: 640},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, tt.peers)
			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()
			fetcher := newBlocksFetcher(
				ctx,
				&blocksFetcherConfig{
					chain: mc,
					p2p:   p2p,
				},
			)
			mc.FinalizedCheckPoint = &ethpb.Checkpoint{
				Epoch: tt.ourFinalizedEpoch,
			}
			require.NoError(t, mc.State.SetSlot(tt.ourHeadSlot))
			fetcher.mode = tt.syncMode

			// Head and target epochs calculation.
			headEpoch, targetEpoch, peers := fetcher.calculateHeadAndTargetEpochs()
			assert.Equal(t, tt.expectedHeadEpoch, headEpoch, "Unexpected head epoch")
			assert.Equal(t, tt.targetEpoch, targetEpoch, "Unexpected target epoch")
			assert.Equal(t, tt.targetEpochSupport, len(peers), "Unexpected number of peers supporting target epoch")

			// Best finalized and non-finalized slots.
			finalizedSlot := params.BeaconConfig().SlotsPerEpoch.Mul(uint64(tt.targetEpoch))
			if tt.syncMode == modeStopOnFinalizedEpoch {
				assert.Equal(t, finalizedSlot, fetcher.bestFinalizedSlot(), "Unexpected finalized slot")
			} else {
				assert.Equal(t, finalizedSlot, fetcher.bestNonFinalizedSlot(), "Unexpected non-finalized slot")
			}
		})
	}
}
