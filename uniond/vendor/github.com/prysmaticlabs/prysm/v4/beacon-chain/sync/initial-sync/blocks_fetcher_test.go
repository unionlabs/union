package initialsync

import (
	"context"
	"fmt"
	"math/rand"
	"sort"
	"sync"
	"testing"
	"time"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/libp2p/go-libp2p/core/network"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	p2pm "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2pt "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	beaconsync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/verify"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestBlocksFetcher_InitStartStop(t *testing.T) {
	mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, []*peerData{})

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(
		ctx,
		&blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
		},
	)

	t.Run("check for leaked goroutines", func(t *testing.T) {
		err := fetcher.start()
		require.NoError(t, err)
		fetcher.stop() // should block up until all resources are reclaimed
		select {
		case <-fetcher.requestResponses():
		default:
			t.Error("fetchResponses channel is leaked")
		}
	})

	t.Run("re-starting of stopped fetcher", func(t *testing.T) {
		assert.ErrorContains(t, errFetcherCtxIsDone.Error(), fetcher.start())
	})

	t.Run("multiple stopping attempts", func(t *testing.T) {
		fetcher := newBlocksFetcher(
			context.Background(),
			&blocksFetcherConfig{
				chain: mc,
				p2p:   p2p,
			})
		require.NoError(t, fetcher.start())
		fetcher.stop()
		fetcher.stop()
	})

	t.Run("cancellation", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		fetcher := newBlocksFetcher(
			ctx,
			&blocksFetcherConfig{
				chain: mc,
				p2p:   p2p,
			})
		require.NoError(t, fetcher.start())
		cancel()
		fetcher.stop()
	})

	t.Run("peer filter capacity weight", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		fetcher := newBlocksFetcher(
			ctx,
			&blocksFetcherConfig{
				chain:                    mc,
				p2p:                      p2p,
				peerFilterCapacityWeight: 2,
			})
		require.NoError(t, fetcher.start())
		assert.Equal(t, peerFilterCapacityWeight, fetcher.capacityWeight)
	})
}

