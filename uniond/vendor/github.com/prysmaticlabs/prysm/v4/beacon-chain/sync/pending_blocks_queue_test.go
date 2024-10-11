package sync

import (
	"context"
	"math"
	"sync"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	gcache "github.com/patrickmn/go-cache"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

//	/- b1 - b2
//
// b0
//
//	\- b3
//
// Test b1 was missing then received and we can process b0 -> b1 -> b2
func TestRegularSyncBeaconBlockSubscriber_ProcessPendingBlocks1(t *testing.T) {
	db := dbtest.SetupDB(t)

	p1 := p2ptest.NewTestP2P(t)
	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain: &mock.ChainService{
				FinalizedCheckPoint: &ethpb.Checkpoint{
					Epoch: 0,
				},
			},
			clock:    startup.NewClock(time.Unix(0, 0), [32]byte{}),
			stateGen: stategen.New(db, doublylinkedtree.New()),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	b0 := util.NewBeaconBlock()
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b0)
	b0Root, err := b0.Block.HashTreeRoot()
	require.NoError(t, err)
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = b0Root[:]
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b3)
	// Incomplete block link
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = b0Root[:]
	b1Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)
	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 2
	b2.Block.ParentRoot = b1Root[:]
	b2Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)

	// Add b2 to the cache
	wsb, err := blocks.NewSignedBeaconBlock(b2)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b2.Block.Slot, wsb, b2Root))

	require.NoError(t, r.processPendingBlocks(context.Background()))
	assert.Equal(t, 1, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 1, len(r.seenPendingBlocks), "Incorrect size for seen pending block")

	// Add b1 to the cache
	wsb, err = blocks.NewSignedBeaconBlock(b1)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b1.Block.Slot, wsb, b1Root))
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b1)

	nBlock := util.NewBeaconBlock()
	nBlock.Block.Slot = b1.Block.Slot
	nRoot, err := nBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	// Insert bad b1 in the cache to verify the good one doesn't get replaced.
	wsb, err = blocks.NewSignedBeaconBlock(nBlock)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(nBlock.Block.Slot, wsb, nRoot))
	require.NoError(t, r.processPendingBlocks(context.Background())) // Marks a block as bad
	require.NoError(t, r.processPendingBlocks(context.Background())) // Bad block removed on second run

	assert.Equal(t, 2, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 2, len(r.seenPendingBlocks), "Incorrect size for seen pending block")
}

func TestRegularSyncBeaconBlockSubscriber_OptimisticStatus(t *testing.T) {
	db := dbtest.SetupDB(t)

	p1 := p2ptest.NewTestP2P(t)
	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain: &mock.ChainService{
				Optimistic: true,
				FinalizedCheckPoint: &ethpb.Checkpoint{
					Epoch: 0,
				},
			},
			clock:    startup.NewClock(time.Unix(0, 0), [32]byte{}),
			stateGen: stategen.New(db, doublylinkedtree.New()),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	b0 := util.NewBeaconBlock()
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b0)
	b0Root, err := b0.Block.HashTreeRoot()
	require.NoError(t, err)
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = b0Root[:]
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b3)
	// Incomplete block link
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = b0Root[:]
	b1Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)
	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 2
	b2.Block.ParentRoot = b1Root[:]
	b2Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)

	// Add b2 to the cache
	wsb, err := blocks.NewSignedBeaconBlock(b2)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b2.Block.Slot, wsb, b2Root))

	require.NoError(t, r.processPendingBlocks(context.Background()))
	assert.Equal(t, 1, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 1, len(r.seenPendingBlocks), "Incorrect size for seen pending block")

	// Add b1 to the cache
	wsb, err = blocks.NewSignedBeaconBlock(b1)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b1.Block.Slot, wsb, b1Root))
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b1)

	nBlock := util.NewBeaconBlock()
	nBlock.Block.Slot = b1.Block.Slot
	nRoot, err := nBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	// Insert bad b1 in the cache to verify the good one doesn't get replaced.
	wsb, err = blocks.NewSignedBeaconBlock(nBlock)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(nBlock.Block.Slot, wsb, nRoot))
	require.NoError(t, r.processPendingBlocks(context.Background())) // Marks a block as bad
	require.NoError(t, r.processPendingBlocks(context.Background())) // Bad block removed on second run

	assert.Equal(t, 2, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 2, len(r.seenPendingBlocks), "Incorrect size for seen pending block")
}

