package sync

import (
	"context"
	"math/big"
	"sync"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	gcache "github.com/patrickmn/go-cache"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	db "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	mockExecution "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	p2pTypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	"github.com/prysmaticlabs/prysm/v4/proto/eth/v2"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestRecentBeaconBlocksRPCHandler_ReturnsBlocks(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	d := db.SetupDB(t)

	var blkRoots p2pTypes.BeaconBlockByRootsReq
	// Populate the database with blocks that would match the request.
	for i := primitives.Slot(1); i < 11; i++ {
		blk := util.NewBeaconBlock()
		blk.Block.Slot = i
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		util.SaveBlock(t, context.Background(), d, blk)
		blkRoots = append(blkRoots, root)
	}

	r := &Service{cfg: &config{p2p: p1, beaconDB: d, clock: startup.NewClock(time.Unix(0, 0), [32]byte{})}, rateLimiter: newRateLimiter(p1)}
	r.cfg.chain = &mock.ChainService{ValidatorsRoot: [32]byte{}}
	pcl := protocol.ID(p2p.RPCBlocksByRootTopicV1)
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(10000, 10000, time.Second, false)

	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		for i := range blkRoots {
			expectSuccess(t, stream)
			res := util.NewBeaconBlock()
			assert.NoError(t, r.cfg.p2p.Encoding().DecodeWithMaxLength(stream, res))
			if uint64(res.Block.Slot) != uint64(i+1) {
				t.Errorf("Received unexpected block slot %d but wanted %d", res.Block.Slot, i+1)
			}
		}
	})

	stream1, err := p1.BHost.NewStream(context.Background(), p2.BHost.ID(), pcl)
	require.NoError(t, err)
	err = r.beaconBlocksRootRPCHandler(context.Background(), &blkRoots, stream1)
	assert.NoError(t, err)

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}
}

func TestRecentBeaconBlocksRPCHandler_ReturnsBlocks_ReconstructsPayload(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	d := db.SetupDB(t)

	// Start service with 160 as allowed blocks capacity (and almost zero capacity recovery).
	parent := bytesutil.PadTo([]byte("parentHash"), fieldparams.RootLength)
	stateRoot := bytesutil.PadTo([]byte("stateRoot"), fieldparams.RootLength)
	receiptsRoot := bytesutil.PadTo([]byte("receiptsRoot"), fieldparams.RootLength)
	logsBloom := bytesutil.PadTo([]byte("logs"), fieldparams.LogsBloomLength)
	tx := gethTypes.NewTransaction(
		0,
		common.HexToAddress("095e7baea6a6c7c4c2dfeb977efac326af552d87"),
		big.NewInt(0), 0, big.NewInt(0),
		nil,
	)
	txs := []*gethTypes.Transaction{tx}
	encodedBinaryTxs := make([][]byte, 1)
	var err error
	encodedBinaryTxs[0], err = txs[0].MarshalBinary()
	require.NoError(t, err)
	blockHash := bytesutil.ToBytes32([]byte("foo"))
	payload := &enginev1.ExecutionPayload{
		ParentHash:    parent,
		FeeRecipient:  make([]byte, fieldparams.FeeRecipientLength),
		StateRoot:     stateRoot,
		ReceiptsRoot:  receiptsRoot,
		LogsBloom:     logsBloom,
		PrevRandao:    blockHash[:],
		BlockNumber:   0,
		GasLimit:      0,
		GasUsed:       0,
		Timestamp:     0,
		ExtraData:     make([]byte, 0),
		BlockHash:     blockHash[:],
		BaseFeePerGas: bytesutil.PadTo([]byte("baseFeePerGas"), fieldparams.RootLength),
		Transactions:  encodedBinaryTxs,
	}
	wrappedPayload, err := blocks.WrappedExecutionPayload(payload)
	require.NoError(t, err)
	header, err := blocks.PayloadToHeader(wrappedPayload)
	require.NoError(t, err)

	var blkRoots p2pTypes.BeaconBlockByRootsReq
	// Populate the database with blocks that would match the request.
	for i := primitives.Slot(1); i < 11; i++ {
		blk := util.NewBlindedBeaconBlockBellatrix()
		blk.Block.Body.ExecutionPayloadHeader = header
		blk.Block.Slot = i
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		wsb, err := blocks.NewSignedBeaconBlock(blk)
		require.NoError(t, err)
		require.NoError(t, d.SaveBlock(context.Background(), wsb))
		blkRoots = append(blkRoots, root)
	}

	mockEngine := &mockExecution.EngineClient{
		ExecutionPayloadByBlockHash: map[[32]byte]*enginev1.ExecutionPayload{
			blockHash: payload,
		},
	}
	r := &Service{cfg: &config{
		p2p:                           p1,
		beaconDB:                      d,
		executionPayloadReconstructor: mockEngine,
		chain:                         &mock.ChainService{ValidatorsRoot: [32]byte{}},
		clock:                         startup.NewClock(time.Unix(0, 0), [32]byte{}),
	}, rateLimiter: newRateLimiter(p1)}
	pcl := protocol.ID(p2p.RPCBlocksByRootTopicV1)
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(10000, 10000, time.Second, false)

	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		for i := range blkRoots {
			expectSuccess(t, stream)
			res := util.NewBeaconBlockBellatrix()
			assert.NoError(t, r.cfg.p2p.Encoding().DecodeWithMaxLength(stream, res))
			if uint64(res.Block.Slot) != uint64(i+1) {
				t.Errorf("Received unexpected block slot %d but wanted %d", res.Block.Slot, i+1)
			}
		}
		require.Equal(t, uint64(10), mockEngine.NumReconstructedPayloads)
	})

	stream1, err := p1.BHost.NewStream(context.Background(), p2.BHost.ID(), pcl)
	require.NoError(t, err)
	err = r.beaconBlocksRootRPCHandler(context.Background(), &blkRoots, stream1)
	assert.NoError(t, err)

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}
}