func TestBlocksFetcher_RoundRobin(t *testing.T) {
	slotsInBatch := primitives.Slot(flags.Get().BlockBatchLimit)
	requestsGenerator := func(start, end, batchSize primitives.Slot) []*fetchRequestParams {
		var requests []*fetchRequestParams
		for i := start; i <= end; i += batchSize {
			requests = append(requests, &fetchRequestParams{
				start: i,
				count: uint64(batchSize),
			})
		}
		return requests
	}
	tests := []struct {
		name               string
		expectedBlockSlots []primitives.Slot
		peers              []*peerData
		requests           []*fetchRequestParams
	}{
		{
			name:               "Single peer with all blocks",
			expectedBlockSlots: makeSequence(1, 3*slotsInBatch),
			peers: []*peerData{
				{
					blocks:         makeSequence(1, 3*slotsInBatch),
					finalizedEpoch: slots.ToEpoch(3 * slotsInBatch),
					headSlot:       3 * slotsInBatch,
				},
			},
			requests: requestsGenerator(1, 3*slotsInBatch, slotsInBatch),
		},
		{
			name:               "Single peer with all blocks (many small requests)",
			expectedBlockSlots: makeSequence(1, 3*slotsInBatch),
			peers: []*peerData{
				{
					blocks:         makeSequence(1, 3*slotsInBatch),
					finalizedEpoch: slots.ToEpoch(3 * slotsInBatch),
					headSlot:       3 * slotsInBatch,
				},
			},
			requests: requestsGenerator(1, 3*slotsInBatch, slotsInBatch/4),
		},
		{
			name:               "Multiple peers with all blocks",
			expectedBlockSlots: makeSequence(1, 3*slotsInBatch),
			peers: []*peerData{
				{
					blocks:         makeSequence(1, 3*slotsInBatch),
					finalizedEpoch: slots.ToEpoch(3 * slotsInBatch),
					headSlot:       3 * slotsInBatch,
				},
				{
					blocks:         makeSequence(1, 3*slotsInBatch),
					finalizedEpoch: slots.ToEpoch(3 * slotsInBatch),
					headSlot:       3 * slotsInBatch,
				},
				{
					blocks:         makeSequence(1, 3*slotsInBatch),
					finalizedEpoch: slots.ToEpoch(3 * slotsInBatch),
					headSlot:       3 * slotsInBatch,
				},
			},
			requests: requestsGenerator(1, 3*slotsInBatch, slotsInBatch),
		},
		{
			name:               "Multiple peers with skipped slots",
			expectedBlockSlots: append(makeSequence(1, 64), makeSequence(500, 640)...), // up to 18th epoch
			peers: []*peerData{
				{
					blocks:         append(makeSequence(1, 64), makeSequence(500, 640)...),
					finalizedEpoch: 18,
					headSlot:       640,
				},
				{
					blocks:         append(makeSequence(1, 64), makeSequence(500, 640)...),
					finalizedEpoch: 18,
					headSlot:       640,
				},
				{
					blocks:         append(makeSequence(1, 64), makeSequence(500, 640)...),
					finalizedEpoch: 18,
					headSlot:       640,
				},
				{
					blocks:         append(makeSequence(1, 64), makeSequence(500, 640)...),
					finalizedEpoch: 18,
					headSlot:       640,
				},
				{
					blocks:         append(makeSequence(1, 64), makeSequence(500, 640)...),
					finalizedEpoch: 18,
					headSlot:       640,
				},
			},
			requests: []*fetchRequestParams{
				{
					start: 1,
					count: uint64(slotsInBatch),
				},
				{
					start: slotsInBatch + 1,
					count: uint64(slotsInBatch),
				},
				{
					start: 2*slotsInBatch + 1,
					count: uint64(slotsInBatch),
				},
				{
					start: 500,
					count: 53,
				},
				{
					start: 553,
					count: 200,
				},
			},
		},
		{
			name:               "Multiple peers with failures",
			expectedBlockSlots: makeSequence(1, 2*slotsInBatch),
			peers: []*peerData{
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
					failureSlots:   makeSequence(1, 32), // first epoch
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
			},
			requests: []*fetchRequestParams{
				{
					start: 1,
					count: uint64(slotsInBatch),
				},
				{
					start: slotsInBatch + 1,
					count: uint64(slotsInBatch),
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cache.initializeRootCache(tt.expectedBlockSlots, t)

			beaconDB := dbtest.SetupDB(t)

			p := p2pt.NewTestP2P(t)
			connectPeers(t, p, tt.peers, p.Peers())
			cache.RLock()
			genesisRoot := cache.rootCache[0]
			cache.RUnlock()

			util.SaveBlock(t, context.Background(), beaconDB, util.NewBeaconBlock())

			st, err := util.NewBeaconState()
			require.NoError(t, err)

			gt := time.Now()
			vr := [32]byte{}
			clock := startup.NewClock(gt, vr)
			mc := &mock.ChainService{
				State: st,
				Root:  genesisRoot[:],
				DB:    beaconDB,
				FinalizedCheckPoint: &ethpb.Checkpoint{
					Epoch: 0,
				},
				Genesis:        time.Now(),
				ValidatorsRoot: [32]byte{},
			}

			ctx, cancel := context.WithCancel(context.Background())
			fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
				chain: mc,
				p2p:   p,
				clock: clock,
			})
			require.NoError(t, fetcher.start())

			var wg sync.WaitGroup
			wg.Add(len(tt.requests)) // how many block requests we are going to make
			go func() {
				wg.Wait()
				log.Debug("Stopping fetcher")
				fetcher.stop()
			}()

			processFetchedBlocks := func() ([]blocks.BlockWithROBlobs, error) {
				defer cancel()
				var unionRespBlocks []blocks.BlockWithROBlobs

				for {
					select {
					case resp, ok := <-fetcher.requestResponses():
						if !ok { // channel closed, aggregate
							return unionRespBlocks, nil
						}

						if resp.err != nil {
							log.WithError(resp.err).Debug("Block fetcher returned error")
						} else {
							unionRespBlocks = append(unionRespBlocks, resp.bwb...)
							if len(resp.bwb) == 0 {
								log.WithFields(logrus.Fields{
									"start": resp.start,
									"count": resp.count,
								}).Debug("Received empty slot")
							}
						}

						wg.Done()
					case <-ctx.Done():
						log.Debug("Context closed, exiting goroutine")
						return unionRespBlocks, nil
					}
				}
			}

			maxExpectedBlocks := uint64(0)
			for _, requestParams := range tt.requests {
				err = fetcher.scheduleRequest(context.Background(), requestParams.start, requestParams.count)
				assert.NoError(t, err)
				maxExpectedBlocks += requestParams.count
			}

			bwb, err := processFetchedBlocks()
			assert.NoError(t, err)

			sort.Sort(blocks.BlockWithROBlobsSlice(bwb))
			ss := make([]primitives.Slot, len(bwb))
			for i, b := range bwb {
				ss[i] = b.Block.Block().Slot()
			}

			log.WithFields(logrus.Fields{
				"blocksLen": len(bwb),
				"slots":     ss,
			}).Debug("Finished block fetching")

			if len(bwb) > int(maxExpectedBlocks) {
				t.Errorf("Too many blocks returned. Wanted %d got %d", maxExpectedBlocks, len(bwb))
			}
			assert.Equal(t, len(tt.expectedBlockSlots), len(bwb), "Processes wrong number of blocks")
			var receivedBlockSlots []primitives.Slot
			for _, b := range bwb {
				receivedBlockSlots = append(receivedBlockSlots, b.Block.Block().Slot())
			}
			missing := slice.NotSlot(slice.IntersectionSlot(tt.expectedBlockSlots, receivedBlockSlots), tt.expectedBlockSlots)
			if len(missing) > 0 {
				t.Errorf("Missing blocks at slots %v", missing)
			}
		})
	}
}