func TestRegularSyncBeaconBlockSubscriber_ExecutionEngineTimesOut(t *testing.T) {
	db := dbtest.SetupDB(t)

	p1 := p2ptest.NewTestP2P(t)
	fcs := doublylinkedtree.New()
	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain: &mock.ChainService{
				FinalizedCheckPoint: &ethpb.Checkpoint{
					Epoch: 0,
				},
				ReceiveBlockMockErr: execution.ErrHTTPTimeout,
			},
			clock:    startup.NewClock(time.Unix(0, 0), [32]byte{}),
			stateGen: stategen.New(db, fcs),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	b0 := util.NewBeaconBlock()
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b0)
	b0Root, err := b0.Block.HashTreeRoot()
	require.NoError(t, err)
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = b0Root[:]
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b3)
	// Incomplete block link
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = b0Root[:]
	b1Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)
	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 2
	b2.Block.ParentRoot = b1Root[:]
	b2Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)

	// Add b2 to the cache
	wsb, err := blocks.NewSignedBeaconBlock(b2)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b2.Block.Slot, wsb, b2Root))

	require.NoError(t, r.processPendingBlocks(context.Background()))
	assert.Equal(t, 1, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 1, len(r.seenPendingBlocks), "Incorrect size for seen pending block")

	// Add b1 to the cache
	wsb, err = blocks.NewSignedBeaconBlock(b1)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b1.Block.Slot, wsb, b1Root))
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b1)

	nBlock := util.NewBeaconBlock()
	nBlock.Block.Slot = b1.Block.Slot
	nRoot, err := nBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	// Insert bad b1 in the cache to verify the good one doesn't get replaced.
	wsb, err = blocks.NewSignedBeaconBlock(nBlock)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(nBlock.Block.Slot, wsb, nRoot))
	require.NoError(t, r.processPendingBlocks(context.Background())) // Marks a block as bad
	require.NoError(t, r.processPendingBlocks(context.Background())) // Bad block removed on second run

	assert.Equal(t, 2, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 2, len(r.seenPendingBlocks), "Incorrect size for seen pending block")
	require.Equal(t, 0, len(r.badBlockCache.Keys())) // Account for the bad block above
	require.Equal(t, 0, len(r.seenBlockCache.Keys()))
}

func TestRegularSync_InsertDuplicateBlocks(t *testing.T) {
	db := dbtest.SetupDB(t)

	p1 := p2ptest.NewTestP2P(t)
	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain: &mock.ChainService{
				FinalizedCheckPoint: &ethpb.Checkpoint{
					Epoch: 0,
					Root:  make([]byte, 32),
				},
			},
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	b0 := util.NewBeaconBlock()
	b0r := [32]byte{'a'}
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b0)
	b0Root, err := b0.Block.HashTreeRoot()
	require.NoError(t, err)
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = b0Root[:]
	b1r := [32]byte{'b'}

	wsb, err := blocks.NewSignedBeaconBlock(b0)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b0.Block.Slot, wsb, b0r))
	require.Equal(t, 1, len(r.pendingBlocksInCache(b0.Block.Slot)), "Block was not added to map")

	wsb, err = blocks.NewSignedBeaconBlock(b1)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b1.Block.Slot, wsb, b1r))
	require.Equal(t, 1, len(r.pendingBlocksInCache(b1.Block.Slot)), "Block was not added to map")

	// Add duplicate block which should not be saved.
	wsb, err = blocks.NewSignedBeaconBlock(b0)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b0.Block.Slot, wsb, b0r))
	require.Equal(t, 1, len(r.pendingBlocksInCache(b0.Block.Slot)), "Block was added to map")

	// Add duplicate block which should not be saved.
	wsb, err = blocks.NewSignedBeaconBlock(b1)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b1.Block.Slot, wsb, b1r))
	require.Equal(t, 1, len(r.pendingBlocksInCache(b1.Block.Slot)), "Block was added to map")

}