func TestRecentBeaconBlocks_RPCRequestSent(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.DelaySend = true

	blockA := util.NewBeaconBlock()
	blockA.Block.Slot = 111
	blockB := util.NewBeaconBlock()
	blockB.Block.Slot = 40
	// Set up a head state with data we expect.
	blockARoot, err := blockA.Block.HashTreeRoot()
	require.NoError(t, err)
	blockBRoot, err := blockB.Block.HashTreeRoot()
	require.NoError(t, err)
	genesisState, err := transition.GenesisBeaconState(context.Background(), nil, 0, &ethpb.Eth1Data{})
	require.NoError(t, err)
	require.NoError(t, genesisState.SetSlot(111))
	require.NoError(t, genesisState.UpdateBlockRootAtIndex(111%uint64(params.BeaconConfig().SlotsPerHistoricalRoot), blockARoot))
	finalizedCheckpt := &ethpb.Checkpoint{
		Epoch: 5,
		Root:  blockBRoot[:],
	}

	expectedRoots := p2pTypes.BeaconBlockByRootsReq{blockBRoot, blockARoot}

	chain := &mock.ChainService{
		State:               genesisState,
		FinalizedCheckPoint: finalizedCheckpt,
		Root:                blockARoot[:],
		Genesis:             time.Now(),
		ValidatorsRoot:      [32]byte{},
	}
	r := &Service{
		cfg: &config{
			p2p:   p1,
			chain: chain,
			clock: startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
		ctx:                 context.Background(),
		rateLimiter:         newRateLimiter(p1),
	}

	// Setup streams
	pcl := protocol.ID("/eth2/beacon_chain/req/beacon_blocks_by_root/1/ssz_snappy")
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(10000, 10000, time.Second, false)

	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		out := new(p2pTypes.BeaconBlockByRootsReq)
		assert.NoError(t, p2.Encoding().DecodeWithMaxLength(stream, out))
		assert.DeepEqual(t, &expectedRoots, out, "Did not receive expected message")
		response := []*ethpb.SignedBeaconBlock{blockB, blockA}
		for _, blk := range response {
			_, err := stream.Write([]byte{responseCodeSuccess})
			assert.NoError(t, err, "Could not write to stream")
			_, err = p2.Encoding().EncodeWithMaxLength(stream, blk)
			assert.NoError(t, err, "Could not send response back")
		}
		assert.NoError(t, stream.Close())
	})

	p1.Connect(p2)
	require.NoError(t, r.sendRecentBeaconBlocksRequest(context.Background(), &expectedRoots, p2.PeerID()))

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}
}