func TestBlocksFetcher_scheduleRequest(t *testing.T) {
	blockBatchLimit := uint64(flags.Get().BlockBatchLimit)
	t.Run("context cancellation", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{})
		cancel()
		assert.ErrorContains(t, "context canceled", fetcher.scheduleRequest(ctx, 1, blockBatchLimit))
	})

	t.Run("unblock on context cancellation", func(t *testing.T) {
		fetcher := newBlocksFetcher(context.Background(), &blocksFetcherConfig{})
		for i := 0; i < maxPendingRequests; i++ {
			assert.NoError(t, fetcher.scheduleRequest(context.Background(), 1, blockBatchLimit))
		}

		// Will block on next request (and wait until requests are either processed or context is closed).
		go func() {
			fetcher.cancel()
		}()
		assert.ErrorContains(t, errFetcherCtxIsDone.Error(),
			fetcher.scheduleRequest(context.Background(), 1, blockBatchLimit))
	})
}
func TestBlocksFetcher_handleRequest(t *testing.T) {
	blockBatchLimit := flags.Get().BlockBatchLimit
	chainConfig := struct {
		expectedBlockSlots []primitives.Slot
		peers              []*peerData
	}{
		expectedBlockSlots: makeSequence(1, primitives.Slot(blockBatchLimit)),
		peers: []*peerData{
			{
				blocks:         makeSequence(1, 320),
				finalizedEpoch: 8,
				headSlot:       320,
			},
			{
				blocks:         makeSequence(1, 320),
				finalizedEpoch: 8,
				headSlot:       320,
			},
		},
	}

	mc, p2p, _ := initializeTestServices(t, chainConfig.expectedBlockSlots, chainConfig.peers)
	mc.ValidatorsRoot = [32]byte{}
	mc.Genesis = time.Now()

	t.Run("context cancellation", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
			clock: startup.NewClock(mc.Genesis, mc.ValidatorsRoot),
		})

		cancel()
		response := fetcher.handleRequest(ctx, 1, uint64(blockBatchLimit))
		assert.ErrorContains(t, "context canceled", response.err)
	})

	t.Run("receive blocks", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
			clock: startup.NewClock(mc.Genesis, mc.ValidatorsRoot),
		})

		requestCtx, reqCancel := context.WithTimeout(context.Background(), 2*time.Second)
		defer reqCancel()
		go func() {
			response := fetcher.handleRequest(requestCtx, 1 /* start */, uint64(blockBatchLimit) /* count */)
			select {
			case <-ctx.Done():
			case fetcher.fetchResponses <- response:
			}
		}()

		var bwb []blocks.BlockWithROBlobs
		select {
		case <-ctx.Done():
			t.Error(ctx.Err())
		case resp := <-fetcher.requestResponses():
			if resp.err != nil {
				t.Error(resp.err)
			} else {
				bwb = resp.bwb
			}
		}
		if uint64(len(bwb)) != uint64(blockBatchLimit) {
			t.Errorf("incorrect number of blocks returned, expected: %v, got: %v", blockBatchLimit, len(bwb))
		}

		var receivedBlockSlots []primitives.Slot
		for _, b := range bwb {
			receivedBlockSlots = append(receivedBlockSlots, b.Block.Block().Slot())
		}
		missing := slice.NotSlot(slice.IntersectionSlot(chainConfig.expectedBlockSlots, receivedBlockSlots), chainConfig.expectedBlockSlots)
		if len(missing) > 0 {
			t.Errorf("Missing blocks at slots %v", missing)
		}
	})
}

func TestBlocksFetcher_requestBeaconBlocksByRange(t *testing.T) {
	blockBatchLimit := flags.Get().BlockBatchLimit
	chainConfig := struct {
		expectedBlockSlots []primitives.Slot
		peers              []*peerData
	}{
		expectedBlockSlots: makeSequence(1, 320),
		peers: []*peerData{
			{
				blocks:         makeSequence(1, 320),
				finalizedEpoch: 8,
				headSlot:       320,
			},
			{
				blocks:         makeSequence(1, 320),
				finalizedEpoch: 8,
				headSlot:       320,
			},
		},
	}

	mc, p2p, _ := initializeTestServices(t, chainConfig.expectedBlockSlots, chainConfig.peers)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	fetcher := newBlocksFetcher(
		ctx,
		&blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
		})

	_, peerIDs := p2p.Peers().BestFinalized(params.BeaconConfig().MaxPeersToSync, slots.ToEpoch(mc.HeadSlot()))
	req := &ethpb.BeaconBlocksByRangeRequest{
		StartSlot: 1,
		Step:      1,
		Count:     uint64(blockBatchLimit),
	}
	blocks, err := fetcher.requestBlocks(ctx, req, peerIDs[0])
	assert.NoError(t, err)
	assert.Equal(t, uint64(blockBatchLimit), uint64(len(blocks)), "Incorrect number of blocks returned")

	// Test context cancellation.
	ctx, cancel = context.WithCancel(context.Background())
	cancel()
	_, err = fetcher.requestBlocks(ctx, req, peerIDs[0])
	assert.ErrorContains(t, "context canceled", err)
}