func TestRegularSyncBeaconBlockSubscriber_DoNotReprocessBlock(t *testing.T) {
	db := dbtest.SetupDB(t)

	p1 := p2ptest.NewTestP2P(t)
	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain: &mock.ChainService{
				FinalizedCheckPoint: &ethpb.Checkpoint{
					Epoch: 0,
				},
			},
			clock:    startup.NewClock(time.Unix(0, 0), [32]byte{}),
			stateGen: stategen.New(db, doublylinkedtree.New()),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	b0 := util.NewBeaconBlock()
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b0)
	b0Root, err := b0.Block.HashTreeRoot()
	require.NoError(t, err)
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = b0Root[:]
	b3Root, err := b3.Block.HashTreeRoot()
	require.NoError(t, err)

	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b3)

	// Add b3 to the cache
	wsb, err := blocks.NewSignedBeaconBlock(b3)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b3.Block.Slot, wsb, b3Root))

	require.NoError(t, r.processPendingBlocks(context.Background()))
	assert.Equal(t, 0, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 0, len(r.seenPendingBlocks), "Incorrect size for seen pending block")
}

//	/- b1 - b2 - b5
//
// b0
//
//	\- b3 - b4
//
// Test b2 and b3 were missed, after receiving them we can process 2 chains.
func TestRegularSyncBeaconBlockSubscriber_ProcessPendingBlocks_2Chains(t *testing.T) {
	db := dbtest.SetupDB(t)
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	pcl := protocol.ID("/eth2/beacon_chain/req/hello/1/ssz_snappy")
	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		code, errMsg, err := ReadStatusCode(stream, p1.Encoding())
		assert.NoError(t, err)
		if code == 0 {
			t.Error("Expected a non-zero code")
		}
		if errMsg != p2ptypes.ErrWrongForkDigestVersion.Error() {
			t.Logf("Received error string len %d, wanted error string len %d", len(errMsg), len(p2ptypes.ErrWrongForkDigestVersion.Error()))
			t.Errorf("Received unexpected message response in the stream: %s. Wanted %s.", errMsg, p2ptypes.ErrWrongForkDigestVersion.Error())
		}
	})

	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain: &mock.ChainService{
				FinalizedCheckPoint: &ethpb.Checkpoint{
					Epoch: 0,
					Root:  make([]byte, 32),
				},
			},
			clock:    startup.NewClock(time.Unix(0, 0), [32]byte{}),
			stateGen: stategen.New(db, doublylinkedtree.New()),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	p1.Peers().Add(new(enr.Record), p2.PeerID(), nil, network.DirOutbound)
	p1.Peers().SetConnectionState(p2.PeerID(), peers.PeerConnected)
	p1.Peers().SetChainState(p2.PeerID(), &ethpb.Status{})

	b0 := util.NewBeaconBlock()
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b0)
	b0Root, err := b0.Block.HashTreeRoot()
	require.NoError(t, err)
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = b0Root[:]
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b1)
	b1Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)

	// Incomplete block links
	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 2
	b2.Block.ParentRoot = b1Root[:]
	b2Root, err := b2.Block.HashTreeRoot()
	require.NoError(t, err)
	b5 := util.NewBeaconBlock()
	b5.Block.Slot = 5
	b5.Block.ParentRoot = b2Root[:]
	b5Root, err := b5.Block.HashTreeRoot()
	require.NoError(t, err)
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = b0Root[:]
	b3Root, err := b3.Block.HashTreeRoot()
	require.NoError(t, err)
	b4 := util.NewBeaconBlock()
	b4.Block.Slot = 4
	b4.Block.ParentRoot = b3Root[:]
	b4Root, err := b4.Block.HashTreeRoot()
	require.NoError(t, err)

	wsb, err := blocks.NewSignedBeaconBlock(b4)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b4.Block.Slot, wsb, b4Root))
	wsb, err = blocks.NewSignedBeaconBlock(b5)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b5.Block.Slot, wsb, b5Root))

	require.NoError(t, r.processPendingBlocks(context.Background())) // Marks a block as bad
	require.NoError(t, r.processPendingBlocks(context.Background())) // Bad block removed on second run

	assert.Equal(t, 2, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 2, len(r.seenPendingBlocks), "Incorrect size for seen pending block")

	// Add b3 to the cache
	wsb, err = blocks.NewSignedBeaconBlock(b3)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b3.Block.Slot, wsb, b3Root))
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b3)

	require.NoError(t, r.processPendingBlocks(context.Background())) // Marks a block as bad
	require.NoError(t, r.processPendingBlocks(context.Background())) // Bad block removed on second run

	assert.Equal(t, 2, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 2, len(r.seenPendingBlocks), "Incorrect size for seen pending block")

	// Add b2 to the cache
	wsb, err = blocks.NewSignedBeaconBlock(b2)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b2.Block.Slot, wsb, b2Root))

	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b2)

	require.NoError(t, r.processPendingBlocks(context.Background())) // Marks a block as bad
	require.NoError(t, r.processPendingBlocks(context.Background())) // Bad block removed on second run

	assert.Equal(t, 2, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 2, len(r.seenPendingBlocks), "Incorrect size for seen pending block")
}