func TestRecentBeaconBlocks_RPCRequestSent_IncorrectRoot(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.DelaySend = true

	blockA := util.NewBeaconBlock()
	blockA.Block.Slot = 111
	blockB := util.NewBeaconBlock()
	blockB.Block.Slot = 40
	// Set up a head state with data we expect.
	blockARoot, err := blockA.Block.HashTreeRoot()
	require.NoError(t, err)
	blockBRoot, err := blockB.Block.HashTreeRoot()
	require.NoError(t, err)
	genesisState, err := transition.GenesisBeaconState(context.Background(), nil, 0, &ethpb.Eth1Data{})
	require.NoError(t, err)
	require.NoError(t, genesisState.SetSlot(111))
	require.NoError(t, genesisState.UpdateBlockRootAtIndex(111%uint64(params.BeaconConfig().SlotsPerHistoricalRoot), blockARoot))
	finalizedCheckpt := &ethpb.Checkpoint{
		Epoch: 5,
		Root:  blockBRoot[:],
	}

	expectedRoots := p2pTypes.BeaconBlockByRootsReq{blockBRoot, blockARoot}

	chain := &mock.ChainService{
		State:               genesisState,
		FinalizedCheckPoint: finalizedCheckpt,
		Root:                blockARoot[:],
		Genesis:             time.Now(),
		ValidatorsRoot:      [32]byte{},
	}
	r := &Service{
		cfg: &config{
			p2p:   p1,
			chain: chain,
			clock: startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
		},
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:   make(map[[32]byte]bool),
		ctx:                 context.Background(),
		rateLimiter:         newRateLimiter(p1),
	}

	// Setup streams
	pcl := protocol.ID("/eth2/beacon_chain/req/beacon_blocks_by_root/1/ssz_snappy")
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(10000, 10000, time.Second, false)

	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		out := new(p2pTypes.BeaconBlockByRootsReq)
		assert.NoError(t, p2.Encoding().DecodeWithMaxLength(stream, out))
		assert.DeepEqual(t, &expectedRoots, out, "Did not receive expected message")
		blockB.Block.Slot = 123123 // Give it a bad slot
		response := []*ethpb.SignedBeaconBlock{blockB, blockA}
		for _, blk := range response {
			_, err := stream.Write([]byte{responseCodeSuccess})
			assert.NoError(t, err, "Could not write to stream")
			_, err = p2.Encoding().EncodeWithMaxLength(stream, blk)
			assert.NoError(t, err, "Could not send response back")
		}
		assert.NoError(t, stream.Close())
	})

	p1.Connect(p2)
	require.ErrorContains(t, "received unexpected block with root", r.sendRecentBeaconBlocksRequest(context.Background(), &expectedRoots, p2.PeerID()))
}

func TestRecentBeaconBlocksRPCHandler_HandleZeroBlocks(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	d := db.SetupDB(t)

	r := &Service{cfg: &config{p2p: p1, beaconDB: d}, rateLimiter: newRateLimiter(p1)}
	pcl := protocol.ID(p2p.RPCBlocksByRootTopicV1)
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(1, 1, time.Second, false)

	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		expectFailure(t, 1, "no block roots provided in request", stream)
	})

	stream1, err := p1.BHost.NewStream(context.Background(), p2.BHost.ID(), pcl)
	require.NoError(t, err)
	err = r.beaconBlocksRootRPCHandler(context.Background(), &p2pTypes.BeaconBlockByRootsReq{}, stream1)
	assert.ErrorContains(t, "no block roots provided", err)
	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}

	r.rateLimiter.RLock() // retrieveCollector requires a lock to be held.
	defer r.rateLimiter.RUnlock()
	lter, err := r.rateLimiter.retrieveCollector(topic)
	require.NoError(t, err)
	assert.Equal(t, 1, int(lter.Count(stream1.Conn().RemotePeer().String())))
}