func TestBlocksFetcher_RequestBlocksRateLimitingLocks(t *testing.T) {
	p1 := p2pt.NewTestP2P(t)
	p2 := p2pt.NewTestP2P(t)
	p3 := p2pt.NewTestP2P(t)
	p1.Connect(p2)
	p1.Connect(p3)
	require.Equal(t, 2, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	req := &ethpb.BeaconBlocksByRangeRequest{
		StartSlot: 100,
		Step:      1,
		Count:     64,
	}

	topic := p2pm.RPCBlocksByRangeTopicV1
	protocol := libp2pcore.ProtocolID(topic + p2.Encoding().ProtocolSuffix())
	streamHandlerFn := func(stream network.Stream) {
		assert.NoError(t, stream.Close())
	}
	p2.BHost.SetStreamHandler(protocol, streamHandlerFn)
	p3.BHost.SetStreamHandler(protocol, streamHandlerFn)

	burstFactor := uint64(flags.Get().BlockBatchLimitBurstFactor)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{p2p: p1})
	fetcher.rateLimiter = leakybucket.NewCollector(float64(req.Count), int64(req.Count*burstFactor), 1*time.Second, false)
	gt := time.Now()
	vr := [32]byte{}
	fetcher.chain = &mock.ChainService{Genesis: gt, ValidatorsRoot: vr}
	hook := logTest.NewGlobal()
	wg := new(sync.WaitGroup)
	wg.Add(1)
	go func() {
		// Exhaust available rate for p2, so that rate limiting is triggered.
		for i := uint64(0); i <= burstFactor; i++ {
			if i == burstFactor {
				// The next request will trigger rate limiting for p2. Now, allow concurrent
				// p3 data request (p3 shouldn't be rate limited).
				time.AfterFunc(1*time.Second, func() {
					wg.Done()
				})
			}
			_, err := fetcher.requestBlocks(ctx, req, p2.PeerID())
			if err != nil {
				assert.ErrorContains(t, errFetcherCtxIsDone.Error(), err)
			}
		}
	}()

	// Wait until p2 exhausts its rate and is spinning on rate limiting timer.
	wg.Wait()

	// The next request should NOT trigger rate limiting as rate is exhausted for p2, not p3.
	ch := make(chan struct{}, 1)
	go func() {
		_, err := fetcher.requestBlocks(ctx, req, p3.PeerID())
		assert.NoError(t, err)
		ch <- struct{}{}
	}()
	timer := time.NewTimer(2 * time.Second)
	select {
	case <-timer.C:
		t.Error("p3 takes too long to respond: lock contention")
	case <-ch:
		// p3 responded w/o waiting for rate limiter's lock (on which p2 spins).
	}
	// Make sure that p2 has been rate limited.
	require.LogsContain(t, hook, fmt.Sprintf("msg=\"Slowing down for rate limit\" peer=%s", p2.PeerID()))
}

func TestBlocksFetcher_WaitForBandwidth(t *testing.T) {
	p1 := p2pt.NewTestP2P(t)
	p2 := p2pt.NewTestP2P(t)
	p1.Connect(p2)
	require.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	req := &ethpb.BeaconBlocksByRangeRequest{
		Count: 64,
	}

	topic := p2pm.RPCBlocksByRangeTopicV1
	protocol := libp2pcore.ProtocolID(topic + p2.Encoding().ProtocolSuffix())
	streamHandlerFn := func(stream network.Stream) {
		assert.NoError(t, stream.Close())
	}
	p2.BHost.SetStreamHandler(protocol, streamHandlerFn)

	burstFactor := uint64(flags.Get().BlockBatchLimitBurstFactor)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{p2p: p1})
	fetcher.rateLimiter = leakybucket.NewCollector(float64(req.Count), int64(req.Count*burstFactor), 5*time.Second, false)
	gt := time.Now()
	vr := [32]byte{}
	fetcher.chain = &mock.ChainService{Genesis: gt, ValidatorsRoot: vr}
	start := time.Now()
	assert.NoError(t, fetcher.waitForBandwidth(p2.PeerID(), 10))
	dur := time.Since(start)
	assert.Equal(t, true, dur < time.Millisecond, "waited excessively for bandwidth")
	fetcher.rateLimiter.Add(p2.PeerID().String(), int64(req.Count*burstFactor))
	start = time.Now()
	assert.NoError(t, fetcher.waitForBandwidth(p2.PeerID(), req.Count))
	dur = time.Since(start)
	assert.Equal(t, float64(5), dur.Truncate(1*time.Second).Seconds(), "waited excessively for bandwidth")
}