func TestRegularSyncBeaconBlockSubscriber_PruneOldPendingBlocks(t *testing.T) {
	db := dbtest.SetupDB(t)
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")

	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain: &mock.ChainService{
				FinalizedCheckPoint: &ethpb.Checkpoint{
					Epoch: 1,
					Root:  make([]byte, 32),
				},
			},
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	p1.Peers().Add(new(enr.Record), p1.PeerID(), nil, network.DirOutbound)
	p1.Peers().SetConnectionState(p1.PeerID(), peers.PeerConnected)
	p1.Peers().SetChainState(p1.PeerID(), &ethpb.Status{})

	b0 := util.NewBeaconBlock()
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b0)
	b0Root, err := b0.Block.HashTreeRoot()
	require.NoError(t, err)
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = b0Root[:]
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b1)
	b1Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)

	// Incomplete block links
	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 2
	b2.Block.ParentRoot = b1Root[:]
	b2Root, err := b2.Block.HashTreeRoot()
	require.NoError(t, err)
	b5 := util.NewBeaconBlock()
	b5.Block.Slot = 5
	b5.Block.ParentRoot = b2Root[:]
	b5Root, err := b5.Block.HashTreeRoot()
	require.NoError(t, err)
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = b0Root[:]
	b3Root, err := b3.Block.HashTreeRoot()
	require.NoError(t, err)
	b4 := util.NewBeaconBlock()
	b4.Block.Slot = 4
	b4.Block.ParentRoot = b3Root[:]
	b4Root, err := b4.Block.HashTreeRoot()
	require.NoError(t, err)

	wsb, err := blocks.NewSignedBeaconBlock(b2)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b2.Block.Slot, wsb, b2Root))
	wsb, err = blocks.NewSignedBeaconBlock(b3)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b3.Block.Slot, wsb, b3Root))
	wsb, err = blocks.NewSignedBeaconBlock(b4)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b4.Block.Slot, wsb, b4Root))
	wsb, err = blocks.NewSignedBeaconBlock(b5)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b5.Block.Slot, wsb, b5Root))

	require.NoError(t, r.processPendingBlocks(context.Background()))
	assert.Equal(t, 0, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 0, len(r.seenPendingBlocks), "Incorrect size for seen pending block")
}

func TestService_sortedPendingSlots(t *testing.T) {
	r := &Service{
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}

	var lastSlot primitives.Slot = math.MaxUint64
	wsb, err := blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlock(&ethpb.SignedBeaconBlock{Block: &ethpb.BeaconBlock{Slot: lastSlot}}))
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(lastSlot, wsb, [32]byte{1}))
	wsb, err = blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlock(&ethpb.SignedBeaconBlock{Block: &ethpb.BeaconBlock{Slot: lastSlot - 3}}))
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(lastSlot-3, wsb, [32]byte{2}))
	wsb, err = blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlock(&ethpb.SignedBeaconBlock{Block: &ethpb.BeaconBlock{Slot: lastSlot - 5}}))
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(lastSlot-5, wsb, [32]byte{3}))
	wsb, err = blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlock(&ethpb.SignedBeaconBlock{Block: &ethpb.BeaconBlock{Slot: lastSlot - 2}}))
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(lastSlot-2, wsb, [32]byte{4}))

	want := []primitives.Slot{lastSlot - 5, lastSlot - 3, lastSlot - 2, lastSlot}
	assert.DeepEqual(t, want, r.sortedPendingSlots(), "Unexpected pending slots list")
}