func TestRequestPendingBlobs(t *testing.T) {
	s := &Service{cfg: &config{blobStorage: filesystem.NewEphemeralBlobStorage(t)}}
	t.Run("old block should not fail", func(t *testing.T) {
		b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
		require.NoError(t, err)
		require.NoError(t, s.requestPendingBlobs(context.Background(), b, [32]byte{}, "test"))
	})
	t.Run("empty commitment block should not fail", func(t *testing.T) {
		b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
		require.NoError(t, err)
		require.NoError(t, s.requestPendingBlobs(context.Background(), b, [32]byte{}, "test"))
	})
	t.Run("unsupported protocol", func(t *testing.T) {
		p1 := p2ptest.NewTestP2P(t)
		p2 := p2ptest.NewTestP2P(t)
		p1.Connect(p2)
		require.Equal(t, 1, len(p1.BHost.Network().Peers()))
		chain := &mock.ChainService{
			FinalizedCheckPoint: &ethpb.Checkpoint{
				Epoch: 1,
				Root:  make([]byte, 32),
			},
			ValidatorsRoot: [32]byte{},
			Genesis:        time.Now(),
		}
		p1.Peers().Add(new(enr.Record), p2.PeerID(), nil, network.DirOutbound)
		p1.Peers().SetConnectionState(p2.PeerID(), peers.PeerConnected)
		p1.Peers().SetChainState(p2.PeerID(), &ethpb.Status{FinalizedEpoch: 1})
		s := &Service{
			cfg: &config{
				p2p:         p1,
				chain:       chain,
				clock:       startup.NewClock(time.Unix(0, 0), [32]byte{}),
				beaconDB:    db.SetupDB(t),
				blobStorage: filesystem.NewEphemeralBlobStorage(t),
			},
		}
		b := util.NewBeaconBlockDeneb()
		b.Block.Body.BlobKzgCommitments = make([][]byte, 1)
		b1, err := blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		require.ErrorContains(t, "protocols not supported", s.requestPendingBlobs(context.Background(), b1, [32]byte{}, p2.PeerID()))
	})
}

func TestConstructPendingBlobsRequest(t *testing.T) {
	d := db.SetupDB(t)
	bs := filesystem.NewEphemeralBlobStorage(t)
	s := &Service{cfg: &config{beaconDB: d, blobStorage: bs}}
	ctx := context.Background()

	// No unknown indices.
	root := [32]byte{1}
	count := 3
	actual, err := s.constructPendingBlobsRequest(ctx, root, count)
	require.NoError(t, err)
	require.Equal(t, 3, len(actual))
	for i, id := range actual {
		require.Equal(t, uint64(i), id.Index)
		require.DeepEqual(t, root[:], id.BlockRoot)
	}

	// Has indices.
	header := &ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			ParentRoot: bytesutil.PadTo([]byte{}, 32),
			StateRoot:  bytesutil.PadTo([]byte{}, 32),
			BodyRoot:   bytesutil.PadTo([]byte{}, 32),
		},
		Signature: bytesutil.PadTo([]byte{}, 96),
	}
	blobSidecars := []blocks.ROBlob{
		util.GenerateTestDenebBlobSidecar(t, root, header, 0, bytesutil.PadTo([]byte{}, 48), make([][]byte, 0)),
		util.GenerateTestDenebBlobSidecar(t, root, header, 2, bytesutil.PadTo([]byte{}, 48), make([][]byte, 0)),
	}
	vscs, err := verification.BlobSidecarSliceNoop(blobSidecars)
	require.NoError(t, err)
	for i := range vscs {
		require.NoError(t, bs.Save(vscs[i]))
	}

	expected := []*eth.BlobIdentifier{
		{Index: 1, BlockRoot: root[:]},
	}
	actual, err = s.constructPendingBlobsRequest(ctx, root, count)
	require.NoError(t, err)
	require.Equal(t, expected[0].Index, actual[0].Index)
	require.DeepEqual(t, expected[0].BlockRoot, actual[0].BlockRoot)
}

func TestFilterUnknownIndices(t *testing.T) {
	haveIndices := [fieldparams.MaxBlobsPerBlock]bool{true, true, true, false, false, false}

	blockRoot := [32]byte{}
	count := 5

	expected := []*eth.BlobIdentifier{
		{Index: 3, BlockRoot: blockRoot[:]},
		{Index: 4, BlockRoot: blockRoot[:]},
	}

	actual := requestsForMissingIndices(haveIndices, count, blockRoot)
	require.Equal(t, len(expected), len(actual))
	require.Equal(t, expected[0].Index, actual[0].Index)
	require.DeepEqual(t, actual[0].BlockRoot, expected[0].BlockRoot)
	require.Equal(t, expected[1].Index, actual[1].Index)
	require.DeepEqual(t, actual[1].BlockRoot, expected[1].BlockRoot)
}