func TestBlocksFetcher_requestBlocksFromPeerReturningInvalidBlocks(t *testing.T) {
	p1 := p2pt.NewTestP2P(t)
	tests := []struct {
		name         string
		req          *ethpb.BeaconBlocksByRangeRequest
		handlerGenFn func(req *ethpb.BeaconBlocksByRangeRequest) func(stream network.Stream)
		wantedErr    string
		validate     func(req *ethpb.BeaconBlocksByRangeRequest, blocks []interfaces.ReadOnlySignedBeaconBlock)
	}{
		{
			name: "no error",
			req: &ethpb.BeaconBlocksByRangeRequest{
				StartSlot: 100,
				Step:      4,
				Count:     64,
			},
			handlerGenFn: func(req *ethpb.BeaconBlocksByRangeRequest) func(stream network.Stream) {
				return func(stream network.Stream) {
					for i := req.StartSlot; i < req.StartSlot.Add(req.Count*req.Step); i += primitives.Slot(req.Step) {
						blk := util.NewBeaconBlock()
						blk.Block.Slot = i
						tor := startup.NewClock(time.Now(), [32]byte{})
						wsb, err := blocks.NewSignedBeaconBlock(blk)
						require.NoError(t, err)
						assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
					}
					assert.NoError(t, stream.Close())
				}
			},
			validate: func(req *ethpb.BeaconBlocksByRangeRequest, blocks []interfaces.ReadOnlySignedBeaconBlock) {
				assert.Equal(t, req.Count, uint64(len(blocks)))
			},
		},
		{
			name: "too many blocks",
			req: &ethpb.BeaconBlocksByRangeRequest{
				StartSlot: 100,
				Step:      1,
				Count:     64,
			},
			handlerGenFn: func(req *ethpb.BeaconBlocksByRangeRequest) func(stream network.Stream) {
				return func(stream network.Stream) {
					for i := req.StartSlot; i < req.StartSlot.Add(req.Count*req.Step+1); i += primitives.Slot(req.Step) {
						blk := util.NewBeaconBlock()
						blk.Block.Slot = i
						tor := startup.NewClock(time.Now(), [32]byte{})
						wsb, err := blocks.NewSignedBeaconBlock(blk)
						require.NoError(t, err)
						assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
					}
					assert.NoError(t, stream.Close())
				}
			},
			validate: func(req *ethpb.BeaconBlocksByRangeRequest, blocks []interfaces.ReadOnlySignedBeaconBlock) {
				assert.Equal(t, 0, len(blocks))
			},
			wantedErr: beaconsync.ErrInvalidFetchedData.Error(),
		},
		{
			name: "not in a consecutive order",
			req: &ethpb.BeaconBlocksByRangeRequest{
				StartSlot: 100,
				Step:      1,
				Count:     64,
			},
			handlerGenFn: func(req *ethpb.BeaconBlocksByRangeRequest) func(stream network.Stream) {
				return func(stream network.Stream) {
					blk := util.NewBeaconBlock()
					blk.Block.Slot = 163
					tor := startup.NewClock(time.Now(), [32]byte{})
					wsb, err := blocks.NewSignedBeaconBlock(blk)
					require.NoError(t, err)
					assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))

					blk = util.NewBeaconBlock()
					blk.Block.Slot = 162
					wsb, err = blocks.NewSignedBeaconBlock(blk)
					require.NoError(t, err)
					assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
					assert.NoError(t, stream.Close())
				}
			},
			validate: func(req *ethpb.BeaconBlocksByRangeRequest, blocks []interfaces.ReadOnlySignedBeaconBlock) {
				assert.Equal(t, 0, len(blocks))
			},
			wantedErr: beaconsync.ErrInvalidFetchedData.Error(),
		},
		{
			name: "same slot number",
			req: &ethpb.BeaconBlocksByRangeRequest{
				StartSlot: 100,
				Step:      1,
				Count:     64,
			},
			handlerGenFn: func(req *ethpb.BeaconBlocksByRangeRequest) func(stream network.Stream) {
				return func(stream network.Stream) {
					blk := util.NewBeaconBlock()
					blk.Block.Slot = 160
					tor := startup.NewClock(time.Now(), [32]byte{})

					wsb, err := blocks.NewSignedBeaconBlock(blk)
					require.NoError(t, err)
					assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))

					blk = util.NewBeaconBlock()
					blk.Block.Slot = 160
					wsb, err = blocks.NewSignedBeaconBlock(blk)
					require.NoError(t, err)
					assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
					assert.NoError(t, stream.Close())
				}
			},
			validate: func(req *ethpb.BeaconBlocksByRangeRequest, blocks []interfaces.ReadOnlySignedBeaconBlock) {
				assert.Equal(t, 0, len(blocks))
			},
			wantedErr: beaconsync.ErrInvalidFetchedData.Error(),
		},
		{
			name: "slot is too low",
			req: &ethpb.BeaconBlocksByRangeRequest{
				StartSlot: 100,
				Step:      1,
				Count:     64,
			},
			handlerGenFn: func(req *ethpb.BeaconBlocksByRangeRequest) func(stream network.Stream) {
				return func(stream network.Stream) {
					defer func() {
						assert.NoError(t, stream.Close())
					}()
					for i := req.StartSlot; i < req.StartSlot.Add(req.Count*req.Step); i += primitives.Slot(req.Step) {
						blk := util.NewBeaconBlock()
						tor := startup.NewClock(time.Now(), [32]byte{})
						// Patch mid block, with invalid slot number.
						if i == req.StartSlot.Add(req.Count*req.Step/2) {
							blk.Block.Slot = req.StartSlot - 1
							wsb, err := blocks.NewSignedBeaconBlock(blk)
							require.NoError(t, err)
							assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
							break
						}
						blk.Block.Slot = i
						wsb, err := blocks.NewSignedBeaconBlock(blk)
						require.NoError(t, err)
						assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
					}
				}
			},
			wantedErr: beaconsync.ErrInvalidFetchedData.Error(),
			validate: func(req *ethpb.BeaconBlocksByRangeRequest, blocks []interfaces.ReadOnlySignedBeaconBlock) {
				assert.Equal(t, 0, len(blocks))
			},
		},
		{
			name: "slot is too high",
			req: &ethpb.BeaconBlocksByRangeRequest{
				StartSlot: 100,
				Step:      1,
				Count:     64,
			},
			handlerGenFn: func(req *ethpb.BeaconBlocksByRangeRequest) func(stream network.Stream) {
				return func(stream network.Stream) {
					defer func() {
						assert.NoError(t, stream.Close())
					}()
					for i := req.StartSlot; i < req.StartSlot.Add(req.Count*req.Step); i += primitives.Slot(req.Step) {
						blk := util.NewBeaconBlock()
						tor := startup.NewClock(time.Now(), [32]byte{})
						// Patch mid block, with invalid slot number.
						if i == req.StartSlot.Add(req.Count*req.Step/2) {
							blk.Block.Slot = req.StartSlot.Add(req.Count * req.Step)
							wsb, err := blocks.NewSignedBeaconBlock(blk)
							require.NoError(t, err)
							assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
							break
						}
						blk.Block.Slot = i
						wsb, err := blocks.NewSignedBeaconBlock(blk)
						require.NoError(t, err)
						assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
					}
				}
			},
			wantedErr: beaconsync.ErrInvalidFetchedData.Error(),
			validate: func(req *ethpb.BeaconBlocksByRangeRequest, blocks []interfaces.ReadOnlySignedBeaconBlock) {
				assert.Equal(t, 0, len(blocks))
			},
		},
		{
			name: "valid step increment",
			req: &ethpb.BeaconBlocksByRangeRequest{
				StartSlot: 100,
				Step:      5,
				Count:     64,
			},
			handlerGenFn: func(req *ethpb.BeaconBlocksByRangeRequest) func(stream network.Stream) {
				return func(stream network.Stream) {
					blk := util.NewBeaconBlock()
					blk.Block.Slot = 100
					tor := startup.NewClock(time.Now(), [32]byte{})
					wsb, err := blocks.NewSignedBeaconBlock(blk)
					require.NoError(t, err)
					assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))

					blk = util.NewBeaconBlock()
					blk.Block.Slot = 105
					wsb, err = blocks.NewSignedBeaconBlock(blk)
					require.NoError(t, err)
					assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
					assert.NoError(t, stream.Close())
				}
			},
			validate: func(req *ethpb.BeaconBlocksByRangeRequest, blocks []interfaces.ReadOnlySignedBeaconBlock) {
				assert.Equal(t, 2, len(blocks))
			},
		},
		{
			name: "invalid step increment",
			req: &ethpb.BeaconBlocksByRangeRequest{
				StartSlot: 100,
				Step:      5,
				Count:     64,
			},
			handlerGenFn: func(req *ethpb.BeaconBlocksByRangeRequest) func(stream network.Stream) {
				return func(stream network.Stream) {
					blk := util.NewBeaconBlock()
					blk.Block.Slot = 100
					tor := startup.NewClock(time.Now(), [32]byte{})
					wsb, err := blocks.NewSignedBeaconBlock(blk)
					require.NoError(t, err)
					assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))

					blk = util.NewBeaconBlock()
					blk.Block.Slot = 103
					wsb, err = blocks.NewSignedBeaconBlock(blk)
					require.NoError(t, err)
					assert.NoError(t, beaconsync.WriteBlockChunk(stream, tor, p1.Encoding(), wsb))
					assert.NoError(t, stream.Close())
				}
			},
			validate: func(req *ethpb.BeaconBlocksByRangeRequest, blocks []interfaces.ReadOnlySignedBeaconBlock) {
				assert.Equal(t, 0, len(blocks))
			},
			wantedErr: beaconsync.ErrInvalidFetchedData.Error(),
		},
	}

	topic := p2pm.RPCBlocksByRangeTopicV1
	protocol := libp2pcore.ProtocolID(topic + p1.Encoding().ProtocolSuffix())

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{p2p: p1, chain: &mock.ChainService{Genesis: time.Now(), ValidatorsRoot: [32]byte{}}})
	fetcher.rateLimiter = leakybucket.NewCollector(0.000001, 640, 1*time.Second, false)

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p2 := p2pt.NewTestP2P(t)
			p1.Connect(p2)

			p2.BHost.SetStreamHandler(protocol, tt.handlerGenFn(tt.req))
			blocks, err := fetcher.requestBlocks(ctx, tt.req, p2.PeerID())
			if tt.wantedErr != "" {
				assert.ErrorContains(t, tt.wantedErr, err)
			} else {
				assert.NoError(t, err)
				tt.validate(tt.req, blocks)
			}
		})
	}
}