func TestService_BatchRootRequest(t *testing.T) {
	db := dbtest.SetupDB(t)
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")

	chain := &mock.ChainService{
		FinalizedCheckPoint: &ethpb.Checkpoint{
			Epoch: 1,
			Root:  make([]byte, 32),
		},
		ValidatorsRoot: [32]byte{},
		Genesis:        time.Now(),
	}
	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain:    chain,
			clock:    startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	p1.Peers().Add(new(enr.Record), p2.PeerID(), nil, network.DirOutbound)
	p1.Peers().SetConnectionState(p2.PeerID(), peers.PeerConnected)
	p1.Peers().SetChainState(p2.PeerID(), &ethpb.Status{FinalizedEpoch: 2})

	b0 := util.NewBeaconBlock()
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b0)
	b0Root, err := b0.Block.HashTreeRoot()
	require.NoError(t, err)
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ParentRoot = b0Root[:]
	util.SaveBlock(t, context.Background(), r.cfg.beaconDB, b1)
	b1Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)

	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 2
	b2.Block.ParentRoot = b1Root[:]
	b2Root, err := b2.Block.HashTreeRoot()
	require.NoError(t, err)
	b5 := util.NewBeaconBlock()
	b5.Block.Slot = 5
	b5.Block.ParentRoot = b2Root[:]
	b5Root, err := b5.Block.HashTreeRoot()
	require.NoError(t, err)
	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = b0Root[:]
	b3Root, err := b3.Block.HashTreeRoot()
	require.NoError(t, err)
	b4 := util.NewBeaconBlock()
	b4.Block.Slot = 4
	b4.Block.ParentRoot = b3Root[:]
	b4Root, err := b4.Block.HashTreeRoot()
	require.NoError(t, err)

	// Send in duplicated roots to also test deduplicaton.
	sentRoots := p2ptypes.BeaconBlockByRootsReq{b2Root, b2Root, b3Root, b3Root, b4Root, b5Root}
	expectedRoots := p2ptypes.BeaconBlockByRootsReq{b2Root, b3Root, b4Root, b5Root}

	pcl := protocol.ID("/eth2/beacon_chain/req/beacon_blocks_by_root/1/ssz_snappy")
	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		var out p2ptypes.BeaconBlockByRootsReq
		assert.NoError(t, p2.Encoding().DecodeWithMaxLength(stream, &out))
		assert.DeepEqual(t, expectedRoots, out, "Did not receive expected message")
		response := []*ethpb.SignedBeaconBlock{b2, b3, b4, b5}
		for _, blk := range response {
			_, err := stream.Write([]byte{responseCodeSuccess})
			assert.NoError(t, err, "Could not write to stream")
			_, err = p2.Encoding().EncodeWithMaxLength(stream, blk)
			assert.NoError(t, err, "Could not send response back")
		}
		assert.NoError(t, stream.Close())
	})

	require.NoError(t, r.sendBatchRootRequest(context.Background(), sentRoots, rand.NewGenerator()))

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}
	assert.Equal(t, 4, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
	assert.Equal(t, 4, len(r.seenPendingBlocks), "Incorrect size for seen pending block")
}

func TestService_AddPendingBlockToQueueOverMax(t *testing.T) {
	r := &Service{
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}

	b := util.NewBeaconBlock()
	b1 := ethpb.CopySignedBeaconBlock(b)
	b1.Block.StateRoot = []byte{'a'}
	b2 := ethpb.CopySignedBeaconBlock(b)
	b2.Block.StateRoot = []byte{'b'}
	wsb, err := blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(0, wsb, [32]byte{}))
	wsb, err = blocks.NewSignedBeaconBlock(b1)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(0, wsb, [32]byte{1}))
	wsb, err = blocks.NewSignedBeaconBlock(b2)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(0, wsb, [32]byte{2}))

	b3 := ethpb.CopySignedBeaconBlock(b)
	b3.Block.StateRoot = []byte{'c'}
	wsb, err = blocks.NewSignedBeaconBlock(b2)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(0, wsb, [32]byte{3}))
	require.Equal(t, maxBlocksPerSlot, len(r.pendingBlocksInCache(0)))
}

func TestService_ProcessPendingBlockOnCorrectSlot(t *testing.T) {
	ctx := context.Background()
	db := dbtest.SetupDB(t)

	p1 := p2ptest.NewTestP2P(t)
	fcs := doublylinkedtree.New()
	mockChain := mock.ChainService{
		Genesis: time.Unix(time.Now().Unix()-int64(params.BeaconConfig().SecondsPerSlot), 0),
		FinalizedCheckPoint: &ethpb.Checkpoint{
			Epoch: 0,
		}}
	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain:    &mockChain,
			clock:    startup.NewClock(mockChain.Genesis, mockChain.ValidatorsRoot),
			stateGen: stategen.New(db, fcs),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	beaconState, privKeys := util.DeterministicGenesisState(t, 100)
	parentBlock := util.NewBeaconBlock()
	util.SaveBlock(t, ctx, db, parentBlock)
	bRoot, err := parentBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, db.SaveState(ctx, beaconState, bRoot))
	require.NoError(t, db.SaveStateSummary(ctx, &ethpb.StateSummary{Root: bRoot[:]}))
	copied := beaconState.Copy()
	require.NoError(t, copied.SetSlot(1))
	proposerIdx, err := helpers.BeaconProposerIndex(ctx, copied)
	require.NoError(t, err)

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	mockChain.Root = bRoot[:]
	mockChain.State = st

	b1 := util.NewBeaconBlock()
	b1.Block.ParentRoot = bRoot[:]
	b1.Block.Slot = 1
	b1Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)
	b1.Block.ProposerIndex = proposerIdx
	b1.Signature, err = signing.ComputeDomainAndSign(beaconState, 0, b1.Block, params.BeaconConfig().DomainBeaconProposer, privKeys[proposerIdx])
	require.NoError(t, err)

	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 2
	b2.Block.ParentRoot = bRoot[:]
	b2Root, err := b2.Block.HashTreeRoot()
	require.NoError(t, err)

	b3 := util.NewBeaconBlock()
	b3.Block.Slot = 3
	b3.Block.ParentRoot = b2Root[:]
	b3Root, err := b3.Block.HashTreeRoot()
	require.NoError(t, err)

	// Add block1 for slot1
	wsb, err := blocks.NewSignedBeaconBlock(b1)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b1.Block.Slot, wsb, b1Root))
	// Add block2 for slot2
	wsb, err = blocks.NewSignedBeaconBlock(b2)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b2.Block.Slot, wsb, b2Root))
	// Add block3 for slot3
	wsb, err = blocks.NewSignedBeaconBlock(b3)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b3.Block.Slot, wsb, b3Root))

	// processPendingBlocks should process only blocks of the current slot. i.e. slot 1.
	// Then check if the other two blocks are still in the pendingQueue.
	require.NoError(t, r.processPendingBlocks(context.Background()))
	assert.Equal(t, 2, len(r.slotToPendingBlocks.Items()), "Incorrect size for slot to pending blocks cache")
}