func TestTimeToWait(t *testing.T) {
	tests := []struct {
		name          string
		wanted        int64
		rem           int64
		capacity      int64
		timeTillEmpty time.Duration
		want          time.Duration
	}{
		{
			name:          "Limiter has sufficient blocks",
			wanted:        64,
			rem:           64,
			capacity:      320,
			timeTillEmpty: 200 * time.Second,
			want:          0 * time.Second,
		},
		{
			name:          "Limiter has full capacity remaining",
			wanted:        350,
			rem:           320,
			capacity:      320,
			timeTillEmpty: 0 * time.Second,
			want:          0 * time.Second,
		},
		{
			name:          "Limiter has reached full capacity",
			wanted:        64,
			rem:           0,
			capacity:      640,
			timeTillEmpty: 60 * time.Second,
			want:          6 * time.Second,
		},
		{
			name:          "Requesting full capacity from peer",
			wanted:        640,
			rem:           0,
			capacity:      640,
			timeTillEmpty: 60 * time.Second,
			want:          60 * time.Second,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := timeToWait(tt.wanted, tt.rem, tt.capacity, tt.timeTillEmpty); got != tt.want {
				t.Errorf("timeToWait() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestSortBlobs(t *testing.T) {
	_, blobs := util.ExtendBlocksPlusBlobs(t, []blocks.ROBlock{}, 10)
	shuffled := make([]blocks.ROBlob, len(blobs))
	for i := range blobs {
		shuffled[i] = blobs[i]
	}
	rand.Shuffle(len(shuffled), func(i, j int) {
		shuffled[i], shuffled[j] = shuffled[j], shuffled[i]
	})
	sorted := sortBlobs(shuffled)
	require.Equal(t, len(sorted), len(shuffled))
	for i := range blobs {
		expect := blobs[i]
		actual := sorted[i]
		require.Equal(t, expect.Slot(), actual.Slot())
		require.Equal(t, expect.Index, actual.Index)
		require.Equal(t, bytesutil.ToBytes48(expect.KzgCommitment), bytesutil.ToBytes48(actual.KzgCommitment))
		require.Equal(t, expect.BlockRoot(), actual.BlockRoot())
	}
}

func TestLowestSlotNeedsBlob(t *testing.T) {
	blks, _ := util.ExtendBlocksPlusBlobs(t, []blocks.ROBlock{}, 10)
	sbbs := make([]interfaces.ReadOnlySignedBeaconBlock, len(blks))
	for i := range blks {
		sbbs[i] = blks[i]
	}
	retentionStart := primitives.Slot(5)
	bwb, err := sortedBlockWithVerifiedBlobSlice(sbbs)
	require.NoError(t, err)
	lowest := lowestSlotNeedsBlob(retentionStart, bwb)
	require.Equal(t, retentionStart, *lowest)
	higher := primitives.Slot(len(blks) + 1)
	lowest = lowestSlotNeedsBlob(higher, bwb)
	var nilSlot *primitives.Slot
	require.Equal(t, nilSlot, lowest)

	blks, _ = util.ExtendBlocksPlusBlobs(t, []blocks.ROBlock{}, 10)
	sbbs = make([]interfaces.ReadOnlySignedBeaconBlock, len(blks))
	for i := range blks {
		sbbs[i] = blks[i]
	}
	bwb, err = sortedBlockWithVerifiedBlobSlice(sbbs)
	require.NoError(t, err)
	retentionStart = bwb[5].Block.Block().Slot()
	next := bwb[6].Block.Block().Slot()
	skip := bwb[5].Block.Block()
	bwb[5].Block, _ = util.GenerateTestDenebBlockWithSidecar(t, skip.ParentRoot(), skip.Slot(), 0)
	lowest = lowestSlotNeedsBlob(retentionStart, bwb)
	require.Equal(t, next, *lowest)
}

func TestBlobRequest(t *testing.T) {
	var nilReq *ethpb.BlobSidecarsByRangeRequest
	// no blocks
	req := blobRequest([]blocks.BlockWithROBlobs{}, 0)
	require.Equal(t, nilReq, req)
	blks, _ := util.ExtendBlocksPlusBlobs(t, []blocks.ROBlock{}, 10)
	sbbs := make([]interfaces.ReadOnlySignedBeaconBlock, len(blks))
	for i := range blks {
		sbbs[i] = blks[i]
	}
	bwb, err := sortedBlockWithVerifiedBlobSlice(sbbs)
	require.NoError(t, err)
	maxBlkSlot := primitives.Slot(len(blks) - 1)

	tooHigh := primitives.Slot(len(blks) + 1)
	req = blobRequest(bwb, tooHigh)
	require.Equal(t, nilReq, req)

	req = blobRequest(bwb, maxBlkSlot)
	require.Equal(t, uint64(1), req.Count)
	require.Equal(t, maxBlkSlot, req.StartSlot)

	halfway := primitives.Slot(5)
	req = blobRequest(bwb, halfway)
	require.Equal(t, halfway, req.StartSlot)
	// adding 1 to include the halfway slot itself
	require.Equal(t, uint64(1+maxBlkSlot-halfway), req.Count)

	before := bwb[0].Block.Block().Slot()
	allAfter := bwb[1:]
	req = blobRequest(allAfter, before)
	require.Equal(t, allAfter[0].Block.Block().Slot(), req.StartSlot)
	require.Equal(t, len(allAfter), int(req.Count))
}

func testSequenceBlockWithBlob(t *testing.T, nblocks int) ([]blocks.BlockWithROBlobs, []blocks.ROBlob) {
	blks, blobs := util.ExtendBlocksPlusBlobs(t, []blocks.ROBlock{}, nblocks)
	sbbs := make([]interfaces.ReadOnlySignedBeaconBlock, len(blks))
	for i := range blks {
		sbbs[i] = blks[i]
	}
	bwb, err := sortedBlockWithVerifiedBlobSlice(sbbs)
	require.NoError(t, err)
	return bwb, blobs
}

func TestVerifyAndPopulateBlobs(t *testing.T) {
	bwb, blobs := testSequenceBlockWithBlob(t, 10)
	lastBlobIdx := len(blobs) - 1
	// Blocks are all before the retention window, blobs argument is ignored.
	windowAfter := bwb[len(bwb)-1].Block.Block().Slot() + 1
	_, err := verifyAndPopulateBlobs(bwb, nil, windowAfter)
	require.NoError(t, err)

	firstBlockSlot := bwb[0].Block.Block().Slot()
	// slice off blobs for the last block so we hit the out of bounds / blob exhaustion check.
	_, err = verifyAndPopulateBlobs(bwb, blobs[0:len(blobs)-6], firstBlockSlot)
	require.ErrorIs(t, err, errMissingBlobsForBlockCommitments)

	bwb, blobs = testSequenceBlockWithBlob(t, 10)
	// Misalign the slots of the blobs for the first block to simulate them being missing from the response.
	offByOne := blobs[0].Slot()
	for i := range blobs {
		if blobs[i].Slot() == offByOne {
			blobs[i].SignedBlockHeader.Header.Slot = offByOne + 1
		}
	}
	_, err = verifyAndPopulateBlobs(bwb, blobs, firstBlockSlot)
	require.ErrorIs(t, err, verify.ErrBlobBlockMisaligned)

	bwb, blobs = testSequenceBlockWithBlob(t, 10)
	blobs[lastBlobIdx], err = blocks.NewROBlobWithRoot(blobs[lastBlobIdx].BlobSidecar, blobs[0].BlockRoot())
	require.NoError(t, err)
	_, err = verifyAndPopulateBlobs(bwb, blobs, firstBlockSlot)
	require.ErrorIs(t, err, verify.ErrBlobBlockMisaligned)

	bwb, blobs = testSequenceBlockWithBlob(t, 10)
	blobs[lastBlobIdx].Index = 100
	_, err = verifyAndPopulateBlobs(bwb, blobs, firstBlockSlot)
	require.ErrorIs(t, err, verify.ErrIncorrectBlobIndex)

	bwb, blobs = testSequenceBlockWithBlob(t, 10)
	blobs[lastBlobIdx].SignedBlockHeader.Header.ProposerIndex = 100
	blobs[lastBlobIdx], err = blocks.NewROBlob(blobs[lastBlobIdx].BlobSidecar)
	require.NoError(t, err)
	_, err = verifyAndPopulateBlobs(bwb, blobs, firstBlockSlot)
	require.ErrorIs(t, err, verify.ErrBlobBlockMisaligned)

	bwb, blobs = testSequenceBlockWithBlob(t, 10)
	blobs[lastBlobIdx].SignedBlockHeader.Header.ParentRoot = blobs[0].SignedBlockHeader.Header.ParentRoot
	blobs[lastBlobIdx], err = blocks.NewROBlob(blobs[lastBlobIdx].BlobSidecar)
	require.NoError(t, err)
	_, err = verifyAndPopulateBlobs(bwb, blobs, firstBlockSlot)
	require.ErrorIs(t, err, verify.ErrBlobBlockMisaligned)

	var emptyKzg [48]byte
	bwb, blobs = testSequenceBlockWithBlob(t, 10)
	blobs[lastBlobIdx].KzgCommitment = emptyKzg[:]
	blobs[lastBlobIdx], err = blocks.NewROBlob(blobs[lastBlobIdx].BlobSidecar)
	require.NoError(t, err)
	_, err = verifyAndPopulateBlobs(bwb, blobs, firstBlockSlot)
	require.ErrorIs(t, err, verify.ErrMismatchedBlobCommitments)

	// happy path
	bwb, blobs = testSequenceBlockWithBlob(t, 10)

	expectedCommits := make(map[[48]byte]bool)
	for _, bl := range blobs {
		expectedCommits[bytesutil.ToBytes48(bl.KzgCommitment)] = true
	}
	// The assertions using this map expect all commitments to be unique, so make sure that stays true.
	require.Equal(t, len(blobs), len(expectedCommits))

	bwb, err = verifyAndPopulateBlobs(bwb, blobs, firstBlockSlot)
	require.NoError(t, err)
	for _, bw := range bwb {
		commits, err := bw.Block.Block().Body().BlobKzgCommitments()
		require.NoError(t, err)
		require.Equal(t, len(commits), len(bw.Blobs))
		for i := range commits {
			bc := bytesutil.ToBytes48(commits[i])
			require.Equal(t, bc, bytesutil.ToBytes48(bw.Blobs[i].KzgCommitment))
			// Since we delete entries we've seen, duplicates will cause an error here.
			_, ok := expectedCommits[bc]
			// Make sure this was an expected delete, then delete it from the map so we can make sure we saw all of them.
			require.Equal(t, true, ok)
			delete(expectedCommits, bc)
		}
	}
	// We delete each entry we've seen, so if we see all expected commits, the map should be empty at the end.
	require.Equal(t, 0, len(expectedCommits))
}