func TestService_ProcessBadPendingBlocks(t *testing.T) {
	ctx := context.Background()
	db := dbtest.SetupDB(t)

	p1 := p2ptest.NewTestP2P(t)
	mockChain := mock.ChainService{Genesis: time.Unix(time.Now().Unix()-int64(params.BeaconConfig().SecondsPerSlot), 0),
		FinalizedCheckPoint: &ethpb.Checkpoint{
			Epoch: 0,
		}}
	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain:    &mockChain,
			stateGen: stategen.New(db, doublylinkedtree.New()),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	beaconState, privKeys := util.DeterministicGenesisState(t, 100)
	parentBlock := util.NewBeaconBlock()
	util.SaveBlock(t, ctx, db, parentBlock)
	bRoot, err := parentBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, db.SaveState(ctx, beaconState, bRoot))
	require.NoError(t, db.SaveStateSummary(ctx, &ethpb.StateSummary{Root: bRoot[:]}))
	copied := beaconState.Copy()
	require.NoError(t, copied.SetSlot(1))
	proposerIdx, err := helpers.BeaconProposerIndex(ctx, copied)
	require.NoError(t, err)

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	mockChain.Root = bRoot[:]
	mockChain.State = st

	b1 := util.NewBeaconBlock()
	b1.Block.ParentRoot = bRoot[:]
	b1.Block.Slot = 1
	b1Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)
	b1.Block.ProposerIndex = proposerIdx
	b1.Signature, err = signing.ComputeDomainAndSign(beaconState, 0, b1.Block, params.BeaconConfig().DomainBeaconProposer, privKeys[proposerIdx])
	require.NoError(t, err)

	b := util.NewBeaconBlock()
	b.Block.Slot = 55
	b.Block.ParentRoot = []byte{'A', 'B', 'C'}
	bA, err := blocks.NewSignedBeaconBlock(b)
	assert.NoError(t, err)

	// Add block1 for slot 55
	require.NoError(t, r.insertBlockToPendingQueue(b.Block.Slot, bA, b1Root))
	bB, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
	assert.NoError(t, err)
	// remove with a different block from the same slot.
	require.NoError(t, r.deleteBlockFromPendingQueue(b.Block.Slot, bB, b1Root))
}

func TestAlreadySyncingBlock(t *testing.T) {
	ctx := context.Background()
	db := dbtest.SetupDB(t)
	hook := logTest.NewGlobal()

	mockChain := &mock.ChainService{
		FinalizedCheckPoint: &ethpb.Checkpoint{
			Epoch: 0,
		},
	}

	p1 := p2ptest.NewTestP2P(t)
	r := &Service{
		cfg: &config{
			p2p:      p1,
			beaconDB: db,
			chain:    mockChain,
			clock:    startup.NewClock(time.Unix(0, 0), [32]byte{}),
			stateGen: stategen.New(db, doublylinkedtree.New()),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
	}
	r.initCaches()

	b := util.NewBeaconBlock()
	b.Block.Slot = 2
	bRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err := blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(b.Block.Slot, wsb, bRoot))
	mockChain.SyncingRoot = bRoot
	require.NoError(t, r.processPendingBlocks(ctx))
	require.LogsContain(t, hook, "Skipping pending block already being processed")
}

func TestExpirationCache_PruneOldBlocksCorrectly(t *testing.T) {
	ctx := context.Background()
	db := dbtest.SetupDB(t)

	mockChain := &mock.ChainService{
		FinalizedCheckPoint: &ethpb.Checkpoint{
			Epoch: 0,
		},
	}

	p1 := p2ptest.NewTestP2P(t)
	// Reset expiration time
	currExpTime := pendingBlockExpTime
	defer func() {
		pendingBlockExpTime = currExpTime
	}()
	pendingBlockExpTime = 500 * time.Millisecond

	r := NewService(ctx,
		WithStateGen(stategen.New(db, doublylinkedtree.New())),
		WithDatabase(db),
		WithChainService(mockChain),
		WithP2P(p1),
	)
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	b1.Block.ProposerIndex = 10
	b1Root, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err := blocks.NewSignedBeaconBlock(b1)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(1, wsb, b1Root))

	// Add new block with the same slot.
	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 1
	b2.Block.ProposerIndex = 11
	b2Root, err := b2.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err = blocks.NewSignedBeaconBlock(b2)
	require.NoError(t, err)
	require.NoError(t, r.insertBlockToPendingQueue(1, wsb, b2Root))

	require.Equal(t, true, r.seenPendingBlocks[b1Root])
	require.Equal(t, true, r.seenPendingBlocks[b2Root])
	require.Equal(t, 2, len(r.pendingBlocksInCache(1)))

	// Wait for expiration cache to cleanup and remove old block.
	time.Sleep(2 * pendingBlockExpTime)

	// Run pending queue with expired blocks.
	require.NoError(t, r.processPendingBlocks(ctx))

	assert.Equal(t, false, r.seenPendingBlocks[b1Root])
	assert.Equal(t, false, r.seenPendingBlocks[b2Root])
	assert.Equal(t, 0, len(r.pendingBlocksInCache(1)))
}
